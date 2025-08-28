use argh::FromArgs;

use crate::cli::Context;

#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
/// List available benchmarks and gateways
pub struct Command {}

pub async fn main(ctx: Context, _cmd: Command) -> anyhow::Result<()> {
    let benchmarks = ctx.load_benchmarks(None, None)?;
    if benchmarks.is_empty() {
        tracing::info!("No benchmarks found");
        return Ok(());
    }

    println!("=== Available Benchmarks ===");
    let mut current_benchmark = String::new();
    for benchmark in &benchmarks {
        let benchmark_name = benchmark.name();
        if benchmark_name != current_benchmark {
            println!("\n{}", benchmark_name);
            current_benchmark = benchmark_name.to_string();
        }
        println!("  - {}", benchmark.gateway().name());
    }

    Ok(())
}
