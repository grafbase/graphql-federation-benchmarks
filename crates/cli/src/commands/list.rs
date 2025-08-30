use argh::FromArgs;

use crate::{benchmark::load_benchmarks, commands::Context};

#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
/// List available benchmark configurations
pub struct Command {
    /// specific benchmark configuration to list details for
    #[argh(positional, default = "String::from(\"default\")")]
    pub name: String,
}

pub async fn main(ctx: Context, cmd: Command) -> anyhow::Result<()> {
    // List details for specific config
    let benchmarks = load_benchmarks(&ctx.docker, &ctx.config, &cmd.name)?;
    if benchmarks.is_empty() {
        println!("No benchmarks found in config '{}'", cmd.name);
        return Ok(());
    }

    println!("=== Benchmarks ===");
    let mut current_scenario = String::new();
    for benchmark in &benchmarks {
        let scenario_name = benchmark.name();
        if scenario_name != current_scenario {
            println!("\nScenario: {}", scenario_name);
            current_scenario = scenario_name.to_string();
        }
        println!("  - {}", benchmark.gateway().label());
    }

    Ok(())
}
