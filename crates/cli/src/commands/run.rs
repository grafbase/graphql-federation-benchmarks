use argh::FromArgs;

use crate::{
    benchmark::{Benchmark, load_benchmarks},
    commands::Context,
    config::Config,
    report::{self, ReportOptions},
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

    /// generate SVG charts for latency metrics
    #[argh(switch, short = 'c')]
    pub charts: bool,

    /// directory to save charts (defaults to embedding in report)
    #[argh(option)]
    pub charts_dir: Option<String>,
}

pub async fn main(ctx: Context, cmd: Command) -> anyhow::Result<()> {
    let benchmarks = load_benchmarks(&ctx.docker, &ctx.config, &cmd.name)?;

    let report_options = ReportOptions {
        generate_charts: cmd.charts,
        charts_dir: cmd.charts_dir.map(std::path::PathBuf::from),
    };

    run_benchmarks(
        benchmarks,
        &ctx.config,
        cmd.duration.as_deref(),
        report_options,
    )
    .await
}

pub async fn run_benchmarks(
    benchmarks: Vec<Benchmark>,
    config: &Config,
    duration: Option<&str>,
    report_options: ReportOptions,
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
        let report = report::generate_report_with_options(
            time::OffsetDateTime::now_utc(),
            &results,
            &system_info,
            config,
            &report_options,
        )?;
        println!("\n{}", report);
    }

    Ok(())
}
