#![doc = embed_doc_image::embed_image!("assets/images/XBB_BJ.1_CJ.1_22897-22941.png", "assets/images/XBB_BJ.1_CJ.1_22897-22941.png")]
//! `rebar` detects **RE**combination between genomics sequences using mutational **BAR**codes.
//!
//! ## Why rebar?
//!
//! 1. `rebar` _detects_ and _visualizes_ genomic recombination.
//!
//!    It follows the [PHA4GE Guidance for Detecting and Characterizing SARS-CoV-2 Recombinants](https://github.com/pha4ge/pipeline-resources/blob/main/docs/sc2-recombinants.md)
//!    which outlines three steps:
//!
//!     1. Assess the genomic evidence for recombination.
//!     1. Identify the breakpoint coordinates and parental regions.
//!     1. Classify sequences as _designated_ or _novel_ recombinant lineages.
//!
//! 1. `rebar` peforms generalized _clade assignment_.
//!
//!    While specifically designed for recombinants, `rebar` works on non-recombinants tool!
//!    It will report a sequence's closest known match in the dataset, as well any mutation
//!    conflicts that were observed. The linelist and visual outputs can be used to detect
//!    novel variants, such as the SARS-CoV-2
//!    [pango-designation](https://github.com/cov-lineages/pango-designation/issues) process.
//!
//! 1. **`rebar` is for _exploring hypotheses_.**
//!
//!     The recombination search can be customized to test your hypotheses about which parents
//!     and genomic regions are recombining. If that sounds overwhelming, you can always just
//!     use the pre-configured datasets (ex. SARS-CoV-2) that are validated against known
//!     recombinants.
//!
//!    ![A plot of the breakpoints and parental regions for the recombinant SARS-CoV-2 lineage XBB.1.16. At the top are rectangles arranged side-by-side horizontally. These are colored and labelled by each parent (ex. BJ.1., CJ.1) and are intepreted as reading left to right, 5' to 3'. Below these regions are genomic annotations, which show the coordinates for each gene. At the bottom are horizontal tracks, where each row is a sample, and each column is a mutation. Mutations are colored according to which parent the recombination region derives from.][assets/images/XBB_BJ.1_CJ.1_22897-22941.png]

pub mod cli;
pub mod dataset;
mod phylogeny;
pub mod run;
pub mod table;
mod utils;

// pub use crate::dataset::Dataset;
#[doc(inline)]
pub use crate::cli::Cli;
#[doc(inline)]
pub use crate::dataset::Dataset;
#[doc(inline)]
pub use crate::phylogeny::Phylogeny;
#[doc(inline)]
pub use table::Table;
#[doc(inline)]
pub use utils::verbosity::Verbosity;
// pub use utils::verbosity::Verbosity;
// pub use utils::table;
// pub use utils::table::Table;
