use crate::FromNewick;
use color_eyre::eyre::{eyre, Report, Result};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

/// A [`Node`] in the [`Phylogeny`](crate::Phylogeny) graph.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Node<N> {
    /// [`Node`] label for display.
    pub label: N,
}

#[rustfmt::skip]
impl<N> Default for Node<N> where N: Default { fn default() -> Self { Self::new() } }
#[rustfmt::skip]
impl<N> Display for Node<N> where N: Display { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.label) } }
#[rustfmt::skip]
impl<N> FromStr for Node<N> where N: Default + FromStr {
    type Err = Report;
    fn from_str(s: &str) -> Result<Node<N>, Report> {
        match s.parse::<N>() {
            Ok(label) => Ok(Node { label }),
            Err(_) => Err(eyre!("Failed to create Node from str: {s}")),
        }
    }
}
#[rustfmt::skip]
impl<N> Node<N> where N: Default { fn new() -> Self { Node { label: N::default() } } }

impl<N> FromNewick for Node<N>
where
    N: Default + FromStr,
{
    /// Returns a [`Node`] created from a [Newick](https://en.wikipedia.org/wiki/Newick_format) node [`str`].
    ///
    /// ## Examples
    ///
    /// Just a node name.
    ///
    /// ```rust
    /// use rebar_phylo::{Node, FromNewick};
    /// let node = Node::from_newick(&"A;")?;
    /// assert_eq!(node, Node { label: "A".to_string()});
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// A node name and branch attributes.
    ///
    /// ```rust
    /// use rebar_phylo::{Node, FromNewick};
    /// let node = Node::from_newick(&"A:2:90;")?;
    /// assert_eq!(node, Node { label: "A".to_string()});
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    fn from_newick(newick: &str) -> Result<Self, Report> {
        let attributes: Vec<_> = newick.replace(';', "").split(':').map(String::from).collect();
        match attributes.is_empty() {
            true => Ok(Node::default()),
            false => match attributes[0].parse::<N>() {
                Ok(label) => Ok(Node { label }),
                Err(_) => Err(eyre!("Failed to create Node from newick string: {newick}")),
            },
        }
    }
}
