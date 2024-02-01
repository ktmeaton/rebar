// //! Command-line interface (CLI) for Dataset [Commands](Command).

use crate::dataset::{DownloadArgs, ListArgs};
use clap::{Parser, Subcommand};
// use rebar_dataset::{download, list};
use serde::{Deserialize, Serialize};

/// CLI arguments to list or download available datasets.
#[derive(Debug, Deserialize, Parser, Serialize)]
#[clap(about = "List or download available datasets.")]
pub struct Args {
    /// Dataset command: List, Download
    #[clap(subcommand)]
    pub command: Command,
}

/// CLI dataset [commands](#variants). Used to decide which dataset method the CLI arguments should be passed to.
#[derive(Debug, Deserialize, Serialize, Subcommand)]
pub enum Command {
    // ------------------------------------------------------------------------
    /// Pass CLI arguments to the dataset [list](crate::dataset::list()) method.
    /// ## Examples
    /// ```rust
    /// use rebar::{Cli, cli, cli::Command, cli::dataset};
    /// use clap::Parser;
    /// let input   = ["rebar", "dataset", "list", "--help"];
    /// let command = Cli::parse_from(input).command;
    /// match command {
    ///   Command::Dataset(args) => assert!(matches!(args.command, dataset::Command::List(_))),
    ///   _                      => assert!(false),
    /// }
    /// ```
    #[clap(about = "List datasets.")]
    List(ListArgs),
    // ------------------------------------------------------------------------
    /// Pass CLI arguments to the dataset [download](crate::dataset::download()) method.
    /// ## Examples
    /// ```rust
    /// use rebar::{Cli, cli, cli::Command, cli::dataset};
    /// use clap::Parser;
    /// let input   = ["rebar", "dataset", "download", "--help"];
    /// let command = Cli::parse_from(input).command;
    /// match command {
    ///   Command::Dataset(args) => assert!(matches!(args.command, dataset::Command::Download(_))),
    ///   _                      => assert!(false),
    /// }
    /// ```
    #[clap(about = "Download dataset.")]
    #[clap(arg_required_else_help = true)]
    Download(DownloadArgs),
}
