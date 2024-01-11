use crate::dataset::{download, list};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use structdoc::StructDoc;

/// CLI arguments to list or download available datasets.
#[derive(Debug, Deserialize, Parser, Serialize, StructDoc)]
#[clap(about = "List or download available datasets.")]
pub struct Args {
    /// Dataset command: List, Download
    #[clap(subcommand)]
    pub command: Command,
}

/// CLI dataset [commands](#variants). Used to decide which dataset method the CLI arguments should be passed to.
#[derive(Debug, Deserialize, Serialize, StructDoc, Subcommand)]
pub enum Command {
    /// Pass CLI arguments to the dataset [list](crate::dataset::list()) method.
    ///
    /// ```rust
    /// use rebar::cli::dataset::Command;
    /// use rebar::dataset::{download, list};
    ///
    /// let args    = list::Args::default();
    /// let command = Command::List(args);
    ///
    /// match command {
    ///   Command::List(args)     => _ = list(&args),
    ///   Command::Download(args) => _ = download(&args),
    /// }
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[clap(about = "List datasets.")]
    List(list::Args),

    /// Pass CLI arguments to the dataset [download](crate::dataset::download()) method.
    ///
    /// ```rust
    /// use rebar::cli::dataset::Command;
    /// use rebar::dataset::{download, list};
    ///
    /// let args    = download::Args::default();
    /// let command = Command::Download(args);
    ///
    /// match command {
    ///   Command::List(args)     => _ = list(&args),
    ///   Command::Download(args) => _ = download(&args),
    /// }
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[clap(about = "Download dataset.")]
    Download(download::Args),
}
