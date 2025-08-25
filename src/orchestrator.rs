use anyhow::{Context, Result};
use bollard::Docker;
use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::config::Config;

pub struct Orchestrator {
    docker: Docker,
    current_dir: PathBuf,
    config: Config,
}

impl Orchestrator {
    pub async fn new(current_dir: PathBuf) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| anyhow::anyhow!("Failed to connect to Docker: {}", e))?;

        let config_path = current_dir.join("config.toml");
        let config_content =
            std::fs::read_to_string(&config_path).context("Could not read configuration")?;
        let config = toml::from_str(&config_content).context("Could not parse configuration")?;

        Ok(Self {
            docker,
            current_dir,
            config,
        })
    }

    fn create_all_benchmarks(
        &self,
        benchmark_filter: Option<String>,
        gateway_filter: Option<String>,
    ) -> Result<Vec<Benchmark<'_>>> {
        let gateways = self
            .config
            .gateways
            .keys()
            .filter(|name| {
                gateway_filter
                    .as_ref()
                    .is_none_or(|filter| filter.as_str() == name.as_str())
            })
            .collect::<Vec<_>>();
        let benchmark_paths = self.discover_benchmarks(benchmark_filter)?;

        let mut benchmarks = Vec::new();

        for benchmark_path in benchmark_paths {
            for gateway_name in gateways.iter().copied() {
                if !benchmark_path.join("gateways").join(gateway_name).exists() {
                    continue;
                }

                let gateway_config = &self.config.gateways[gateway_name];
                let benchmark = Benchmark::new(
                    self.docker.clone(),
                    benchmark_path.clone(),
                    gateway_name,
                    gateway_config,
                );

                benchmarks.push(benchmark);
            }
        }

        Ok(benchmarks)
    }

    pub async fn run(
        &self,
        benchmark_filter: Option<String>,
        gateway_filter: Option<String>,
    ) -> Result<()> {
        let benchmarks = self.create_all_benchmarks(benchmark_filter, gateway_filter)?;

        if benchmarks.is_empty() {
            println!("No benchmarks found matching the filter");
            return Ok(());
        }

        for mut benchmark in benchmarks {
            println!(
                "\n=== Running benchmark '{}' with gateway '{}' ===",
                benchmark.name(),
                benchmark.gateway_name()
            );

            match benchmark.run().await {
                Ok(result) => {
                    println!("\n{}", serde_json::to_string_pretty(&result)?);
                }
                Err(e) => {
                    tracing::error!("Failed to run benchmark: {}", e);
                }
            }

            // Always cleanup
            benchmark.cleanup().await;
        }

        Ok(())
    }

    pub async fn list(&self) -> Result<()> {
        let benchmarks = self.create_all_benchmarks(None, None)?;

        if benchmarks.is_empty() {
            println!("No benchmarks found");
            return Ok(());
        }

        println!("\n=== Available Benchmarks ===");
        let mut current_benchmark = String::new();
        for benchmark in benchmarks {
            let benchmark_name = benchmark.name();
            if benchmark_name != current_benchmark {
                println!("\n{}", benchmark_name);
                current_benchmark = benchmark_name.clone();
            }
            println!("  - {}", benchmark.gateway_name());
        }

        Ok(())
    }

    fn discover_benchmarks(&self, filter: Option<String>) -> Result<Vec<PathBuf>> {
        let benchmarks_dir = self.current_dir.join("benchmarks");

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

            if path.is_dir() {
                let name = path.file_name().unwrap().to_str().unwrap();

                if let Some(ref filter) = filter {
                    if name == filter {
                        // Convert to absolute path
                        benchmarks.push(path.canonicalize()?);
                    }
                } else {
                    // Convert to absolute path
                    benchmarks.push(path.canonicalize()?);
                }
            }
        }

        benchmarks.sort();
        Ok(benchmarks)
    }
}
