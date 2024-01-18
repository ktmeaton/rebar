//! Metadata to uniquely identify a dataset ([Name], [Tag]) and faciliate reproducibility ([Summary]).

use crate::dataset::RemoteFile;

use chrono::{Local, NaiveDate};
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use color_eyre::Help;
use log::warn;
use semver::{Version, VersionReq};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::str::FromStr;
use strum::EnumIter;

// ----------------------------------------------------------------------------
// Dataset Attributes
// ----------------------------------------------------------------------------

/// [`Attributes`] of a [`Dataset`] and its source files.
///
/// ## Generics
///
/// - `D` - Date, recommended [`chrono::NaiveDate`].
/// - `P` - File path.
///
/// ## Examples
///
/// ```rust
/// use rebar::dataset::{Attributes, Name, Tag};
/// use chrono::NaiveDate;
///
/// let attributes: Attributes<NaiveDate, &str> = Attributes { name: Name::SarsCov2, tag:  Tag::Nightly, .. Default::default()};
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Attributes<D, P> {
    /// Dataset [Name].
    pub name: Name,
    /// CLI semantic version used to create the dataset (ex. "0.3.0").
    pub version: String,
    /// Dataset version [Tag].
    pub tag: Tag,
    /// Optional URL of the reference genome file.
    pub reference: Option<RemoteFile<D, P>>,
    /// Optional URL of the population alignment file.
    pub populations: Option<RemoteFile<D, P>>,
    /// Additional files that are dataset-specific.
    ///
    /// For example, in the [SARS-CoV-2](Name::SarsCov2) dataset, the [alias key](https://github.com/cov-lineages/pango-designation/blob/master/pango_designation/alias_key.json) maps aliases to lineage names.
    pub misc: BTreeMap<String, Option<RemoteFile<D, P>>>,
}

impl<D, P> Default for Attributes<D, P> {
    fn default() -> Self {
        Self::new()
    }
}
impl<D, P> Attributes<D, P> {
    /// Returns new [`Attributes`] with empty or default values.
    ///
    /// ```rust
    /// use rebar::dataset::Attributes;
    /// use chrono::NaiveDate;
    ///
    /// let attributes = Attributes::<NaiveDate, &str>::new();
    /// ```
    pub fn new() -> Self {
        Attributes {
            version: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            tag: Tag::default(),
            name: Name::default(),
            reference: None,
            populations: None,
            misc: BTreeMap::new(),
        }
    }

    /// Read [`Dataset`] [`Attributes`] from a JSON file.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar::dataset::Attributes;
    /// use chrono::NaiveDate;
    ///
    /// let attr_out = Attributes::<NaiveDate, String>::new();
    /// let file    = tempfile::NamedTempFile::new()?;
    /// attr_out.write(file.path())?;
    ///
    /// let attr_in = Attributes::<NaiveDate, String>::read(file.path())?;
    /// # assert_eq!(attr_in, attr_out);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[cfg(feature = "serde")]
    pub fn read<R>(path: R) -> Result<Attributes<D, P>, Report>
    where
        D: for<'de> Deserialize<'de>,
        P: Debug + for<'de> Deserialize<'de>,
        R: AsRef<std::path::Path> + Debug,
    {
        let file = std::fs::File::open(&path)
            .wrap_err(eyre!("Failed to open Attributes file: {path:?}."))?;
        let reader = std::io::BufReader::new(file);
        let attributes: Attributes<D, P> = serde_json::from_reader(reader)
            .wrap_err(eyre!("Failed to deserialize Attributes file: {path:?}."))?;
        Ok(attributes)
    }

