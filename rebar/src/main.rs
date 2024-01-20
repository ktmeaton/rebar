#[cfg(feature = "cli")]
use clap::Parser;
use color_eyre::eyre::{Report, Result};
#[cfg(feature = "cli")]
use rebar::dataset;
#[cfg(feature = "cli")]
use rebar::{cli, Cli};

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
            cli::Command::Dataset(args) => match args.command {
                // List datasets available for download as table
                cli::dataset::Command::List(args) => println!("{}", dataset::list(&args)?),
                //cli::dataset::Command::Download(args) => _ = dataset::download(&args).await?,
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
