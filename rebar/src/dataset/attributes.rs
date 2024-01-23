//! Metadata to uniquely identify a dataset ([Name], [Tag]) and faciliate reproducibility ([Summary]).

use crate::utils;

use chrono::{DateTime, Local, Utc};
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use color_eyre::Help;
use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::path::{Path, PathBuf};
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
/// let attributes = Attributes { name: Name::SarsCov2, tag:  Tag::Latest, .. Default::default()};
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Attributes {
    /// Dataset [Name].
    pub name: Name,
    /// CLI semantic version used to create the dataset (ex. "0.3.0").
    pub version: String,
    /// Dataset version [Tag].
    pub tag: Tag,
    /// Optional reference genome file.
    pub reference: Option<VersionedFile>,
    /// Optional population alignment file.
    pub populations: Option<VersionedFile>,
    /// Optional genome annotations.
    pub annotations: Option<VersionedFile>,
    /// Additional files that are dataset-specific.
    ///
    /// For example, in the [SARS-CoV-2](Name::SarsCov2) dataset, the [alias key](https://github.com/cov-lineages/pango-designation/blob/master/pango_designation/alias_key.json) maps aliases to lineage names.
    pub misc: BTreeMap<String, Option<VersionedFile>>,
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}
impl Attributes {
    /// Returns new [`Attributes`] with empty or default values.
    ///
    /// ```rust
    /// use rebar::dataset::Attributes;
    /// use chrono::NaiveDate;
    ///
    /// let attributes = Attributes::new();
    /// ```
    pub fn new() -> Self {
        Attributes {
            version: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            tag: Tag::default(),
            name: Name::default(),
            reference: None,
            populations: None,
            annotations: None,
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
    /// let attr_out = Attributes::default();
    /// let file     = tempfile::NamedTempFile::new()?;
    /// let path     = file.path();
    /// attr_out.write(&path)?;
    ///
    /// let attr_in = Attributes::read(&path)?;
    /// # assert_eq!(attr_in, attr_out);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[cfg(feature = "serde")]
    pub fn read<P>(path: &P) -> Result<Attributes, Report>
    where
        P: AsRef<Path> + Debug,
    {
        let file = std::fs::File::open(path)
            .wrap_err(eyre!("Failed to open Attributes file: {path:?}."))?;
        let reader = std::io::BufReader::new(file);
        let attributes = serde_json::from_reader(reader)
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
    /// let attributes = Attributes::default();
    /// let file       = tempfile::NamedTempFile::new()?;
    /// let path       = file.path();
    ///
    /// attributes.write(&path)?;
    /// # assert!(attributes.write(&"/root").is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[cfg(feature = "serde")]
    pub fn write<P>(&self, path: &P) -> Result<(), Report>
    where
        P: AsRef<Path> + Debug,
    {
        utils::create_parent_dir(path)?;

        // write file
        let mut file = std::fs::File::create(path)
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

impl std::fmt::Display for Name {
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
    /// use rebar::dataset::Tag;
    /// let tag = Tag::Latest;
    /// ```
    Latest,
    /// For a [`Dataset`] that has at least one file that is version-controlled or date-controlled.
    /// For example, source files downloaded from GitHub repositories.
    ///
    /// The String is a UTC date in one of two formats.
    ///
    /// Date: "yyyy-mm-dd".
    ///
    /// ```rust
    /// # use rebar::dataset::Tag;
    /// use std::str::FromStr;
    /// let tag = Tag::from_str("2024-01-01")?;
    /// assert_eq!(tag, Tag::Archive("2024-01-01T23:59:59Z".to_string()));
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// Datetime in [ISO_8601](https://en.wikipedia.org/wiki/ISO_8601) or [RFC 3339](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339) format: "yyyy-mm-ddTHH:mm:ssZ"
    ///
    /// ```rust
    /// # use rebar::dataset::Tag;
    /// # use std::str::FromStr;
    /// let tag = Tag::from_str("2024-01-01T12:05:32Z")?;
    /// assert_eq!(tag, Tag::Archive("2024-01-01T12:05:32Z".to_string()));
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    Archive(String),
    /// For all other [`Dataset`], that are custom created with no options to date-control.
    ///
    /// ```rust
    /// # use rebar::dataset::Tag;
    /// let tag = Tag::Custom;
    /// ```
    #[default]
    Custom,
}

impl std::fmt::Display for Tag {
    /// Test
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tag = match self {
            Tag::Latest => String::from("latest"),
            Tag::Archive(tag) => tag.to_owned(),
            Tag::Custom => String::from("custom"),
        };

        write!(f, "{}", tag)
    }
}

impl FromStr for Tag {
    type Err = Report;

    /// Returns a [`Dataset`] [`Tag`] converted from a [`str`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar::dataset::Tag;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Tag::from_str("latest")?, Tag::Latest);
    /// assert_eq!(Tag::from_str("custom")?, Tag::Custom);
    /// assert_eq!(Tag::from_str("2024-01-01")?, Tag::Archive("2024-01-01T23:59:59Z".to_string()));
    /// assert_eq!(Tag::from_str("2024-01-02T12:00:00Z")?, Tag::Archive("2024-01-02T12:00:00Z".to_string()));
    /// assert!(Tag::from_str("9999-01-02").is_err());
    /// assert!(Tag::from_str("unknown").is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    fn from_str(tag: &str) -> Result<Tag, Report> {
        let tag = match tag {
            "latest" => Tag::Latest,
            "custom" => Tag::Custom,
            _ => {
                let (tag_date, tag): (DateTime<Utc>, String) = match tag.parse() {
                    Ok(date) => (date, tag.to_string()),
                    Err(_) => {
                        let tag_rfc_3339 = format!("{tag}T23:59:59Z");
                        let tag_date = tag_rfc_3339.parse().wrap_err(
                        eyre!("Archive tag date is invalid: {tag:?}")
                        .suggestion("Example of a valid Archive tag: 2023-08-17 or 2023-08-17T12:00:00Z")
                        )?;
                        (tag_date, tag_rfc_3339)
                    }
                };
                // is it in the future?
                let today: DateTime<Utc> = Local::now().into();
                if tag_date > today {
                    return Err(eyre!("Archive tag date is in the future: {tag:?}. Please pick a date on or before today: {today:?}"))?;
                }
                Tag::Archive(tag)
            }
        };

        Ok(tag)
    }
}

// ----------------------------------------------------------------------------
// Dataset Compatibility

/// Returns the [`Compatibility`] of a named [`Dataset`].
///
///  ## Examples
///
/// ```
/// use rebar::dataset::{get_compatibility, Name};
/// use chrono::NaiveDate;
///
/// get_compatibility(&Name::SarsCov2)?;
/// get_compatibility(&Name::Toy1)?;
/// get_compatibility(&Name::Custom)?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn get_compatibility(name: &Name) -> Result<Compatibility, Report> {
    let mut compatibility = Compatibility::new();
    #[allow(clippy::single_match)]
    match name {
        Name::SarsCov2 => {
            compatibility.min_date =
                Some(DateTime::parse_from_rfc3339("2023-02-09T00:00:00Z")?.into())
        }
        Name::Toy1 => compatibility.cli_version = Some(">=0.2.0".to_string()),
        //Name::Custom => compatibility.cli_version = Some(">=0.3.0".to_string()),
        _ => compatibility.cli_version = None,
    }
    Ok(compatibility)
}

/// Returns true if the [`Dataset`] [`Name`] and [`Tag`] are compatible with each other, and the CLI version.
///
/// ## Examples
///
/// ```
/// use rebar::dataset::{is_compatible, Name, Tag};
/// use std::str::FromStr;
///
/// assert_eq!(true,  is_compatible(Some(&Name::SarsCov2), Some(&Tag::Latest))?);
/// assert_eq!(true,  is_compatible(Some(&Name::SarsCov2), Some(&Tag::from_str("2023-06-06")?))?);
/// assert_eq!(false, is_compatible(Some(&Name::SarsCov2), Some(&Tag::from_str("2023-02-08")?))?);
/// assert_eq!(true,  is_compatible(Some(&Name::Toy1),     None)?);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn is_compatible(name: Option<&Name>, tag: Option<&Tag>) -> Result<bool, Report> {
    let compatibility = match name {
        Some(name) => get_compatibility(name)?,
        None => Compatibility::default(),
    };

    // Check CLI Version
    if let Some(cli_version) = compatibility.cli_version {
        let current_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
        let required_version = semver::VersionReq::parse(&cli_version)?;
        if !required_version.matches(&current_version) {
            warn!(
                "CLI version incompatibility.
                Current version {current_version} does not satisfy the required version {required_version}",
                current_version=current_version.to_string()
                );
            return Ok(false);
        }
    }
    // Check Tag
    match tag {
        Some(Tag::Latest) => {
            if let Some(max_date) = compatibility.max_date {
                warn!(
                    "Date incompatibility.
                Tag {tag:?} does not satisfy the maximum date {max_date:?}"
                );
                return Ok(false);
            }
        }
        Some(Tag::Archive(s)) => {
            let tag_date = DateTime::parse_from_rfc3339(s)?;
            // tag date is too early
            if let Some(min_date) = compatibility.min_date {
                if tag_date < min_date {
                    warn!(
                        "Date incompatibility.
                        Tag {tag_date:?} does not satisfy the minimum date {min_date:?}"
                    );
                    return Ok(false);
                }
            }
            // tag date is too late
            if let Some(max_date) = compatibility.max_date {
                if tag_date > max_date {
                    warn!(
                        "Date incompatibility.
                    Tag {tag_date:?} does not satisfy the maximum date {max_date:?}"
                    );
                    return Ok(false);
                }
            }
        }
        _ => (),
    }

    Ok(true)
}

/// A summary of how compatibile a [`Dataset`] is with a [`CLI`] version and date.
///
///  ## Examples
///
/// ```rust
/// use rebar::dataset::Compatibility;
/// use chrono::{DateTime, Utc};
///
/// let min_date = Some("2024-01-01T00:00:00Z".parse::<DateTime<Utc>>()?);
/// let max_date = None;
/// let cli_version = Some(">=1.0.0".to_string());
///
/// let c = Compatibility { min_date, max_date, cli_version};
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Compatibility {
    /// The minimum date for the dataset.
    pub min_date: Option<DateTime<Utc>>,
    /// The maximum date for the dataset.
    pub max_date: Option<DateTime<Utc>>,
    /// The CLI semantic version constraint.
    pub cli_version: Option<String>,
}

impl Default for Compatibility {
    fn default() -> Self {
        Self::new()
    }
}

impl Compatibility {
    /// Returns a new [`Compatibility`] with no dates or CLI constraints.
    ///
    ///  ## Examples
    ///
    /// ```
    /// use rebar::dataset::Compatibility;
    /// use chrono::NaiveDate;
    ///
    /// let c = Compatibility::new();
    /// assert_eq!(c, Compatibility { min_date: None, max_date: None, cli_version: None});
    /// ```
    pub fn new() -> Self {
        Compatibility { min_date: None, max_date: None, cli_version: None }
    }
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
pub struct VersionedFile {
    // Local name of the file
    pub local: PathBuf,
    /// File URL
    pub url: Option<String>,
    // Date the file was created.
    pub date_created: Option<DateTime<Utc>>,
    // Date the file was downloaded.
    pub date_downloaded: Option<DateTime<Utc>>,
    // Optional Decompression
    pub decompress: Option<utils::Decompress>,
}

impl Default for VersionedFile {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionedFile {
    pub fn new() -> Self {
        VersionedFile {
            local: PathBuf::default(),
            url: None,
            date_created: None,
            date_downloaded: None,
            decompress: None,
        }
    }
}
