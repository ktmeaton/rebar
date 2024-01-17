//use clap::Parser;
use color_eyre::eyre::{Report, Result};
//use rebar::{download, list};
//use rebar::dataset;
//use rebar::{cli, Cli};

#[tokio::main]
async fn main() -> Result<(), Report> {
    // use rebar::{phylogeny::Branch, phylogeny::Node, Phylogeny};
    // let node_1 = Node { label: "A" };
    // let node_2 = Node { label: "B" };
    // let branch = Branch { length: 1.0 };
    // let mut phylo = rebar::Phylogeny::new();
    // phylo.add_edge(node_1, node_2, branch);
    // println!("{}", phylo.to_mermaid()?);
    // // ------------------------------------------------------------------------
    // // CLI Setup

    // // Parse CLI parameters
    // let args = Cli::parse();

    // // initialize color_eyre crate for colorized logs
    // color_eyre::install()?;

    // // Set logging/verbosity level via RUST_LOG
    // std::env::set_var("RUST_LOG", args.verbosity.to_string());

    // // initialize env_logger crate for logging/verbosity level
    // env_logger::init();

    // // check which CLI command we're running (dataset, run, plot)
    // match args.command {
    //     // Dataset
    //     cli::Command::Dataset(args) => match args.command {
    //         cli::dataset::Command::List(args) => _ = dataset::list(&args)?,
    //         cli::dataset::Command::Download(args) => _ = dataset::download(&args).await?,
    //     },
    //     // // Run
    //     // Command::Run(mut args) => rebar::run::run(&mut args)?,
    //     // // Plot
    //     // Command::Plot(args) => rebar::plot::plot(&args)?,
    //     // // Simulate
    //     // Command::Simulate(args) => rebar::simulate::simulate(&args)?,
    // }

    Ok(())
}
