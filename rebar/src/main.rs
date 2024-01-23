#[cfg(feature = "cli")]
use clap::Parser;
use color_eyre::eyre::{Report, Result};

#[cfg(feature = "cli")]
use rebar::cli::dataset::Command::{Download, List};
#[cfg(feature = "cli")]
use rebar::cli::Command::Dataset;
#[cfg(feature = "cli")]
use rebar::dataset::{download, list};
#[cfg(feature = "cli")]
use rebar::Cli;

#[tokio::main]
async fn main() -> Result<(), Report> {
    #[cfg(feature = "cli")]
    {
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
            Dataset(args) => match args.command {
                // List datasets available for download as table
                List(args) => println!("{}", list(&args)?),
                // Download available dataset
                Download(args) => _ = download(&args).await?,
            },
            // // Run
            // Command::Run(mut args) => rebar::run::run(&mut args)?,
            // // Plot
            // Command::Plot(args) => rebar::plot::plot(&args)?,
            // // Simulate
            // Command::Simulate(args) => rebar::simulate::simulate(&args)?,
        }
    }

    Ok(())
}
