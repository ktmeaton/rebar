// System paths
// use std::path::Path;
//use tempfile::Builder;
// Logging
//use log::info;
use clap::Parser;
use color_eyre::eyre::{Result, Report};
use rebar::cli::log::LogVerbosity;
use rebar::cli::Cli;
use rebar::dataset::Dataset;
use rebar::traits::ToYaml;
use std::env;
use std::path::Path;
use std::str::FromStr;

fn setup () -> Result<(), Report> {

    color_eyre::install()?;

    let log_verbosity = LogVerbosity::from_str("debug").unwrap();

    // Set default logging level if RUST_LOG is not set.
    //let log_verbosity = "debug";
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", log_verbosity.to_string())
    }

    env_logger::init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {

    // Misc setup actions like logging
    setup().unwrap();

    // Parse CLI parameters
    let cli = Cli::parse();
    println!("{:?}", cli);

    // These will become CLI parameters
    let dataset_name = "sars-cov-2";
    //let dataset_reference = "MN908947";
    let dataset_version = "nightly";
    let dataset_dir = Path::new("dataset/sars-cov-2/nightly");
    // let mask = 200;

    // ------------------------------------------------------------------------
    // Dataset Creation

    let dataset = Dataset::new(
        dataset_name.to_string(),
        dataset_version.to_string(),
        dataset_dir,
    )?;
    //dataset.populations = dataset.download_sequences(&dataset_outdir)?;
    println!("{}", dataset.to_yaml());
    //let dataset = Dataset::create(dataset_name.to_string(), dataset_tag.to_string(), mask);

    // Sequences
    // dataset.populations.set_sequences(&reference_path, &populations_path, &mask).unwrap();
    // dataset.populations.set_mutations().unwrap();

    // // Phylogeny
    // info!("Preparing dataset phylogeny: {}", &phylogeny_path.display());
    // let phylogeny
    // dataset.phylogeny.build_graph(&dataset_name, &dataset_tag, &dataset_dir).expect("Failed to build phylogeny.");
    // dataset.phylogeny.export_graph(&dataset_dir).expect("Failed to export phylogeny.");

    // // Sequences
    // info!("Preparing dataset sequences: {}", &populations_path.display());
    // dataset.populations.set_sequences(&reference_path, &populations_path, &mask).unwrap();
    // dataset.populations.set_mutations().unwrap();

    // // ------------------------------------------------------------------------
    // // Run

    // info!("Importing query sequences: {}", &sequences_path.display());
    // let mut query = Sequences::new();
    // query.set_sequences(&reference_path, &sequences_path, &mask).unwrap();
    // query.summarise_barcodes(&dataset).unwrap();

    // 1. Sumarise the barcodes (conflicts, support, total), etc.
    // 2. Assign consensus population (Barcode Match)
    // 3. Find recombination parents (Barcode Match)

    Ok(())
}
