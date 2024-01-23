//! Download an available dataset.

use crate::dataset::{is_compatible, toy1, Attributes, Name, Tag, VersionedFile};
use crate::utils;
use crate::Dataset;

#[cfg(feature = "cli")]
use clap::Parser;
use color_eyre::eyre::{eyre, Report, Result};
use log::{info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

/// Download dataset arguments.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct DownloadArgs {
    /// [`Dataset`] [`Name`].
    #[cfg_attr(feature = "cli", clap(short = 'r', long, required = true))]
    pub name: Name,

    /// [`Dataset`] version [`Tag`].
    ///
    /// A date (YYYY-MM-DD), 'latest', or 'custom'
    #[cfg_attr(feature = "cli", clap(short = 't', long, required = true))]
    pub tag: Tag,

    /// Output directory.
    ///
    /// If the directory does not exist, it will be created.
    #[cfg_attr(feature = "cli", clap(short = 'o', long, required = true))]
    pub output_dir: PathBuf,

    /// Download [`Dataset`] from an [`Attributes`] JSON [`snapshot`].
    #[cfg_attr(feature = "cli", clap(short = 's', long, required = false))]
    #[cfg_attr(feature = "cli", clap(help = "Download dataset from a Summary JSON."))]
    pub attributes: Option<PathBuf>,
}

impl Default for DownloadArgs {
    fn default() -> Self {
        DownloadArgs::new()
    }
}
impl DownloadArgs {
    pub fn new() -> Self {
        DownloadArgs {
            name: Name::default(),
            tag: Tag::default(),
            output_dir: PathBuf::new(),
            attributes: None,
        }
    }
}

