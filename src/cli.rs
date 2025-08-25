use argh::FromArgs;

#[derive(FromArgs)]
/// GraphQL Federation Benchmark Runner
pub struct Cli {
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Run(RunCommand),
    List(ListCommand),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run benchmarks
pub struct RunCommand {
    /// filter by benchmark name
    #[argh(option, short = 'b')]
    pub benchmark: Option<String>,

    /// filter by gateway name
    #[argh(option, short = 'g')]
    pub gateway: Option<String>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
/// List available benchmarks and gateways
pub struct ListCommand {}
