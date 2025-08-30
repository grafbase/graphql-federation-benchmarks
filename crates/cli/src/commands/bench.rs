use argh::FromArgs;

use crate::commands::Context;

#[derive(FromArgs)]
#[argh(subcommand, name = "bench")]
/// Run benchmarks with specific gateways and scenarios
pub struct Command {
    /// comma-separated gateway names (e.g., "grafbase,apollo-router")
    #[argh(option, short = 'g')]
    pub gateway: String,

    /// comma-separated scenario names (e.g., "big-response,many-plans")
    #[argh(option, short = 's')]
    pub scenario: String,

    /// override K6 test duration (e.g., "30s", "1m", "2m30s")
    #[argh(option, short = 'd')]
    pub duration: Option<String>,
}

pub async fn main(ctx: Context, cmd: Command) -> anyhow::Result<()> {
    // Parse comma-separated gateways and scenarios
    let gateways: Vec<&str> = cmd.gateway.split(',').map(|s| s.trim()).collect();
    let scenarios: Vec<&str> = cmd.scenario.split(',').map(|s| s.trim()).collect();
    let benchmarks =
        crate::benchmark::create_benchmarks(&ctx.docker, &ctx.config, &gateways, &scenarios)?;

    super::run::run_benchmarks(benchmarks, cmd.duration.as_deref()).await
}
