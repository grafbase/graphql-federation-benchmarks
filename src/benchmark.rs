use anyhow::Result;
use bollard::Docker;
use serde::Serialize;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::{
    config::GatewayConfig,
    docker,
    k6::{self, K6Run},
    resources::{DockerStatsCollector, ResourceStats},
};

pub struct Benchmark<'a> {
    path: PathBuf,
    gateway_name: &'a str,
    gateway_config: &'a GatewayConfig,
    docker: &'a Docker,
    container_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    pub benchmark: String,
    pub gateway: String,
    pub k6_run: K6Run,
    pub resource_stats: ResourceStats,
}

impl<'a> Benchmark<'a> {
    pub fn new(
        docker: &'a Docker,
        path: PathBuf,
        gateway_name: &'a str,
        gateway_config: &'a GatewayConfig,
    ) -> Self {
        Self {
            path,
            gateway_name,
            gateway_config,
            docker,
            container_id: None,
        }
    }

    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn gateway_name(&self) -> &'a str {
        self.gateway_name
    }

    pub async fn run(&mut self) -> Result<BenchmarkResult> {
        // Start subgraphs
        docker::compose_up(&self.path)?;

        // Start gateway
        let container_id = docker::start_gateway(
            self.gateway_config,
            &self.path.join("gateways").join(self.gateway_name),
        )?;
        self.container_id = Some(container_id.clone());

        // Start metrics collection
        let collector = DockerStatsCollector::start(self.docker.clone(), &container_id).await?;

        // Start log streaming and wait for gateway to be healthy
        self.wait_for_gateway_health_with_logs(&container_id)
            .await?;

        // Run K6 test
        let k6_run = k6::run(&self.path).await?;

        // Stop collection and get filtered stats
        let resource_stats = collector.stop_and_filter(k6_run.start, k6_run.end).await?;

        // Build result
        Ok(BenchmarkResult {
            benchmark: self.name(),
            gateway: self.gateway_config.label.clone(),
            k6_run,
            resource_stats,
        })
    }

    pub async fn cleanup(self) {
        // Stop gateway container if it exists
        if let Some(container_id) = self.container_id {
            if let Err(e) = docker::stop(&container_id) {
                tracing::error!("Failed to stop container: {}", e);
            }
        }

        // Stop subgraphs
        if let Err(e) = docker::compose_down(&self.path) {
            tracing::error!("Failed to stop subgraphs: {}", e);
        }
    }

    async fn wait_for_gateway_health_with_logs(&self, container_id: &str) -> Result<()> {
        const WAIT_DURATION_S: u64 = 30;
        let client = reqwest::Client::new();
        let health_query = r#"{"query":"{ __typename }"}"#;
        let expected_response = r#"{"data":{"__typename":"Query"}}"#;

        tracing::info!("Waiting for gateway to be healthy...");

        // Start log streaming process
        let mut log_process = Command::new("docker")
            .args(["logs", "-f", container_id])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Create readers for stdout and stderr
        let stdout = log_process.stdout.take().expect("Failed to get stdout");
        let stderr = log_process.stderr.take().expect("Failed to get stderr");
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        // Spawn tasks to read and print logs
        let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let log_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => println!("{}", line),
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                    line = stderr_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => eprintln!("{}", line),
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                    _ = log_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Health check loop
        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < WAIT_DURATION_S {
            if let Ok(response) = client
                .post("http://localhost:4000")
                .header("Content-Type", "application/json")
                .body(health_query)
                .send()
                .await
            {
                if response.status().is_success() {
                    let body = response.text().await?;
                    if body.contains(expected_response) {
                        tracing::info!("Gateway is healthy");

                        // Stop log streaming
                        let _ = log_tx.send(());
                        let _ = log_process.kill().await;
                        let _ = log_handle.await;

                        return Ok(());
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // Stop log streaming on timeout
        let _ = log_tx.send(());
        let _ = log_process.kill().await;
        let _ = log_handle.await;

        Err(anyhow::anyhow!(
            "Gateway did not become healthy after {} seconds",
            WAIT_DURATION_S
        ))
    }
}