    /// Write [`Dataset`] [`Attributes`] to a JSON file.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar::dataset::Attributes;
    /// use chrono::NaiveDate;
    ///
    /// let attributes = Attributes::<NaiveDate, &str>::new();
    /// let file       = tempfile::NamedTempFile::new()?;
    ///
    /// attributes.write(file.path())?;
    /// # assert!(attributes.write("/root").is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[cfg(feature = "serde")]
    pub fn write<W>(&self, path: W) -> Result<(), Report>
    where
        D: Debug + Serialize,
        P: Debug + Serialize,
        W: AsRef<std::path::Path> + Debug,
    {
        let mut file = std::fs::File::create(&path)
            .wrap_err(eyre!("Failed to create Attributes file: {path:?}"))?;
        let output = serde_json::to_string_pretty(self)
            .wrap_err(eyre!("Failed to serialize Attributes: {self:?}"))?;
        file.write_all(format!("{}\n", output).as_bytes())
            .wrap_err(eyre!("Failed to write Attributes file: {path:?}"))?;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Dataset Name
// ----------------------------------------------------------------------------

/// The [`Name`] of a [`Dataset`].
///
/// Might represent a particular organism (ex. sars-cov-2) or simulated data for testing (toy1).
#[derive(Clone, Copy, Debug, Default, EnumIter, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Name {
    /// Severe Acute Respiratory Syndrome Coronavirus 2 (SARS-CoV-2)
    ///
    /// ```
    /// let name = rebar::dataset::Name::SarsCov2;
    /// ```
    ///
    #[cfg_attr(feature = "serde", serde(rename = "sars-cov-2"))]
    SarsCov2,
    /// Toy dataset 1 for testing.
    /// ```
    /// let name = rebar::dataset::Name::Toy1;
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "toy1"))]
    Toy1,
    /// Custom dataset
    /// ```
    /// let name = rebar::dataset::Name::Custom;
    /// ```
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "custom"))]
    Custom,
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Name::SarsCov2 => String::from("sars-cov-2"),
            Name::Toy1 => String::from("toy1"),
            Name::Custom => String::from("custom"),
        };

        write!(f, "{}", name)
    }
}

impl FromStr for Name {
    type Err = Report;

    /// Returns a dataset [`Name`] converted from a [`str`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar::dataset::Name;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Name::SarsCov2, Name::from_str("sars-cov-2")?);
    /// assert_eq!(Name::Toy1,     Name::from_str("toy1")?);
    /// assert_eq!(Name::Custom,   Name::from_str("custom")?);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    fn from_str(name: &str) -> Result<Self, Report> {
        let name = match name {
            "sars-cov-2" => Name::SarsCov2,
            "custom" => Name::Custom,
            "toy1" => Name::Toy1,
            _ => Err(eyre!("Unknown dataset name: {name}")).suggestion("Please choose from:")?,
        };

        Ok(name)
    }
}

impl Name {
    /// Returns the [`Compatibility`] of a named [`Dataset`].
    ///
    ///  ## Examples
    ///
    /// ```
    /// use rebar::dataset::{Compatibility, Name};
    /// use chrono::NaiveDate;
    ///
    /// Name::SarsCov2.get_compatibility::<NaiveDate>()?;
    /// Name::Toy1.get_compatibility::<NaiveDate>()?;
    /// Name::Custom.get_compatibility::<NaiveDate>()?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_compatibility<D>(&self) -> Result<Compatibility<D>, Report>
    where
        D: std::convert::From<NaiveDate>,
    {
        let mut compatibility: Compatibility<D> = Compatibility::new();
        #[allow(clippy::single_match)]
        match self {
            Name::SarsCov2 => {
                compatibility.min_date =
                    Some(NaiveDate::parse_from_str("2023-02-09", "%Y-%m-%d")?.into());
            }
            Name::Toy1 => compatibility.cli_version = Some(">=0.2.0".to_string()),
            _ => compatibility.cli_version = Some(">=1.0.0".to_string()),
        }
        Ok(compatibility)
    }
}

// ----------------------------------------------------------------------------
// Dataset Tag
// ----------------------------------------------------------------------------

/// The version [`Tag`] of a [`Dataset`].
///
/// Typically identifies the date when source files were downloaded.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Tag {
    /// For a [`Dataset`] where files were downloaded from the latest possible available.
    ///
    /// ```rust
    /// let tag = rebar::dataset::Tag::Nightly;
    /// ```
    Nightly,
    /// For a [`Dataset`] that has at least one file that is version-controlled or date-controlled.
    /// For example, source files downloaded from GitHub repositories.
    ///
    /// The String is a date in the format "yyyy-mm-dd", such as "2024-01-01".
    ///
    /// ```rust
    /// let date = "2024-01-01".to_string();
    /// let tag = rebar::dataset::Tag::Archive(date);
    /// ```
    Archive(String),
    /// For all other [`Dataset`], that are custom created with no options to date-control.
    ///
    /// ```rust
    /// let tag = rebar::dataset::Tag::Custom;;
    /// ```
    #[default]
    Custom,
}

