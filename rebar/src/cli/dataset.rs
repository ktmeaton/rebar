// //! Command-line interface (CLI) for Dataset [Commands](Command).

use crate::dataset::list;
use clap::{Parser, Subcommand};
// use rebar_dataset::{download, list};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// CLI arguments to list or download available datasets.
#[derive(Debug, Parser)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[clap(about = "List or download available datasets.")]
pub struct Args {
    /// Dataset command: List, Download
    #[clap(subcommand)]
    pub command: Command,
}

/// CLI dataset [commands](#variants). Used to decide which dataset method the CLI arguments should be passed to.
#[derive(Debug, Subcommand)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Command {
    /// Pass CLI arguments to the dataset [list](crate::dataset::list()) method.
    /// <br>
    /// ```rust
    /// use rebar::cli::dataset::Command;
    /// use rebar::dataset::{download, list};
    /// let args    = list::Args::default();
    /// let command = Command::List( list::Args::default() );
    /// match command {
    ///   Command::List(args)     => _ = list(&args),
    ///   Command::Download(args) => _ = download(&args),
    /// }
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[clap(about = "List datasets.")]
    List(list::Args),
    // /// Pass CLI arguments to the dataset [download](crate::dataset::download()) method.
    // /// <br>
    // /// ```rust
    // /// use rebar::cli::dataset::Command;
    // /// use rebar::dataset::{download, list};
    // /// let args    = download::Args::default();
    // /// let command = Command::Download(args);
    // /// match command {
    // ///   Command::List(args)     => _ = list(&args),
    // ///   Command::Download(args) => _ = download(&args),
    // /// }
    // /// # Ok::<(), color_eyre::eyre::Report>(())
    // /// ```
    // #[clap(about = "Download dataset.")]
    // #[clap(arg_required_else_help = true)]
    // Download(download::Args),
}
