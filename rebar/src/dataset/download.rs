//! Download an available dataset.

use crate::dataset::{is_compatible, Name, Tag};
use crate::Dataset;

#[cfg(feature = "cli")]
use clap::Parser;
use color_eyre::eyre::{eyre, Report, Result};
use log::{info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fmt::Debug;
use std::path::PathBuf;

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
    #[cfg_attr(feature = "cli", clap(short = 's', long, required = true))]
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
/// let output = "test/utils/download_file/reference.fasta";
/// let dataset: Dataset<chrono::NaiveDate, &str> = download(&args).await?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub async fn download<D, P>(args: &DownloadArgs) -> Result<Dataset<D, P>, Report> {
    info!("Downloading dataset: {} {}", &args.name, &args.tag);

    let dataset = Dataset::new();

    // --------------------------------------------------------------------
    // Optional Input Attributes

    // let mut attributes: Attributes<chrono::NaiveDate, PathBuf> = match cfg!(serde) {
    //     true  => match &args.attributes {
    //         #[cfg(serde = true)]
    //         Some(path) => Attributes::read(path)?,
    //         _ => Attributes { name: args.name, tag: args.tag.clone(), .. Default::default() },
    //     },
    //     false => Attributes { name: args.name, tag: args.tag.clone(), .. Default::default() },
    // };

    // if let Some(path) = &args.attributes {
    //     info!("Importing Attributes: {path:?}");
    //     let attributes = Attributes::read(path)?;

    //     // Warn if attributes conflict with any CLI args
    //     if attributes.name != args.name || attributes.tag != args.tag {
    //         warn!("Dataset has been changed by Attributes to: {} {}", &attributes.name, &attributes.tag);
    //     }
    //     attributes
    // } else {
    //     Attributes { name: args.name, tag: args.tag.clone(), .. Default::default() }
    // };

    // --------------------------------------------------------------------
    // Compatibility Check

    if !is_compatible(Some(&args.name), Some(&args.tag))? {
        Err(eyre!("Dataset incompatibility"))?;
    }

    // Warn if the directory already exists
    if !args.output_dir.exists() {
        info!("Creating output directory: {:?}", &args.output_dir);
        std::fs::create_dir_all(&args.output_dir)?;
    } else {
        warn!("Proceed with caution! --output-dir {:?} already exists.", args.output_dir);
    }

    // // --------------------------------------------------------------------
    // // Reference

    // let output_path = args.output_dir.join("reference.fasta");
    // info!("Downloading reference: {output_path:?}");

    // attributes.reference = if args.attributes.is_some() {
    //     match attributes.reference {
    //         //Some(remote_file) => snapshot(&remote_file, &output_path).await.ok(),
    //         //todo!()
    //         Some(remote_file) => None,
    //         None => None,
    //     }
    // } else {
    //     match args.name {
    //         //Name::SarsCov2 => sarscov2::download::reference(&args.tag, &output_path).await?,
    //         Name::Toy1 => toy1::download::reference(&args.tag, &output_path).ok(),
    //         _ => todo!(),
    //     }
    // };

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

// ----------------------------------------------------------------------------
// Remote File
// ----------------------------------------------------------------------------

/// A file downloaded from a remote URL.
///
/// ## Generics
///
/// - `D` - Date, recommended [`chrono::NaiveDate`].
/// - `P` - File path.
///
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RemoteFile<D, P> {
    /// File URL
    pub url: P,
    // Github commit SHA hash
    pub sha: String,
    // Local path of the file.
    pub local_path: P,
    // Date the file was created.
    pub date_created: D,
    // Date the file was downloaded.
    pub date_downloaded: D,
}

impl<D, P> Default for RemoteFile<D, P>
where
    D: Default,
    P: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<D, P> RemoteFile<D, P>
where
    D: Default,
    P: Default,
{
    pub fn new() -> Self {
        RemoteFile {
            url: P::default(),
            sha: String::new(),
            local_path: P::default(),
            date_created: D::default(),
            date_downloaded: D::default(),
        }
    }
}

// /// Download from a [`RemoteFile`].
// ///
// /// ## Arguments
// ///
// /// ## Examples
// ///
// /// ```rust
// /// # use tokio_test::{block_on, assert_ok};
// /// use rebar::dataset::{download_remote_file, RemoteFile};
// /// let remote_file = RemoteFile {url:  .. Default::default()};
// /// # assert_ok!(block_on(async {
// /// download_remote_file()
// /// # Ok::<(), color_eyre::eyre::Report>(())
// /// # }));
// /// # Ok::<(), color_eyre::eyre::Report>(())
// /// ```
// pub async fn download_remote_file<D, P>(
//     remote_file: &RemoteFile<D, P>,
//     output_path: &P,
// ) -> Result<(), Report>
// where
//     P: AsRef<std::path::Path> + Debug,
// {
//     // Check extension for decompression
//     //let ext = utils::get_extension(&remote_file.url)?;
//     //let decompress = ext == "zst";

//     // Update the local path to the desired output
//     //let mut remote_file = remote_file.clone();
//     //remote_file.local_path = output_path.into();

//     // utils::download_file(&snapshot.url, output_path, decompress).await?;

//     // Ok(remote_file)
//     Ok(())
// }
