//! Downloading, loading, and manipulating of the [Dataset].

mod attributes;
pub mod toy1;

#[doc(inline)]
pub use attributes::*;

use crate::sequence;

#[cfg(feature = "cli")]
use clap::Parser;
use color_eyre::eyre::{eyre, ContextCompat, Report, Result, WrapErr};
use log::{info, warn};
use rebar_phylo::{FromJson, Phylogeny};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;
use tabled::Table;

// ----------------------------------------------------------------------------
// Dataset

/// A collection of parent population sequences aligned to a reference.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Dataset {
    /// [`Dataset`] [`Attributes`].
    pub attributes: Attributes,
    /// Reference sequence record, with sequence bases kept
    pub reference: sequence::Record,
    /// Dataset populations, map of names to sequences.
    pub populations: BTreeMap<String, sequence::Record>,
    // /// Dataset mutations, map of substitutions to named sequences.
    // pub mutations: BTreeMap<sequence::Substitution, Vec<String>>,
    /// Phylogenetic representation, as an ancestral recombination graph (ARG)
    pub phylogeny: Phylogeny<String, usize>,
    // /// Edge cases of problematic populations
    // pub edge_cases: Vec<run::Args>,
}

impl Display for Dataset {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "name: {}, tag: {}", self.attributes.name, self.attributes.tag)
    }
}

