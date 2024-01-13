//! [Command-line interface](Cli) (CLI) of the main binary.

pub mod dataset;
// pub mod plot;
// pub mod run;
// pub mod simulate;

use crate::Verbosity;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::default::Default;
use structdoc::StructDoc;

// ----------------------------------------------------------------------------
// CLI Entry Point
// ----------------------------------------------------------------------------

/// The command-line interface (CLI).
/// ---
/// The CLI is intended for parsing user input from the command-line in the main function. This is achieved with the `parse` function, which parses the command line arguments from [`std::env::args`](https://doc.rust-lang.org/std/env/fn.args.html).
/// ```no_run
/// use clap::Parser;
/// let args = rebar::Cli::parse();
/// ```
/// The command-line arguments from `std::env::args` are simply a vector of space separated strings. Here is a manual example of setting the command-line input:
/// ```rust
/// # use clap::Parser;
/// let input = ["rebar", "dataset", "download", "--name", "toy1", "--tag", "custom", "--output-dir", "dataset/toy1"];
/// let args = rebar::Cli::parse_from(input);
/// use serde_json;
/// serde_json::to_string_pretty(&args)?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
/// With the following pretty JSON representation:
/// ```json
/// {
///     "command": {
///       "Dataset": {
///         "command": {
///           "Download": {
///             "name": "toy1",
///             "tag": "Custom",
///             "output_dir": "output/toy1",
///             "summary": null
///           }
///         }
///       }
///     },
///     "verbosity": "Info"
///   }
/// ```
#[derive(Debug, Deserialize, Parser, Serialize)]
#[clap(name = "rebar", author, version)]
#[clap(about = "rebar detects recombination between genomics sequences using mutational barcodes.")]
#[clap(after_help = "Long help message")]
#[clap(trailing_var_arg = true)]
pub struct Cli {
    #[clap(subcommand)]
    /// Pass CLI arguments to a particular [Command].
    #[clap(help = "Set the command.")]
    pub command: Command,

    /// Set the output [Verbosity] level.
    #[clap(short = 'v', long)]
    #[clap(value_enum, default_value_t = Verbosity::default())]
    #[clap(hide_possible_values = false)]
    #[clap(global = true)]
    #[clap(help = "Set the output verbosity level.")]
    pub verbosity: Verbosity,
}

/// CLI [commands](#variants). Used to decide which runtime [Command](#variants) the CLI arguments should be passed to.
///
#[derive(Debug, Deserialize, Serialize, StructDoc, Subcommand)]
pub enum Command {
    /// Pass CLI arguments to the [Dataset](dataset::Command) subcommands.
    /// <br>
    /// ```rust
    /// use rebar::{cli::Command, cli::dataset, Cli};
    /// use clap::Parser;
    /// let input = ["rebar", "dataset", "--help"];
    /// let args = Cli::parse_from(input);
    /// match args.command {
    ///   Command::Dataset(dataset_args) => dataset_args.command {
    ///     cli::dataset::Download(dataset_args) => (),
    ///     cli::dataset::List(dataset_args)     => (),
    ///   }
    /// }
    /// ```
    #[clap(about = "List or download available datasets.")]
    Dataset(dataset::Args),
    //Run(run::Args),
    //Plot(plot::Args),
    //Simulate(simulate::Args),
}
