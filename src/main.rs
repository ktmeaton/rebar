use clap::Parser;
use color_eyre::eyre::{Report, Result};
use rebar::dataset::{download, list};
use rebar::{cli, cli::Cli};

#[tokio::main]
async fn main() -> Result<(), Report> {
    // ------------------------------------------------------------------------
    // CLI Setup

    // Parse CLI parameters
    let args = Cli::parse();

    // initialize color_eyre crate for colorized logs
    color_eyre::install()?;

    // Set logging/verbosity level via RUST_LOG
    std::env::set_var("RUST_LOG", args.verbosity.to_string());

    // initialize env_logger crate for logging/verbosity level
    env_logger::init();

    // check which CLI command we're running (dataset, run, plot)
    match args.command {
        // Dataset
        cli::Command::Dataset(args) => match args.command {
            cli::dataset::Command::List(args) => _ = list::datasets(&args)?,
            cli::dataset::Command::Download(mut args) => _ = download::dataset(&mut args).await?,
        },
        _ => (),
        // // Run
        // Command::Run(mut args) => rebar::run::run(&mut args)?,
        // // Plot
        // Command::Plot(args) => rebar::plot::plot(&args)?,
        // // Simulate
        // Command::Simulate(args) => rebar::simulate::simulate(&args)?,
    }

    Ok(())
}
