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
}

pub async fn main(ctx: Context, cmd: Command) -> anyhow::Result<()> {
    let benchmarks = load_benchmarks(&ctx.docker, &ctx.config, &cmd.name)?;

    run_benchmarks(
        benchmarks,
        &ctx.config,
        cmd.duration.as_deref(),
    )
    .await
}

pub async fn run_benchmarks(
    benchmarks: Vec<Benchmark>,
    config: &Config,
    duration: Option<&str>,
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
        let timestamp = time::OffsetDateTime::now_utc();
        
        // Print TTY report to terminal
        let tty_report = report::generate_report_with_options(
            timestamp,
            &results,
            &system_info,
            config,
            &ReportOptions { is_tty: true },
        )?;
        println!("\n{}", tty_report);
        
        // Write full report to REPORT.md
        let full_report = report::generate_report_with_options(
            timestamp,
            &results,
            &system_info,
            config,
            &ReportOptions { is_tty: false },
        )?;
        let report_path = config.current_dir.join("REPORT.md");
        std::fs::write(&report_path, full_report)?;
        tracing::info!("Full report written to {:?}", report_path);
        
        // Write charts to the charts directory
        let charts_dir = config.current_dir.join("charts");
        crate::charts::write_charts(&results, config, &charts_dir)?;
        tracing::info!("Charts written to {:?}", charts_dir);
    }

    Ok(())
}
