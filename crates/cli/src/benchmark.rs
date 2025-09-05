use anyhow::{Context as _, Result};
use bollard::Docker;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    config::{Config, Gateway},
    docker::{self, ContainerId},
    gateway::wait_for_gateway_health_with_logs,
    k6::{self, K6Run},
    resources::{DockerStatsCollector, ResourceStats},
};

pub fn create_benchmarks<S: AsRef<str>>(
    docker: &Docker,
    config: &Config,
    gateways: &[S],
    scenarios: &[S],
) -> anyhow::Result<Vec<Benchmark>> {
    let mut benchmarks = Vec::new();
    for (scenario_name, gateway_name) in scenarios.iter().cartesian_product(gateways.iter()) {
        let scenario_name: &str = scenario_name.as_ref();
        let gateway_name: &str = gateway_name.as_ref();

        let scenario_config = config.get_scenario(scenario_name)?;
        let supergraph_config = config.get_supergraph(&scenario_config.supergraph)?;
        let gateway = config.get_gateway(gateway_name)?;

        benchmarks.push(Benchmark {
            docker: docker.clone(),
            scenario_name: scenario_name.to_string(),
            scenario_path: config.current_dir.join("scenarios").join(scenario_name),
            supergraph_path: config
                .current_dir
                .join("supergraphs")
                .join(&scenario_config.supergraph),
            subgraphs: supergraph_config.subgraphs.clone(),
            compose_env: scenario_config.env.clone(),
            gateway,
            project_dir: config.current_dir.clone(),
            container_id: None,
        });
    }

    benchmarks.sort_by(|a, b| {
        a.scenario_name
            .cmp(&b.scenario_name)
            .then_with(|| a.gateway.name().cmp(b.gateway.name()))
    });

    Ok(benchmarks)
}

/// Load benchmarks from a configuration file
pub fn load_benchmarks(docker: &Docker, config: &Config, name: &str) -> Result<Vec<Benchmark>> {
    // Load benchmark configuration
    let config_path = config
        .current_dir
        .join("benchmarks")
        .join(format!("{}.toml", name));
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Benchmark configuration '{}' not found at {:?}",
            name,
            config_path
        ));
    }

    let config_content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Could not read benchmark config '{}'", name))?;

    let benchmark_config: BenchmarkConfig = toml::from_str(&config_content)
        .with_context(|| format!("Could not parse benchmark config '{}'", name))?;

    let mut benchmarks = Vec::new();

    // Process each benchmark entry
    for entry in &benchmark_config.benchmarks {
        // If gateways are empty, use all available gateways
        let gateways = if entry.gateway.is_empty() {
            config
                .gateways
                .iter()
                .map(|g| g.name().to_string())
                .collect()
        } else {
            entry.gateway.clone()
        };

        // If scenarios are empty, use all available scenarios
        let scenarios = if entry.scenario.is_empty() {
            config.scenarios.keys().cloned().collect()
        } else {
            entry.scenario.clone()
        };

        benchmarks.extend(create_benchmarks(docker, config, &gateways, &scenarios)?);
    }

    benchmarks.sort_by(|a, b| {
        a.scenario_name
            .cmp(&b.scenario_name)
            .then_with(|| a.gateway.name().cmp(b.gateway.name()))
    });

    Ok(benchmarks)
}

#[derive(Debug, Deserialize)]
struct BenchmarkConfig {
    benchmarks: Vec<BenchmarkEntry>,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
struct BenchmarkEntry {
    #[serde(default)]
    #[serde_as(as = "serde_with::OneOrMany<_>")]
    scenario: Vec<String>,
    #[serde(default)]
    #[serde_as(as = "serde_with::OneOrMany<_>")]
    gateway: Vec<String>,
}

pub struct Benchmark {
    docker: Docker,
    scenario_name: String,
    scenario_path: PathBuf,
    supergraph_path: PathBuf,
    subgraphs: Vec<String>,
    compose_env: HashMap<String, String>,
    gateway: Arc<Gateway>,
    project_dir: PathBuf,
    container_id: Option<ContainerId>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    pub scenario: String,
    pub gateway: String,
    pub k6_run: K6Run,
    pub resource_stats: ResourceStats,
}

impl BenchmarkResult {
    pub fn is_valid(&self) -> bool {
        !self.has_failures() && self.request_count() > 0
    }

