use argh::FromArgs;

use crate::{commands::Context, report::ReportOptions};

#[derive(FromArgs)]
#[argh(subcommand, name = "bench")]
/// Run benchmarks with specific gateways and scenarios
pub struct Command {
    /// comma-separated gateway names (e.g., "grafbase,apollo-router")
    #[argh(option, default = "String::new()", short = 'g')]
    pub gateway: String,

    /// comma-separated scenario names (e.g., "big-response,many-plans")
    #[argh(option, default = "String::new()", short = 's')]
    pub scenario: String,

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
    // Parse comma-separated gateways and scenarios
    let gateways: Vec<&str> = if cmd.gateway.is_empty() {
        ctx.config.gateways.iter().map(|g| g.name()).collect()
    } else {
        cmd.gateway.split(',').map(|s| s.trim()).collect()
    };
    let scenarios: Vec<&str> = if cmd.scenario.is_empty() {
        ctx.config
            .scenarios
            .keys()
            .map(|key| key.as_str())
            .collect()
    } else {
        cmd.scenario.split(',').map(|s| s.trim()).collect()
    };
    let benchmarks =
        crate::benchmark::create_benchmarks(&ctx.docker, &ctx.config, &gateways, &scenarios)?;

    let report_options = ReportOptions {
        generate_charts: cmd.charts,
        charts_dir: cmd.charts_dir.map(std::path::PathBuf::from),
    };

    super::run::run_benchmarks(
        benchmarks,
        &ctx.config,
        cmd.duration.as_deref(),
        report_options,
    )
    .await
}
