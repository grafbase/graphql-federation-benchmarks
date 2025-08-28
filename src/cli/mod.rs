pub mod list;
pub mod run;

use std::path::PathBuf;

use argh::FromArgs;
use bollard::Docker;

use crate::benchmark::Benchmark;

#[derive(FromArgs)]
/// GraphQL Federation Benchmark Runner
pub struct Cli {
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Run(run::Command),
    List(list::Command),
}

pub struct Context {
    docker: Docker,
    current_dir: PathBuf,
}

impl Context {
    pub fn new(current_dir: PathBuf) -> anyhow::Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| anyhow::anyhow!("Failed to connect to Docker: {}", e))?;
        Ok(Self {
            docker,
            current_dir,
        })
    }

    pub fn load_benchmarks(
        &self,
        gateway_filter: Option<String>,
        benchmark_filter: Option<String>,
    ) -> anyhow::Result<Vec<Benchmark>> {
        let gateways = crate::gateway::load(&self.current_dir, gateway_filter)?;
        crate::benchmark::load(&self.docker, &gateways, &self.current_dir, benchmark_filter)
    }
}
