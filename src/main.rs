mod benchmark;
mod cli;
mod config;
mod docker;
mod k6;
mod orchestrator;
mod report;
mod resources;
mod system;

use anyhow::Result;
use cli::{Cli, Command};
use orchestrator::Orchestrator;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(std::env::var("RUST_LOG").is_ok())
        )
        .init();

    let cli: Cli = argh::from_env();
    let current_dir = std::env::current_dir()?;
    let orchestrator = Orchestrator::new(current_dir).await?;

    match cli.command {
        Command::Run(run_cmd) => {
            orchestrator.run(run_cmd.benchmark, run_cmd.gateway).await?;
        }
        Command::List(_) => {
            orchestrator.list().await?;
        }
    }

    Ok(())
}
