//! List available datasets for download.

use crate::dataset::{Compatibility, Name};
use clap::Parser;
use color_eyre::eyre::{Report, Result};
use comfy_table::Table;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

// ----------------------------------------------------------------------------
// Structs

/// Arguments for list datasets.
#[derive(Debug, Parser)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[clap(verbatim_doc_comment)]
pub struct Args {
    /// Dataset name.
    #[clap(short = 'n', long)]
    pub name: Option<Name>,
}

impl Default for Args {
    fn default() -> Self {
        Args::new()
    }
}
impl Args {
    pub fn new() -> Self {
        Args { name: None }
    }
}

// ----------------------------------------------------------------------------
// Functions

/// List datasets available for download.
pub fn list(args: &Args) -> Result<Table, Report> {
    // table of name, tag, cli_version
    let mut table = Table::new();
    table.set_header(vec!["Name", "CLI Version", "Minimum Tag Date", "Maximum Tag Date"]);

    for name in Name::iter() {
        // Check if this was not the name requested by CLI args
        if let Some(args_name) = &args.name {
            if &name != args_name {
                continue;
            }
        }

        // Extract compatibility attributes
        let compatibility: Compatibility<chrono::NaiveDate> = name.get_compatibility()?;

        let cli_version = compatibility.cli_version.unwrap_or_default();
        let min_date = match compatibility.min_date {
            Some(date) => date.to_string(),
            None => "nightly".to_string(),
        };
        let max_date = match compatibility.max_date {
            Some(date) => date.to_string(),
            None => "nightly".to_string(),
        };

        // Add to row
        let row = vec![name.to_string(), cli_version.to_string(), min_date, max_date];
        table.add_row(row);
    }

    println!("{table}");

    Ok(table)
}
