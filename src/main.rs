use clap::Parser;
use color_eyre::eyre::{Report, Result};
use rebar::dataset::{download, list};
use rebar::{cli, cli::Cli};

#[tokio::main]
async fn main() -> Result<(), Report> {
    use rebar::utils::{table, table::Table};
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut table = Table::new();
    table.headers = vec!["1", "2", "3"];
    table.add_row(vec!["A", "B", "C"]);
    table.set_row(0, vec!["AA", "BB", "CC"]);
    println!("{}", table.to_markdown().unwrap());

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
            cli::dataset::Command::List(args) => list::datasets(&args)?,
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