    pub fn median_latency(&self) -> f64 {
        self.k6_run
            .summary
            .metrics
            .http_req_duration
            .as_ref()
            .map(|m| m.values.med)
            .unwrap_or(0.0)
    }

    /// Check if there are request failures
    pub fn has_failures(&self) -> bool {
        self.k6_run
            .summary
            .metrics
            .checks
            .as_ref()
            .map(|c| c.values.fails > 0)
            .unwrap_or(false)
    }

    /// Get the number of failures
    pub fn failure_count(&self) -> u64 {
        self.k6_run
            .summary
            .metrics
            .checks
            .as_ref()
            .map(|c| c.values.fails)
            .unwrap_or(0)
    }

    /// Get request count
    pub fn request_count(&self) -> u64 {
        self.k6_run
            .summary
            .metrics
            .http_req_duration
            .as_ref()
            .map(|m| m.values.count)
            .unwrap_or(0)
    }

    /// Calculate request rate per second
    pub fn request_rate(&self) -> f64 {
        self.k6_run
            .summary
            .metrics
            .http_reqs
            .as_ref()
            .map(|m| m.values.rate)
            .unwrap_or(0.0)
    }

    /// Calculate requests per CPU core second
    pub fn requests_per_core_s(&self) -> f64 {
        let rate = self.request_rate();
        if self.resource_stats.cpu_usage_max > 0.0 {
            rate / self.resource_stats.cpu_usage_max
        } else {
            0.0
        }
    }

    /// Calculate requests per GB second
    pub fn requests_per_gb_s(&self) -> f64 {
        let rate = self.request_rate();
        let memory_gb = self.resource_stats.memory_mib_max / 1024.0;
        if memory_gb > 0.0 {
            rate / memory_gb
        } else {
            0.0
        }
    }

    /// Calculate average subgraph requests per gateway request
    pub fn average_subgraph_requests(&self) -> f64 {
        let requests = self.request_count() as f64;
        if requests > 0.0 {
            self.k6_run.summary.subgraph_stats.count as f64 / requests
        } else {
            0.0
        }
    }

    /// Get total subgraph request count
    pub fn subgraph_request_count(&self) -> u64 {
        self.k6_run.summary.subgraph_stats.count
    }
}

impl Benchmark {
    pub fn name(&self) -> &str {
        &self.scenario_name
    }

    pub fn gateway(&self) -> &Arc<Gateway> {
        &self.gateway
    }

    pub async fn run(&mut self, duration: Option<&str>) -> Result<BenchmarkResult> {
        // Start subgraphs using the main compose file with specific services
        docker::compose_up(&self.project_dir, &self.subgraphs, &self.compose_env)?;

        // Start gateway with supergraph mount
        let container_id = self.gateway.start_with_supergraph(&self.supergraph_path)?;
        self.container_id = Some(container_id.clone());

        // Start metrics collection
        let collector = DockerStatsCollector::start(self.docker.clone(), &container_id).await?;

        // Start log streaming and wait for gateway to be healthy
        wait_for_gateway_health_with_logs(&container_id).await?;

        // Run K6 test from scenario directory
        let k6_script_path = self.scenario_path.join("k6.js");
        if !k6_script_path.exists() {
            return Err(anyhow::anyhow!(
                "K6 script not found at {:?}",
                k6_script_path
            ));
        }
        let k6_run = k6::run(&self.scenario_path, "k6.js", duration).await?;

        // Stop collection and get filtered stats
        let resource_stats = collector.stop_and_filter(k6_run.start, k6_run.end).await?;

        // Build result
        Ok(BenchmarkResult {
            scenario: self.scenario_name.clone(),
            gateway: self.gateway.label().to_string(),
            k6_run,
            resource_stats,
        })
    }

    pub async fn cleanup(self) {
        // Stop gateway container if it exists
        if let Some(container_id) = self.container_id
            && let Err(e) = docker::stop(&container_id)
        {
            tracing::error!("Failed to stop container: {}", e);
        }

        // Stop subgraphs
        if let Err(e) = docker::compose_down(&self.project_dir) {
            tracing::error!("Failed to stop subgraphs: {}", e);
        }
    }
}
