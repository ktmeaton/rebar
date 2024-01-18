pub mod dataset;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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