/// Download a [`Dataset`].
///
/// ## Arguments
///
/// - `args` - [`DownloadArgs`] to use for downloading available datasets.
///
/// ## Examples
///
/// ```rust
/// use rebar::dataset::*;
/// use std::path::PathBuf;
/// # use tokio_test::{assert_ok, block_on};
///
/// let args = DownloadArgs {name: Name::Toy1, tag: Tag::Custom, output_dir: PathBuf::from("test/dataset/toy1"), attributes: None };
/// # assert_ok!(block_on(async {
/// let dataset = download(&args).await?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
///
/// ```rust
/// // todo!() make this with sarscov2
/// // create an attributes JSON
/// # use rebar::dataset::*;
/// # use std::path::PathBuf;
/// # use tokio_test::{assert_ok, block_on};
/// use chrono::NaiveDate;
/// let attributes = Attributes { name: Name::Toy1, tag: Tag::Custom, .. Default::default() };
/// let output_dir = PathBuf::from("test/dataset/toy1");
/// let attributes_path = output_dir.join("attributes.json");
/// attributes.write(&attributes_path)?;
/// let attributes = Some(attributes_path);
///
/// // download dataset from attributes JSON
/// let args = DownloadArgs {name: Name::Toy1, tag: Tag::Custom, output_dir, attributes };
/// # assert_ok!(block_on(async {
/// let dataset = download(&args).await?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
#[cfg(feature = "download")]
pub async fn download(args: &DownloadArgs) -> Result<Dataset, Report> {
    info!("Downloading dataset: {} {}", &args.name, &args.tag);

    let dataset = Dataset::new();

    // --------------------------------------------------------------------
    // Optional Input Attributes

    let mut attributes = match &args.attributes {
        Some(path) => {
            info!("Importing Attributes: {path:?}");
            let attributes = Attributes::read(path)?;
            // Warn if attributes conflict with any CLI args
            if attributes.name != args.name || attributes.tag != args.tag {
                warn!(
                    "Dataset has been changed by Attributes to: {} {}",
                    &attributes.name, &attributes.tag
                );
            }
            attributes
        }
        _ => Attributes { name: args.name, tag: args.tag.clone(), ..Default::default() },
    };

    // --------------------------------------------------------------------
    // Compatibility Check

    if !is_compatible(Some(&attributes.name), Some(&attributes.tag))? {
        Err(eyre!("Dataset incompatibility"))?;
    }

    // Warn if the directory already exists
    if !args.output_dir.exists() {
        info!("Creating output directory: {:?}", &args.output_dir);
        std::fs::create_dir_all(&args.output_dir)?;
    } else {
        warn!("Proceed with caution! --output-dir {:?} already exists.", args.output_dir);
    }

    // --------------------------------------------------------------------
    // Reference

    info!("Downloading reference.");

    // Select downloading from versioned file or internal function
    let reference =
        if attributes.reference.is_some() && attributes.reference.clone().unwrap().url.is_some() {
            download_versioned_file(attributes.reference.unwrap(), &args.output_dir).await?
        } else {
            match args.name {
                Name::Toy1 => toy1::reference(&attributes.tag, &args.output_dir)?,
                _ => todo!(),
            }
        };
    attributes.reference = Some(reference);

    // --------------------------------------------------------------------
    // Populations

    info!("Downloading populations.");

    // Select downloading from versioned file or internal function
    let populations = if attributes.populations.is_some()
        && attributes.populations.clone().unwrap().url.is_some()
    {
        download_versioned_file(attributes.populations.unwrap(), &args.output_dir).await?
    } else {
        match args.name {
            Name::Toy1 => toy1::populations(&attributes.tag, &args.output_dir)?,
            _ => todo!(),
        }
    };
    attributes.populations = Some(populations);

    // --------------------------------------------------------------------
    // Annotations

    info!("Downloading annotations.");

    // Select downloading from versioned file or internal function
    let annotations = if attributes.annotations.is_some()
        && attributes.annotations.clone().unwrap().url.is_some()
    {
        download_versioned_file(attributes.annotations.unwrap(), &args.output_dir).await?
    } else {
        match args.name {
            Name::Toy1 => toy1::annotations(&attributes.tag, &args.output_dir)?,
            _ => todo!(),
        }
    };
    attributes.annotations = Some(annotations);

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

/// Download from a [`VersionedFile`].
///
/// ## Arguments
///
/// ## Examples
///
/// Without decompression.
///
/// ```rust
/// # use tokio_test::{block_on, assert_ok};
/// use rebar::dataset::{download_versioned_file, VersionedFile};
///
/// let url = "https://raw.githubusercontent.com/nextstrain/ncov/v13/data/references_sequences.fasta";
/// let local = "reference.fasta";
/// let input_file = VersionedFile {url: Some(url.into()), local: local.into(), .. Default::default()};
///
/// let output_dir = "test/dataset/download_versioned_file";
/// # assert_ok!(block_on(async {
/// let output_file = download_versioned_file(input_file, &output_dir).await?;
/// # assert!(std::path::PathBuf::from(output_dir).join(local).exists());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
///
/// With decompression.
///
/// ```rust
/// # use tokio_test::{block_on, assert_ok};
/// # use rebar::dataset::{download_versioned_file, VersionedFile};
/// use rebar::utils::Decompress;
///
/// let url = "https://raw.githubusercontent.com/corneliusroemer/pango-sequences/a8596a6/data/pango-consensus-sequences_genome-nuc.fasta.zst";
/// let local = "populations.fasta";
/// let decompress = Decompress::Zst;
/// let input_file = VersionedFile {url: Some(url.into()), local: local.into(), decompress: Some(decompress), .. Default::default()};
///
/// let output_dir = "test/dataset/download_versioned_file";
/// # assert_ok!(block_on(async {
/// let output_file = download_versioned_file(input_file, &output_dir).await?;
/// # assert!(std::path::PathBuf::from(output_dir).join(local).exists());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
#[cfg(feature = "download")]
pub async fn download_versioned_file<P>(
    file: VersionedFile,
    output_dir: &P,
) -> Result<VersionedFile, Report>
where
    P: AsRef<Path> + Debug,
{
    // create output dir if needed
    std::fs::create_dir_all(output_dir)?;
    let output = output_dir.as_ref().to_owned().join(&file.local);

    // make sure a URL exists
    let url = match &file.url {
        Some(url) => url.as_str(),
        None => Err(eyre!("Failed to download versioned file, URL is missing: {file:?}"))?,
    };

    // decompress if requested
    if let Some(decompress) = &file.decompress {
        // download to temp file before decompressing
        let tmp_path = output.with_extension(decompress.to_string());
        utils::download_file(url, &tmp_path).await?;
        // decompress to temporary file
        let tmp_path = utils::decompress_file(&tmp_path, file.decompress.clone())?;
        std::fs::rename(tmp_path, &output)?;
    } else {
        utils::download_file(url, &output).await?;
    };

    Ok(file)
}
