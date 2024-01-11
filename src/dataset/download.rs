use crate::dataset::attributes::{is_compatible, Name, Summary, Tag};
// use crate::dataset::{sarscov2, toy1, Dataset};
use crate::dataset::{toy1, Dataset};
// use crate::{dataset, dataset::load};
use crate::{utils, utils::remote_file::RemoteFile, utils::table::Table};
use clap::Parser;
use color_eyre::eyre::{eyre, Report, Result};
// use itertools::Itertools;
use log::{info, warn};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

/// Download dataset arguments.
#[derive(Parser, Debug)]
#[clap(verbatim_doc_comment)]
pub struct Args {
    /// Dataset name.
    #[clap(short = 'r', long, required = true)]
    pub name: Name,

    /// Dataset tag.
    ///
    /// A date (YYYY-MM-DD), or 'nightly', or 'custom'
    #[clap(short = 't', long, required = true)]
    pub tag: Tag,

    /// Output directory.
    ///
    /// If the directory does not exist, it will be created.
    #[clap(short = 'o', long, required = true)]
    pub output_dir: PathBuf,

    /// Download dataset from a summary.json snapshot.
    #[clap(short = 's', long)]
    pub summary: Option<PathBuf>,
}

/// Download dataset
pub async fn dataset(args: &mut Args) -> Result<Dataset, Report> {
    info!("Downloading dataset: {} {}", &args.name, &args.tag);

    let dataset = Dataset::new();

    // --------------------------------------------------------------------
    // Optional Input Summary Snapshot

    let mut summary: Summary = if let Some(summary_path) = &args.summary {
        info!("Importing summary: {summary_path:?}");
        let summary = Summary::read(summary_path)?;

        // Warn if summary conflicts with any CLI args
        if summary.name != args.name || summary.tag != args.tag {
            warn!(
                "Dataset has been changed by summary to: {} {}",
                &summary.name, &summary.tag
            );
        }
        summary
    } else {
        let mut summary = Summary::new();
        summary.name = args.name;
        summary.tag = args.tag.clone();
        summary
    };

    // --------------------------------------------------------------------
    // Compatibility Check

    if !is_compatible(&args.name, &args.tag)? {
        Err(eyre!("Dataset incompatibility"))?;
    }

    // Warn if the directory already exists
    if !args.output_dir.exists() {
        info!("Creating output directory: {:?}", &args.output_dir);
        create_dir_all(&args.output_dir)?;
    } else {
        warn!(
            "Proceed with caution! --output-dir {:?} already exists.",
            args.output_dir
        );
    }

    // --------------------------------------------------------------------
    // Reference

    let output_path = args.output_dir.join("reference.fasta");
    info!("Downloading reference: {output_path:?}");

    summary.reference = if args.summary.is_some() {
        match summary.reference {
            Some(remote_file) => snapshot(&remote_file, &output_path).await.ok(),
            None => None,
        }
    } else {
        match args.name {
            //Name::SarsCov2 => sarscov2::download::reference(&args.tag, &output_path).await?,
            Name::Toy1 => toy1::create::reference(&args.tag, &output_path).ok(),
            _ => todo!(),
        }
    };

    // // --------------------------------------------------------------------
    // // Populations

    // let output_path = args.output_dir.join("populations.fasta");
    // info!("Downloading populations: {output_path:?}");

    // summary.populations = if args.summary.is_some() {
    //     dataset::download::snapshot(&summary.populations, &output_path).await?
    // } else {
    //     match args.name {
    //         Name::SarsCov2 => sarscov2::download::populations(&args.tag, &output_path).await?,
    //         Name::Toy1 => toy1::download::populations(&args.tag, &output_path)?,
    //         _ => todo!(),
    //     }
    // };

    // // --------------------------------------------------------------------
    // // Annotations

    // let output_path = args.output_dir.join("annotations.tsv");
    // info!("Creating annotations: {output_path:?}");

    // let annotations = match args.name {
    //     Name::SarsCov2 => sarscov2::annotations::build()?,
    //     Name::Toy1 => toy1::annotations::build()?,
    //     _ => todo!(),
    // };
    // annotations.write(&output_path)?;

    // // --------------------------------------------------------------------
    // // Graph (Phylogeny)

    // let output_path = args.output_dir.join("phylogeny.json");
    // info!("Building phylogeny: {output_path:?}");

    // let phylogeny = match args.name {
    //     Name::SarsCov2 => sarscov2::phylogeny::build(&mut summary, &args.output_dir).await?,
    //     Name::Toy1 => toy1::phylogeny::build()?,
    //     _ => todo!(),
    // };
    // phylogeny.write(&output_path)?;
    // // Also write as .dot file for graphviz visualization.
    // let output_path = args.output_dir.join("phylogeny.dot");
    // info!("Exporting graphviz phylogeny: {output_path:?}");
    // phylogeny.write(&output_path)?;

    // // --------------------------------------------------------------------
    // // Export Mutations

    // let output_path = args.output_dir.join("mutations.json");
    // info!("Mapping mutations to populations: {output_path:?}");
    // let mask = vec![0, 0];
    // let (_populations, mutations) = dataset::load::parse_populations(
    //     &summary.populations.local_path,
    //     &summary.reference.local_path,
    //     &mask,
    // )?;

    // // --------------------------------------------------------------------
    // // Create Edge Cases
    // //
    // // Edge cases are simply a vector of the CLI Run Args (cli::run::Args)
    // // customized to particular recombinants.

    // let output_path = args.output_dir.join("edge_cases.json");
    // info!("Creating edge cases: {output_path:?}");

    // let mut edge_cases = match args.name {
    //     Name::SarsCov2 => dataset::sarscov2::edge_cases::default()?,
    //     Name::Toy1 => dataset::toy1::edge_cases::default()?,
    //     _ => todo!(),
    // };
    // let manual_populations =
    //     edge_cases.clone().into_iter().filter_map(|e| e.population).collect_vec();

    // phylogeny.get_problematic_recombinants()?.into_iter().try_for_each(|r| {
    //     //let recombinant = r.to_string();
    //     let parents = phylogeny.get_parents(r)?;
    //     warn!("Recombinant {r} is problematic. Parents are not sister taxa: {parents:?}");
    //     if manual_populations.contains(&r.to_string()) {
    //         warn!("Manual edge case exists: {r:?}");
    //     } else {
    //         warn!("Creating auto edge case: {r:?}");
    //         let population = Some(r.to_string());
    //         let parents = Some(parents.to_vec().into_iter().map(String::from).collect());
    //         let edge_case = cli::run::Args {
    //             population,
    //             parents,
    //             ..Default::default()
    //         };
    //         edge_cases.push(edge_case);
    //     }

    //     Ok::<(), Report>(())
    // });

    // // --------------------------------------------------------------------
    // // Export

    // let dataset = load::dataset(&args.output_dir, &mask)?;

    // let path = args.output_dir.join("edge_cases.json");
    // info!("Exporting edge cases: {path:?}");
    // dataset.write_edge_cases(&path)?;

    // let path = args.output_dir.join("mutations.tsv");
    // info!("Exporting mutations: {path:?}");
    // dataset.write_mutations(&path)?;

    // let path = args.output_dir.join("summary.json");
    // info!("Exporting summary: {path:?}");
    // dataset.write_summary(&path)?;

    // --------------------------------------------------------------------
    // Finish

    info!("Done.");
    Ok(dataset)
}

/// Download remote file from a summary snapshot.
pub async fn snapshot(snapshot: &RemoteFile, output_path: &Path) -> Result<RemoteFile, Report> {
    // Check extension for decompression
    let ext = utils::path_to_ext(Path::new(&snapshot.url))?;
    let decompress = ext == "zst";

    // Update the local path to the desired output
    let mut remote_file = snapshot.clone();
    remote_file.local_path = output_path.to_path_buf();

    // Download the file
    utils::download_file(&snapshot.url, output_path, decompress).await?;

    Ok(remote_file)
}
