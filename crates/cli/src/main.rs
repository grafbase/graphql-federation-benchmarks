mod benchmark;
mod commands;
mod config;
mod docker;
mod gateway;
mod k6;
mod report;
mod resources;
mod system;

use anyhow::Result;
use commands::{CliArgs, Command};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::commands::Context;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(std::env::var("RUST_LOG").is_ok()))
        .init();

    let args: CliArgs = argh::from_env();
    let current_dir = std::env::current_dir()?;
    let ctx = Context::new(current_dir)?;

    match args.command {
        Command::Bench(args) => {
            commands::bench::main(ctx, args).await?;
        }
        Command::List(args) => {
            commands::list::main(ctx, args).await?;
        }
        Command::Run(args) => {
            commands::run::main(ctx, args).await?;
        }
    }

    Ok(())
}
