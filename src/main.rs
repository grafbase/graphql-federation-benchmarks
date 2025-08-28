mod benchmark;
mod cli;
mod docker;
mod gateway;
mod k6;
mod report;
mod resources;
mod system;

use anyhow::Result;
use cli::{Cli, Command};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::Context;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(std::env::var("RUST_LOG").is_ok()))
        .init();

    let args: Cli = argh::from_env();
    let current_dir = std::env::current_dir()?;
    let ctx = Context::new(current_dir)?;

    match args.command {
        Command::Run(args) => {
            cli::run::main(ctx, args).await?;
        }
        Command::List(args) => {
            cli::list::main(ctx, args).await?;
        }
    }

    Ok(())
}
