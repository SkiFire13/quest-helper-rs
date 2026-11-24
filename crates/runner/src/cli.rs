use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(arg_required_else_help = false, args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[command(flatten)]
    pub run_args: RunCmd,

    #[command(subcommand)]
    pub subcommand: Option<CliCmd>,
}

#[derive(Args, Debug)]
pub struct RunCmd {
    /// Run every day
    #[arg(short, long, conflicts_with_all = ["day", "part"])]
    pub all: bool,

    /// Day to run
    #[arg(short, long)]
    pub day: Option<u32>,

    /// Part to run, requires --day
    #[arg(short, long, requires = "day")]
    pub part: Option<u32>,
}

#[derive(Subcommand, Debug)]
pub enum CliCmd {
    Setup {
        #[arg(short, long)]
        day: Option<u32>,
    },
    Run(RunCmd),
}
