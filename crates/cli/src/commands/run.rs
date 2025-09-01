use argh::FromArgs;

use crate::{
    benchmark::{Benchmark, load_benchmarks},
    commands::Context,
    config::Config,
    report,
    system::SystemInfo,
};

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run benchmarks
pub struct Command {
    /// benchmark configuration name (defaults to "default")
    #[argh(positional, default = "String::from(\"default\")")]
    pub name: String,

    /// override K6 test duration (e.g., "30s", "1m", "2m30s")
    #[argh(option, short = 'd')]
    pub duration: Option<String>,
}

pub async fn main(ctx: Context, cmd: Command) -> anyhow::Result<()> {
    let benchmarks = load_benchmarks(&ctx.docker, &ctx.config, &cmd.name)?;

    run_benchmarks(benchmarks, &ctx.config, cmd.duration.as_deref()).await
}

pub async fn run_benchmarks(
    benchmarks: Vec<Benchmark>,
    config: &Config,
    duration: Option<&str>,
) -> anyhow::Result<()> {
    let mut results = Vec::new();

    for mut benchmark in benchmarks {
        tracing::info!(
            "=== Running benchmark '{}' with gateway '{}' ===",
            benchmark.name(),
            benchmark.gateway().name()
        );

        match benchmark.run(duration).await {
            Ok(result) => {
                results.push(result);
            }
            Err(e) => {
                tracing::error!("Failed to run benchmark: {}", e);
            }
        }

        // Always cleanup
        benchmark.cleanup().await;
    }

    // Generate and print the markdown report
    if !results.is_empty() {
        let system_info = SystemInfo::detect()?;
        let report = report::generate_report(
            time::OffsetDateTime::now_utc(),
            &results,
            &system_info,
            config,
        )?;
        println!("\n{}", report);
    }

    Ok(())
}
