use anyhow::Result;
use bollard::Docker;
use std::path::PathBuf;

use crate::config::GatewayConfig;
use crate::docker;
use crate::k6;
use crate::resources::DockerStatsCollector;
use crate::results::BenchmarkResult;

pub struct Benchmark<'a> {
    path: PathBuf,
    gateway_name: &'a str,
    gateway_config: &'a GatewayConfig,
    docker: Docker,
    container_id: Option<String>,
}

impl<'a> Benchmark<'a> {
    pub fn new(
        docker: Docker,
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

    pub async fn run(&mut self) -> Result<BenchmarkResult<'a>> {
        // Start subgraphs
        docker::compose_up(&self.path)?;

        // Start gateway
        let container_id =
            docker::start_gateway(self.gateway_name, self.gateway_config, &self.path)?;
        self.container_id = Some(container_id);

        // Start metrics collection
        let collector =
            DockerStatsCollector::start(self.docker.clone(), self.container_id.as_deref().unwrap())
                .await?;

        // Wait for gateway to be healthy
        self.wait_for_gateway_health().await?;

        // Run K6 test
        let k6_run = k6::run(&self.path).await?;

        // Stop collection and get filtered stats
        let resource_stats = collector.stop_and_filter(k6_run.start, k6_run.end).await?;

        // Build result
        Ok(BenchmarkResult {
            benchmark: self.name(),
            gateway: self.gateway_name,
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

    async fn wait_for_gateway_health(&self) -> Result<()> {
        const WAIT_DURATION_S: u64 = 30;
        let client = reqwest::Client::new();
        let health_query = r#"{"query":"{ __typename }"}"#;
        let expected_response = r#"{"data":{"__typename":"Query"}}"#;

        tracing::info!("Waiting for gateway to be healthy...");

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
                        return Ok(());
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        Err(anyhow::anyhow!(
            "Gateway did not become healthy after {} seconds",
            WAIT_DURATION_S
        ))
    }
}
