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

    /// compact report mode (hides charts and descriptions)
    #[argh(switch, short = 'c')]
    pub compact_report: bool,
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
        charts_dir: Some(ctx.config.current_dir.join("charts")),
        compact_mode: cmd.compact_report,
    };

    super::run::run_benchmarks(
        benchmarks,
        &ctx.config,
        cmd.duration.as_deref(),
        report_options,
    )
    .await
}
