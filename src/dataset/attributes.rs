use crate::utils::remote_file::RemoteFile;
use chrono::prelude::*;
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use color_eyre::Help;
use log::warn;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::default::Default;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use structdoc::StructDoc;
use strum::{EnumIter, EnumProperty};

// ----------------------------------------------------------------------------
// Dataset Name

/// The name of a dataset.
///
/// Might represent a particular organism (ex. sars-cov-2) or simulated data for testing (toy1).
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumIter,
    EnumProperty,
    PartialEq,
    Serialize,
    StructDoc,
)]
pub enum Name {
    /// Severe Acute Respiratory Syndrome Coronavirus 2
    #[serde(rename = "sars-cov-2")]
    #[strum(props(implemented = "true"))]
    SarsCov2,
    /// Test dataset 1
    #[serde(rename = "toy1")]
    #[strum(props(implemented = "true"))]
    Toy1,
    /// Custom dataset
    #[default]
    #[serde(rename = "custom")]
    #[strum(props(implemented = "false"))]
    Custom,
}

impl Name {
    /// Get the compatibility for a named dataset.
    pub fn get_compatibility(&self) -> Result<Compatibility, Report> {
        let mut compatibility = Compatibility::new();
        #[allow(clippy::single_match)]
        match self {
            Name::SarsCov2 => {
                compatibility.min_date = Some(NaiveDate::parse_from_str("2023-02-09", "%Y-%m-%d")?);
            }
            Name::Toy1 => compatibility.cli_version = Some(">=0.2.0".to_string()),
            _ => compatibility.cli_version = Some(">=1.0.0".to_string()),
        }
        Ok(compatibility)
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    /// Convert a string to a dataset Name.
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

// ----------------------------------------------------------------------------
// Dataset Tag

/// The version tag of a dataset.
///
/// Typically identifies the date when source files were downloaded.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, StructDoc)]
pub enum Tag {
    /// For datasets where the source files were downloaded from the latest possible available.
    Nightly,
    /// For datasets that have at least one file that is version-controlled or date-controlled.
    /// For example, source files downloaded from GitHub repositories.
    ///
    /// The String is a date in the format "yyyy-mm-dd", such as "2024-01-01".
    Archive(String),
    /// For all other datasets, that are created custom with no option to date-control.
    #[default]
    Custom,
}

impl std::fmt::Display for Tag {
    /// Test
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    /// Convert a string to a dataset tag.
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

/// Return true if dataset name and tag are compatible.
///
/// # Examples
///
/// ```
/// use rebar::dataset::attributes::{is_compatible, Name, Tag};
///
/// let name = Name::SarsCov2;
/// let tag = Tag::Nightly;
/// assert_eq!(true, is_compatible(&name, &tag).unwrap());
///
/// // The GitHub repo of SARS-CoV-2 sequences was only created in 2023-02.
/// // So dates after this are compatible, but before are not.
/// let tag = Tag::Archive("2023-06-06".to_string());
/// assert_eq!(true, is_compatible(&name, &tag).unwrap());
///
/// let tag = Tag::Archive("2023-01-01".to_string());
/// assert_eq!(false, is_compatible(&name, &tag).unwrap());
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

/// A summary of a dataset's compatibility with a CLI version and date.
///
/// # Examples
///
/// ```
/// use rebar::dataset::attributes::Compatibility;
/// use chrono::NaiveDate;
///
/// let min_date = Some(NaiveDate::parse_from_str("2023-02-09", "%Y-%m-%d").unwrap());
/// let max_date = None;
/// let cli_version = Some(">=1.0.0".to_string());
///
/// let c = Compatibility { min_date, max_date, cli_version};
/// ```
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Compatibility {
    /// The minimum date for the dataset.
    pub min_date: Option<NaiveDate>,
    /// The maximum date for the dataset.
    pub max_date: Option<NaiveDate>,
    /// The CLI semantic version constraint.
    pub cli_version: Option<String>,
}

impl Default for Compatibility {
    fn default() -> Self {
        Self::new()
    }
}

impl Compatibility {
    /// Create new Compatibility with no date or CLI constraints.
    /// # Examples
    ///
    /// ```
    /// use rebar::dataset::attributes::Compatibility;
    ///
    /// let c = Compatibility::new();
    /// assert_eq!(c, Compatibility { min_date: None, max_date: None, cli_version: None});
    /// ```
    pub fn new() -> Self {
        Compatibility {
            min_date: None,
            max_date: None,
            cli_version: None,
        }
    }
}

// ----------------------------------------------------------------------------
// Dataset Summary

/// A summary of a dataset's attributes and source files.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Summary {
    /// CLI version used to create the dataset.
    pub version: String,
    /// Dataset version tag.
    pub tag: Tag,
    /// Dataset name.
    pub name: Name,
    /// URL of the reference genome file.
    pub reference: Option<RemoteFile>,
    /// URL of the population alignment file.
    pub populations: Option<RemoteFile>,
    /// Additional files that are dataset-specific.
    ///
    /// For example, the sars-cov-2 alias key that maps aliases to lineage names.
    pub misc: BTreeMap<String, Option<RemoteFile>>,
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}
impl Summary {
    /// Create a new dataset summary.
    pub fn new() -> Self {
        Summary {
            version: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            tag: Tag::default(),
            name: Name::default(),
            reference: None,
            populations: None,
            misc: BTreeMap::new(),
        }
    }
    /// Read dataset summary from JSON file.
    pub fn read(path: &Path) -> Result<Summary, Report> {
        let summary = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read file: {path:?}."))?;
        let summary = serde_json::from_str(&summary)
            .wrap_err_with(|| format!("Failed to parse file: {path:?}"))?;

        Ok(summary)
    }

    /// Write dataset summary to JSON file.
    pub fn write(&self, path: &Path) -> Result<(), Report> {
        // create output file
        let mut file =
            File::create(path).wrap_err_with(|| format!("Failed to create file: {path:?}"))?;

        // parse to string
        let output = serde_json::to_string_pretty(self)
            .wrap_err_with(|| format!("Failed to parse: {self:?}"))?;

        // write to file
        file.write_all(format!("{}\n", output).as_bytes())
            .wrap_err_with(|| format!("Failed to write file: {path:?}"))?;

        Ok(())
    }
}
