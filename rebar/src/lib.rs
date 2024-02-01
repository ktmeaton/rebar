#![doc = include_str!("../../README.md")]

#[cfg(feature = "cli")]
pub mod cli;
pub mod dataset;
pub mod run;
pub mod sequence;
// pub mod table;
pub mod utils;

// // pub use crate::dataset::Dataset;
#[doc(inline)]
#[cfg(feature = "cli")]
pub use crate::cli::Cli;
#[doc(inline)]
pub use crate::dataset::Dataset;

#[doc(inline)]
pub use crate::run::{run, RunArgs};
// #[doc(inline)]
// pub use table::Table;
// #[doc(inline)]
// pub use utils::verbosity::Verbosity;
// pub use utils::verbosity::Verbosity;
// pub use utils::table;
// pub use utils::table::Table;