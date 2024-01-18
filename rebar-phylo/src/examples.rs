//! Example [`Phylogeny`] for testing and documentation.

use crate::Phylogeny;

/// Returns a [`Phylogeny`], with fully sampled internal
/// nodes and  recombination from 1-3 parents.
///
#[doc = include_str!("../../assets/docs/example_1.md")]
///
/// # Examples
///
/// ```rust
/// use rebar_phylo::{Phylogeny, examples};
/// let phylo: Phylogeny<&str, usize> = examples::example_1();
/// # let nodes = ["A", "B", "D", "E", "G", "H", "C", "F"].iter().collect::<Vec<_>>();
/// # let branches = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1].iter().collect::<Vec<_>>();
/// # assert_eq!(phylo.get_nodes()?, nodes);
/// assert_eq!(phylo.get_branches()?, branches);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn example_1() -> Phylogeny<&'static str, usize> {
    let data = vec![
        ("A", "B", 1),
        ("A", "C", 1),
        ("A", "D", 1),
        ("B", "D", 1),
        ("C", "F", 1),
        ("C", "G", 1),
        ("D", "E", 1),
        ("E", "G", 1),
        ("E", "H", 1),
        ("F", "G", 1),
    ];

    let phylo: Phylogeny<&str, usize> = Phylogeny::from(data);
    phylo
}

/// Returns a [`Phylogeny`], with unsampled internal nodes.
///
/// # Examples
///
/// ```rust
/// use rebar_phylo::{Phylogeny, examples};
/// let phylo: Phylogeny<&str, usize> = examples::example_2();
/// # let nodes = ["A", "B", "C"].iter().collect::<Vec<_>>();
/// # let branches = [1, 1].iter().collect::<Vec<_>>();
/// # assert_eq!(phylo.get_nodes()?, nodes);
/// assert_eq!(phylo.get_branches()?, branches);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn example_2() -> Phylogeny<&'static str, usize> {
    let data = vec![("B", "C", 1), ("A", "B", 1)];

    let phylo: Phylogeny<&str, usize> = Phylogeny::from(data);
    phylo
}
