//! Downloading, loading, and manipulating of the [Dataset].

pub mod attributes;
pub mod download;
//pub mod list;
pub mod toy1;

#[doc(inline)]
pub use crate::dataset::attributes::{Compatibility, Name, Summary, Tag};
#[doc(inline)]
pub use crate::dataset::download::download;
#[doc(inline)]
pub use crate::dataset::list::list;

use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
// Dataset

/// A collection of parent population sequences aligned to a reference.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Dataset {
    /// Summary of dataset attributes.
    pub summary: attributes::Summary,
    // /// Reference sequence record, with sequence bases kept
    // pub reference: sequence::Record,
    // /// Dataset populations, map of names to sequences.
    // pub populations: BTreeMap<String, sequence::Record>,
    // /// Dataset mutations, map of substitutions to named sequences.
    // pub mutations: BTreeMap<sequence::Substitution, Vec<String>>,
    // /// Phylogenetic representation, as an ancestral recombination graph (ARG)
    // pub phylogeny: Phylogeny<String>,
    // /// Edge cases of problematic populations
    // pub edge_cases: Vec<run::Args>,
}

impl std::fmt::Display for Dataset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "name: {}, tag: {}", self.summary.name, self.summary.tag)
    }
}

impl Default for Dataset {
    fn default() -> Self {
        Self::new()
    }
}

impl Dataset {
    /// Create a new dataset.
    pub fn new() -> Self {
        Dataset {
            summary: attributes::Summary::new(),
            // reference: sequence::Record::new(),
            // populations: BTreeMap::new(),
            // mutations: BTreeMap::new(),
            // phylogeny: Phylogeny::new(),
            // edge_cases: Vec::new(),
        }
    }
}
