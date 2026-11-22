mod cli;
mod client;
mod registry;
mod runner;
mod setup;

pub use client::Client;
pub use registry::Registry;
pub use runner::Runner;
pub use setup::SetupInfo;

// Re-exported so that `register_part!` can name the `linkme` items it expands
// to without requiring downstream crates to depend on `linkme` themselves.
#[doc(hidden)]
pub use linkme as __linkme;

#[linkme::distributed_slice]
pub static RUNNERS: [Runner];

pub fn run(client: &dyn Client, event: &str, setup_info: SetupInfo) {
    use clap::Parser as _;

    let args = cli::Cli::parse();
    let registry = &*registry::REGISTRY;

    if let Some(cmd) = args.subcommand {
        match cmd {
            cli::CliCmd::Setup => run_setup(&registry, setup_info),
            cli::CliCmd::Run(run_cmd) => run_runner(client, event, registry, &run_cmd),
        }
    } else {
        run_runner(client, event, registry, &args.run_args)
    }
}

fn run_setup(registry: &Registry, setup_info: SetupInfo) {
    let day = registry.get_last_day().map_or(1, |day| day + 1);
    setup_info.write_template(day);
}

fn run_runner(client: &dyn Client, event: &str, registry: &Registry, args: &cli::RunCmd) {
    if args.all {
        registry.run_all(client, event);
    } else if let Some(day) = args.day {
        if let Some(part) = args.part {
            registry.run_part(client, event, day, part);
        } else {
            registry.run_day(client, event, day);
        }
    } else {
        registry.run_last_day(client, event);
    }
}
