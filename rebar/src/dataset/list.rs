//! List available datasets for download.

use crate::dataset::{get_compatibility, is_compatible, Name, Tag};
#[cfg(feature = "cli")]
use clap::Parser;
use color_eyre::eyre::{Report, Result};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tabled::Table;

// ----------------------------------------------------------------------------
// Structs

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

// ----------------------------------------------------------------------------
// Functions

/// Returns a [`Table`] of datasets available for download.
///
/// ## Arguments
///
/// - `args` - [`ListArgs`] to use for listing available datasets.
///
/// ## Examples
///
/// ```rust
/// use rebar::dataset::{list, ListArgs, Name, Tag};
/// use std::str::FromStr;
///
/// let table = list(&ListArgs::default())?;
/// let table = list(&ListArgs { name: Some(Name::SarsCov2), tag: None })?;
/// let table = list(&ListArgs { name: None,                 tag: Some(Tag::Latest) })?;
/// let table = list(&ListArgs { name: None,                 tag: Some(Tag::from_str("2023-01-01")?) })?;
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
        .filter(|name| is_compatible(Some(name), args.tag.as_ref()).unwrap_or(false))
        .try_for_each(|name| {
            let c = get_compatibility(&name)?;

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