impl Dataset {
    /// Create a new dataset.
    pub fn new() -> Self {
        Dataset {
            attributes: Attributes::default(),
            reference: sequence::Record::default(),
            populations: BTreeMap::new(),
            // mutations: BTreeMap::new(),
            phylogeny: rebar_phylo::Phylogeny::new(),
            // edge_cases: Vec::new(),
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
    /// let dataset = Dataset::download(&args).await?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// # }));
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[cfg(feature = "download")]
    pub async fn download(args: &DownloadArgs) -> Result<(), Report> {
        info!("Downloading dataset: {} {}", &args.name, &args.tag);

        // --------------------------------------------------------------------
        // Read Optional Input Attributes

        let mut attributes = match &args.attributes {
            Some(path) => {
                info!("Importing Attributes: {path:?}");
                let mut attributes = Attributes::read(path)?;
                // Warn if attributes conflict with any CLI args
                if attributes.name != args.name || attributes.tag != args.tag {
                    warn!(
                        "Dataset has been changed by Attributes to: {} {}",
                        &attributes.name, &attributes.tag
                    );
                }
                // update cli version, just in case
                attributes.version = Attributes::default().version;
                attributes
            }
            _ => Attributes { name: args.name, tag: args.tag.clone(), ..Default::default() },
        };

        // --------------------------------------------------------------------
        // Compatibility Check

        if !Compatibility::is_compatible(Some(&attributes.name), Some(&attributes.tag))? {
            Err(eyre!("Dataset incompatibility"))?;
        }

        // Warn if the directory already exists
        if !args.output_dir.exists() {
            info!("Creating output directory: {:?}", &args.output_dir);
            std::fs::create_dir_all(&args.output_dir)?;
        } else {
            warn!("Proceed with caution! --output-dir {:?} already exists.", args.output_dir);
        }

        info!("Downloading reference.");
        attributes.reference = match attributes.reference.url {
            Some(_) => attributes.reference.download(&args.output_dir).await?,
            None => match args.name {
                Name::Toy1 => toy1::reference(&attributes.tag, &args.output_dir)?,
                _ => todo!(),
            },
        };

        info!("Downloading populations.");
        attributes.populations = match attributes.populations.url {
            Some(_) => attributes.populations.download(&args.output_dir).await?,
            None => match args.name {
                Name::Toy1 => toy1::populations(&attributes.tag, &args.output_dir)?,
                _ => todo!(),
            },
        };

        info!("Downloading annotations.");
        attributes.annotations = if attributes.annotations.is_some()
            && attributes.annotations.clone().unwrap().url.is_some()
        {
            let versioned_file = attributes.annotations.unwrap();
            let file = versioned_file.download(&args.output_dir).await?;
            Some(file)
        } else {
            match args.name {
                Name::Toy1 => Some(toy1::annotations(&attributes.tag, &args.output_dir)?),
                _ => None,
            }
        };

        // mutations

        info!("Downloading phylogeny.");
        attributes.phylogeny = if attributes.phylogeny.is_some()
            && attributes.phylogeny.clone().unwrap().url.is_some()
        {
            let versioned_file = attributes.phylogeny.unwrap();
            Some(versioned_file.download(&args.output_dir).await?)
        } else {
            match args.name {
                Name::Toy1 => Some(toy1::phylogeny(&attributes.tag, &args.output_dir)?),
                _ => None,
            }
        };

        // Export and write
        let path = args.output_dir.join("attributes.json");
        info!("Writing attributes: {path:?}");
        attributes.write(&path)?;

        info!("Done.");

        Ok(())
    }

    /// Returns a [`Table`] of datasets available for download.
    ///
    /// ## Arguments
    ///
    /// - `args` - [`ListArgs`] to use for listing available datasets.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar::dataset::*;
    /// use std::str::FromStr;
    ///
    /// let table = Dataset::list(&ListArgs::default())?;
    /// let table = Dataset::list(&ListArgs { name: Some(Name::SarsCov2), tag: None })?;
    /// let table = Dataset::list(&ListArgs { name: None,                 tag: Some(Tag::Latest) })?;
    /// let table = Dataset::list(&ListArgs { name: None,                 tag: Some(Tag::from_str("2023-01-01")?) })?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn list(args: &ListArgs) -> Result<Table, Report> {
        // table of name, tag, cli_version
        let mut builder = tabled::builder::Builder::default();
        builder.push_record(vec!["Name", "CLI Version", "Minimum Tag Date", "Maximum Tag Date"]);

        // Check all named datasets
        Name::iter()
            // check args name
            .filter(|name| match args.name {
                Some(args_name) => args_name == *name,
                None => true,
            })
            // check compatibility
            .filter(|name| {
                Compatibility::is_compatible(Some(name), args.tag.as_ref()).unwrap_or(false)
            })
            .try_for_each(|name| {
                let c = Compatibility::get_compatibility(&name)?;

                let cli_version = match c.cli_version {
                    Some(v) => v.to_string(),
                    None => "".to_string(),
                };

                let min_date = match c.min_date {
                    Some(date) => date.to_string(),
                    None => "".to_string(),
                };
                let max_date = match c.max_date {
                    Some(date) => date.to_string(),
                    None => "".to_string(),
                };

                let row = vec![name.to_string(), cli_version, min_date, max_date];
                builder.push_record(row);

                Ok::<(), Report>(())
            })?;

        let table = builder.build();
        Ok(table)
    }

    /// Returns a [`Dataset`] read from files in a directory.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar::dataset::*;
    /// use std::path::PathBuf;
    /// # use tokio_test::{assert_ok, block_on};
    ///
    /// # assert_ok!(block_on(async {
    /// let args = DownloadArgs {name: Name::Toy1, tag: Tag::Custom, output_dir: PathBuf::from("test/dataset/toy1"), attributes: None };
    /// Dataset::download(&args).await?;
    /// let dataset = Dataset::read(&"test/dataset/toy1")?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// # }));
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn read<P>(dataset_dir: &P) -> Result<Dataset, Report>
    where
        P: AsRef<Path> + Debug,
    {
        // convert from generics to path
        let dataset_dir = dataset_dir.as_ref();
        info!("Reading dataset: {dataset_dir:?}");

        let mut dataset = Dataset::new();

        let path = dataset_dir.join("attributes.json");
        info!("Reading attributes: {path:?}");
        dataset.attributes = Attributes::read(&path)?;

        // Reference
        let path = dataset_dir.join(&dataset.attributes.reference.local);
        info!("Reading reference: {path:?}");
        let (mut reader, _count) = sequence::get_reader(&path, false)?;
        // read file into a noodles::fasta object
        let record = reader
            .records()
            .next()
            .wrap_err(format!("Failed to read first fasta record: {path:?}"))?
            .wrap_err(format!("Failed to parse first fasta record: {path:?}"))?;
        // convert from noodles fasta to rebar
        dataset.reference =
            sequence::Record::from_noodles(record, None, dataset.attributes.alphabet.clone())?;

        // Populations
        let path = dataset_dir.join(&dataset.attributes.populations.local);
        info!("Reading populations: {path:?}");
        let (mut reader, _count) = sequence::get_reader(&path, true)?;

        dataset.populations = reader
            .records()
            .map(|result| {
                let record = match result {
                    Ok(record) => sequence::Record::from_noodles(
                        record,
                        Some(&dataset.reference),
                        dataset.attributes.alphabet.clone(),
                    )?,
                    Err(_) => Err(eyre!("Failed to parse populations record: {result:?}"))?,
                };
                Ok((record.id.clone(), record))
            })
            .collect::<Result<_, Report>>()?;

        // (Optional) Phylogeny
        dataset.phylogeny = match &dataset.attributes.phylogeny {
            Some(versioned_file) => {
                let path = dataset_dir.join(&versioned_file.local);
                info!("Reading phylogeny: {path:?}");
                let input = std::fs::read_to_string(&path)
                    .wrap_err(format!("Failed to read phylogeny: {path:?}."))?;
                Phylogeny::from_json(&input)?
            }
            None => Phylogeny::new(),
        };

        // (Optional) Edge Cases

        Ok(dataset)
    }
}

// ----------------------------------------------------------------------------
// Download Args
// ----------------------------------------------------------------------------

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

// ----------------------------------------------------------------------------
// List Args
// ----------------------------------------------------------------------------

/// Arguments for listing datasets available for download.
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ListArgs {
    /// [`Dataset`] [`Name`].
    #[cfg_attr(feature = "cli", clap(short = 'n', long))]
    pub name: Option<Name>,
    /// [`Dataset`] [`Tag`].
    #[cfg_attr(feature = "cli", clap(short = 't', long))]
    pub tag: Option<Tag>,
}

impl Default for ListArgs {
    fn default() -> Self {
        ListArgs::new()
    }
}
impl ListArgs {
    pub fn new() -> Self {
        ListArgs { name: None, tag: None }
    }
}
