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

    /// compact report mode (hides charts and descriptions)
    #[argh(switch, short = 'c')]
    pub compact_report: bool,
}

pub async fn main(ctx: Context, cmd: Command) -> anyhow::Result<()> {
    let benchmarks = load_benchmarks(&ctx.docker, &ctx.config, &cmd.name)?;

    let report_options = ReportOptions {
        charts_dir: Some(ctx.config.current_dir.join("charts")),
        compact_mode: cmd.compact_report,
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
    // Clean up any existing Docker containers before starting
    tracing::info!("Cleaning up existing Docker containers...");
    let mut results = Vec::new();

    for mut benchmark in benchmarks {
        tracing::info!(
            "=== Running benchmark '{}' with gateway '{}' ===",
            benchmark.name(),
            benchmark.gateway().name()
        );
        let clean_result = std::process::Command::new("sh")
            .arg("./docker-clean.sh")
            .output();

        match clean_result {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // Don't fail if cleanup fails - there might be no containers to clean
                    tracing::debug!("Docker cleanup script returned non-zero status: {}", stderr);
                } else {
                    tracing::info!("Docker cleanup completed successfully");
                }
            }
            Err(e) => {
                tracing::warn!("Could not run docker cleanup script: {}", e);
            }
        }

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
