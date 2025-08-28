use anyhow::{Context as _, Result};
use bollard::Docker;
use fast_glob::glob_match;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    docker::{self, ContainerId},
    gateway::{wait_for_gateway_health_with_logs, Gateway},
    k6::{self, K6Run},
    resources::{DockerStatsCollector, ResourceStats},
};

pub fn load(
    docker: &Docker,
    gateways: &[Arc<Gateway>],
    current_dir: &Path,
    filter: Option<String>,
) -> Result<Vec<Benchmark>> {
    let benchmarks_dir = current_dir.join("benchmarks");
    if !benchmarks_dir.exists() {
        return Err(anyhow::anyhow!(
            "Benchmarks directory not found at {:?}",
            benchmarks_dir
        ));
    }

    let mut benchmarks = Vec::new();

    for entry in std::fs::read_dir(&benchmarks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().unwrap().to_str().unwrap().to_lowercase();

        // Load benchmark config
        let config_path = path.join("config.toml");
        let config_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Could not read config for benchmark '{}'", name))?;

        let local_benchmarks: HashMap<String, Config> = toml::from_str(&config_content)
            .with_context(|| format!("Could not parse config for benchmark '{}'", name))?;
        let n = local_benchmarks.len();

        // Process each benchmark configuration
        for (suffix, config) in local_benchmarks {
            let name = if suffix == "default" && n == 1 {
                name.clone()
            } else {
                format!("{name}-{suffix}").to_lowercase()
            };

            // Apply benchmark filter using glob pattern
            if let Some(ref filter) = filter {
                if !glob_match(filter, &name) {
                    continue;
                }
            }

            for (gateway_name, gateway_config) in config.gateways {
                // Find matching gateway
                let gateway = gateways.iter().find(|g| g.name() == gateway_name).cloned();

                if let Some(gateway) = gateway {
                    benchmarks.push(Benchmark {
                        docker: docker.clone(),
                        path: path.clone(),
                        gateway,
                        name: name.clone(),
                        k6_script: config.k6_script.clone(),
                        gateway_args: gateway_config.args,
                        container_id: None,
                    });
                }
            }
        }
    }

    benchmarks.sort_by(|a, b| {
        a.name()
            .cmp(b.name())
            .then_with(|| a.gateway().name().cmp(b.gateway().name()))
    });

    Ok(benchmarks)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    k6_script: String,
    gateways: HashMap<String, GatewayConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatewayConfig {
    #[serde(default)]
    args: Vec<String>,
}

pub struct Benchmark {
    docker: Docker,
    path: PathBuf,
    gateway: Arc<Gateway>,
    name: String,
    k6_script: String,
    gateway_args: Vec<String>,
    container_id: Option<ContainerId>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    pub benchmark: String,
    pub gateway: String,
    pub k6_run: K6Run,
    pub resource_stats: ResourceStats,
}

impl Benchmark {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn gateway(&self) -> &Arc<Gateway> {
        &self.gateway
    }

    pub async fn run(&mut self) -> Result<BenchmarkResult> {
        // Start subgraphs
        docker::compose_up(&self.path)?;

        // Start gateway
        let container_id = self.gateway.start(&self.path, self.gateway_args.clone())?;
        self.container_id = Some(container_id.clone());

        // Start metrics collection
        let collector = DockerStatsCollector::start(self.docker.clone(), &container_id).await?;

        // Start log streaming and wait for gateway to be healthy
        wait_for_gateway_health_with_logs(&container_id).await?;

        // Run K6 test
        let k6_run = k6::run(&self.path, &self.k6_script).await?;

        // Stop collection and get filtered stats
        let resource_stats = collector.stop_and_filter(k6_run.start, k6_run.end).await?;

        // Build result
        Ok(BenchmarkResult {
            benchmark: self.name().to_string(),
            gateway: self.gateway.label().to_string(),
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
}
