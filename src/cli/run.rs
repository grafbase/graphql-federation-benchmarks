use argh::FromArgs;

use crate::{cli::Context, report, system::SystemInfo};

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run benchmarks
pub struct Command {
    /// filter by benchmark name
    #[argh(option, short = 'b')]
    pub benchmark: Option<String>,

    /// filter by gateway name
    #[argh(option, short = 'g')]
    pub gateway: Option<String>,
}

pub async fn main(ctx: Context, cmd: Command) -> anyhow::Result<()> {
    let benchmarks = ctx.load_benchmarks(cmd.gateway, cmd.benchmark)?;

    if benchmarks.is_empty() {
        tracing::info!("No benchmarks found matching the filter");
        return Ok(());
    }

    let mut results = Vec::new();

    for mut benchmark in benchmarks {
        tracing::info!(
            "=== Running benchmark '{}' with gateway '{}' ===",
            benchmark.name(),
            benchmark.gateway().name()
        );

        match benchmark.run().await {
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
        let report =
            report::generate_report(time::OffsetDateTime::now_utc(), &results, &system_info);
        println!("\n{}", report);
    }

    Ok(())
}
