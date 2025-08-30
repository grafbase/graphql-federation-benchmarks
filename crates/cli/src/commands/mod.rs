pub mod bench;
pub mod list;
pub mod run;

use std::path::PathBuf;

use argh::FromArgs;
use bollard::Docker;

use crate::config::Config;

#[derive(FromArgs)]
/// GraphQL Federation Benchmark Runner
pub struct CliArgs {
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Bench(bench::Command),
    List(list::Command),
    Run(run::Command),
}

pub struct Context {
    docker: Docker,
    config: Config,
}

impl Context {
    pub fn new(current_dir: PathBuf) -> anyhow::Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| anyhow::anyhow!("Failed to connect to Docker: {}", e))?;
        let config = Config::load(current_dir)?;
        Ok(Self { docker, config })
    }
}