impl std::fmt::Display for Tag {
    /// Test
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tag = match self {
            Tag::Nightly => String::from("nightly"),
            Tag::Archive(tag) => tag.to_owned(),
            Tag::Custom => String::from("custom"),
        };

        write!(f, "{}", tag)
    }
}

impl FromStr for Tag {
    type Err = Report;

    /// Returns a [`Dataset`] [`Tag`] converted from a [`str`].
    fn from_str(tag: &str) -> Result<Tag, Report> {
        let tag = match tag {
            "nightly" => Tag::Nightly,
            "custom" => Tag::Custom,
            _ => {
                // check if it's an archival date string
                let tag_date = NaiveDate::parse_from_str(tag, "%Y-%m-%d")
                    .wrap_err_with(|| eyre!("Archive tag date is invalid: {tag:?}. Example of a valid Archive tag: 2023-08-17"))?;
                // is it in the future?
                let today = Local::now().date_naive();
                if tag_date > today {
                    return Err(eyre!("Archive tag date is in the future: {tag:?}. Please pick a date on or before today: {today:?}"))?;
                }
                Tag::Archive(tag.to_string())
            }
        };

        Ok(tag)
    }
}

// ----------------------------------------------------------------------------
// Dataset Compatibility

/// Returns true if the [`Dataset`] [`Name`] and [`Tag`] are compatible with each other, and the CLI version.
///
/// ## Examples
///
/// ```
/// use rebar::dataset::{is_compatible, Name, Tag};
/// use std::str::FromStr;
///
/// assert_eq!(true,  is_compatible(&Name::SarsCov2, &Tag::Nightly)?);
/// assert_eq!(true,  is_compatible(&Name::SarsCov2, &Tag::from_str("2023-06-06")?)?);
/// assert_eq!(false, is_compatible(&Name::SarsCov2, &Tag::from_str("2023-01-01")?)?);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn is_compatible(name: &Name, tag: &Tag) -> Result<bool, Report> {
    let compatibility = name.get_compatibility()?;

    // Check CLI Version
    if let Some(cli_version) = compatibility.cli_version {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let required_version = VersionReq::parse(&cli_version)?;
        if !required_version.matches(&current_version) {
            warn!(
                "CLI version incompatibility.
                Current version {current_version} does not satisfy the {name} dataset requirement {required_version}",
                current_version=current_version.to_string()
                );
            return Ok(false);
        }
    }
    // Check Tag Dates
    if matches!(tag, Tag::Archive(_)) {
        let tag_date = NaiveDate::parse_from_str(&tag.to_string(), "%Y-%m-%d")?;

        // Minimum Date
        if let Some(min_date) = compatibility.min_date {
            if tag_date < min_date {
                warn!(
                    "Date incompatibility.
                    Tag {tag_date:?} does not satisfy the {name} dataset minimum date {min_date:?}"
                );
                return Ok(false);
            }
        }
        // Maximum Date
        if let Some(max_date) = compatibility.max_date {
            if tag_date > max_date {
                warn!(
                    "Date incompatibility.
                    Tag {tag_date:?} does not satisfy the {name} dataset maximum date {max_date:?}"
                );
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// A summary of how compatibile a [`Dataset`] is with a [`CLI`] version and date.
///
///  ## Examples
///
/// ```rust
/// use rebar::dataset::Compatibility;
/// use chrono::NaiveDate;
///
/// let min_date = Some(NaiveDate::parse_from_str("2023-02-09", "%Y-%m-%d")?);
/// let max_date = None;
/// let cli_version = Some(">=1.0.0".to_string());
///
/// let c = Compatibility { min_date, max_date, cli_version};
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Compatibility<D> {
    /// The minimum date for the dataset.
    pub min_date: Option<D>,
    /// The maximum date for the dataset.
    pub max_date: Option<D>,
    /// The CLI semantic version constraint.
    pub cli_version: Option<String>,
}

impl<D> Default for Compatibility<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D> Compatibility<D> {
    /// Returns a new [`Compatibility`] with no dates or CLI constraints.
    ///
    ///  ## Examples
    ///
    /// ```
    /// use rebar::dataset::Compatibility;
    /// use chrono::NaiveDate;
    ///
    /// let c: Compatibility<NaiveDate> = Compatibility::new();
    /// assert_eq!(c, Compatibility { min_date: None, max_date: None, cli_version: None});
    /// ```
    pub fn new() -> Self {
        Compatibility { min_date: None, max_date: None, cli_version: None }
    }
}
