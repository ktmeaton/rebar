//! Downloading, loading, and manipulating of the [Dataset].

mod attributes;
mod download;
mod list;
//pub mod list;
// pub mod toy1;

#[doc(inline)]
pub use attributes::{get_compatibility, is_compatible, Attributes, Compatibility, Name, Tag};
#[doc(inline)]
pub use download::{download, DownloadArgs, RemoteFile};
#[doc(inline)]
pub use list::{list, ListArgs};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

// // ----------------------------------------------------------------------------
// // Dataset

/// A collection of parent population sequences aligned to a reference.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Dataset<D, P> {
    /// [`Dataset`] [`Attributes`].
    pub attributes: Attributes<D, P>,
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

impl<D, P> Display for Dataset<D, P> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "name: {}, tag: {}", self.attributes.name, self.attributes.tag)
    }
}

impl<D, P> Default for Dataset<D, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D, P> Dataset<D, P> {
    /// Create a new dataset.
    pub fn new() -> Self {
        Dataset {
            attributes: Attributes::default(),
            // reference: sequence::Record::new(),
            // populations: BTreeMap::new(),
            // mutations: BTreeMap::new(),
            // phylogeny: Phylogeny::new(),
            // edge_cases: Vec::new(),
        }
    }
}
