use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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

impl std::fmt::Display for Verbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Convert to lowercase for RUST_LOG env var compatibility
        let lowercase = format!("{:?}", self).to_lowercase();
        write!(f, "{lowercase}")
    }
}
