// //! [Command-line interface](Cli) (CLI) of the main binary.

pub mod dataset;
// // pub mod plot;
// // pub mod run;
// // pub mod simulate;

use crate::RunArgs;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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
#[derive(Debug, Deserialize, Serialize, Subcommand)]
pub enum Command {
    /// Pass CLI arguments to the [Dataset](dataset::Command) subcommands.
    /// ## Examples
    /// ```rust
    /// use rebar::{Cli, cli::Command};
    /// use clap::Parser;
    /// let input = ["rebar", "dataset", "--help"];
    /// let args = Cli::parse_from(input);
    /// matches!(args.command, Command::Dataset(_));
    /// ```
    #[clap(about = "List or download available datasets.")]
    Dataset(dataset::Args),
    #[clap(about = "Run recombination detection.")]
    Run(RunArgs),
    //Plot(plot::Args),
    //Simulate(simulate::Args),
}

// -----------------------------------------------------------------------------
// Verbosity
// -----------------------------------------------------------------------------

/// The output verbosity level.
#[derive(Clone, Debug, Default, Deserialize, Serialize, ValueEnum)]
pub enum Verbosity {
    #[default]
    Info,
    Warn,
    Debug,
    Error,
}

impl Display for Verbosity {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        // Convert to lowercase for RUST_LOG env var compatibility
        let lowercase = format!("{:?}", self).to_lowercase();
        write!(f, "{lowercase}")
    }
}
