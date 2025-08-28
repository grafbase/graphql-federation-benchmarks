use anyhow::{Context as _, Result};
use fast_glob::glob_match;
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::docker::{self, ContainerId};

pub fn load(current_dir: &Path, filter: Option<String>) -> Result<Vec<Arc<Gateway>>> {
    let gateways_path = current_dir.join("gateways");
    let path = gateways_path.join("config.toml");
    let content = std::fs::read_to_string(&path).context("Could not read configuration")?;
    let gateways: HashMap<String, Config> =
        toml::from_str(&content).context("Could not parse configuration")?;

    Ok(gateways
        .into_iter()
        .filter(|(name, _)| {
            if let Some(ref filter) = filter {
                glob_match(filter, name)
            } else {
                true
            }
        })
        .map(|(name, config)| {
            Arc::new(Gateway {
                gateways_path: gateways_path.clone(),
                name: name.to_lowercase(),
                config,
            })
        })
        .collect())
}

pub struct Gateway {
    name: String,
    gateways_path: PathBuf,
    config: Config,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub label: String,
    pub image: String,
    pub arguments: Vec<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

impl Gateway {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn label(&self) -> &str {
        &self.config.label
    }

    pub fn start(&self, current_dir: &Path, args: Vec<String>) -> Result<ContainerId> {
        let bench_path = current_dir;

        let volumes = vec![
            (
                self.gateways_path.to_string_lossy().to_string(),
                "/gateways".to_string(),
            ),
            (
                bench_path.to_string_lossy().to_string(),
                "/benchmark".to_string(),
            ),
        ];

        // Combine gateway arguments with provided args
        let mut all_args = self.config.arguments.clone();
        all_args.extend(args);

        docker::run(
            &self.config.image,
            self.config
                .environment
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
            volumes.into_iter(),
            all_args.into_iter(),
        )
    }
}

pub async fn wait_for_gateway_health_with_logs(container_id: &str) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

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
