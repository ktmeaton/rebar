#![doc = include_str!("../README.md")]
use color_eyre::eyre::{Report, Result};
// mod branch;
mod branch;
pub mod examples;
pub mod newick;
mod node;
mod phylogeny;

#[doc(inline)]
pub use branch::Branch;
#[doc(inline)]
pub use examples::*;
#[doc(inline)]
pub use node::Node;
#[doc(inline)]
pub use phylogeny::Phylogeny;

// ----------------------------------------------------------------------------
// Traits
// ----------------------------------------------------------------------------

/// Returns an object created from a JSON [`str`].
pub trait FromJson {
    fn from_json(json: &str) -> Result<Self, Report>
    where
        Self: Sized;
}

/// Returns an object created from a [Mermaid](https://mermaid.js.org/) [`str`].
pub trait FromMermaid {
    fn from_mermaid(mermaid: &str) -> Result<Self, Report>
    where
        Self: Sized;
}

/// Returns an object created from a [Newick](https://en.wikipedia.org/wiki/Newick_format) [`str`].
pub trait FromNewick {
    fn from_newick(newick: &str) -> Result<Self, Report>
    where
        Self: Sized;
}

/// Returns a [DOT](https://graphviz.org/doc/info/lang.html) [`str`] created from an object.
pub trait ToDot {
    fn to_dot(&self) -> Result<String, Report>;
}

/// Returns a JSON [`str`] created from an object.
pub trait ToJson {
    fn to_json(&self) -> Result<String, Report>;
}

/// Returns a [Mermaid](https://mermaid.js.org/) [`str`] created from an object.
pub trait ToMermaid {
    fn to_mermaid(&self) -> Result<String, Report>;
}

/// Returns a [Newick](https://en.wikipedia.org/wiki/Newick_format) [`str`] created from an object.
pub trait ToNewick {
    fn to_newick(&self) -> Result<String, Report>;
}
