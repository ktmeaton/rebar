// ----------------------------------------------------------------------------
// Node

/// A node in the [`Phylogeny`] graph.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Node<N> {
    pub label: N,
}

impl<N> std::default::Default for Node<N> where N: std::default::Default {
    fn default() -> Self {
        Self::new()
    }
}

impl<N> Node<N> where N: std::default::Default {
    fn new() -> Self {
        Node { label: N::default() }
    }
}

impl<N> std::fmt::Display for Node<N>
where
    N: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label.to_string())
    }
}

impl<N> std::str::FromStr for Node<N> where N: std::default::Default + std::str::FromStr {
    type Err = Report;

    /// Convert a string to a [Node].
    fn from_str(name: &str) -> Result<Node<N>, Report> {
        match name.parse::<N>() {
            Ok(label) => Ok(Node {label}),
            Err(_) => Err(eyre!("Failed to create Node from str: {name}")),
        }
    }
}

impl<N> FromNewickStr for Node<N> where N: std::default::Default + std::str::FromStr {

    /// Returns a [Node] created from a [Newick](https://en.wikipedia.org/wiki/Newick_format) [`str`].
    ///
    /// # Examples
    ///
    /// Just a node name.
    ///
    /// ```rust
    /// use rebarg::{Node, FromNewickStr};
    /// let newick = "A;";
    /// let node = Node::from_newick_str(&newick)?;
    /// assert_eq!(node, Node { label: "A".to_string()});
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// A node name and branch attributes.
    ///
    /// ```rust
    /// use rebarg::{Node, FromNewickStr};
    /// let newick = "A:2:90;";
    /// let node = Node::from_newick_str(&newick)?;
    /// # assert_eq!(node, Node { label: "A".to_string()});
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    fn from_newick_str(newick: &str) -> Result<Self, Report> {
        let attributes = newick.replace(';', "").split(":").map(String::from).collect_vec();
        match attributes.len() == 0 {
            true => Ok(Node::default()),
            false => match attributes[0].parse::<N>() {
                Ok(label) => Ok(Node {label}),
                Err(_) => Err(eyre!("Failed to create Node from newick string: {newick}")),
            }
        }
    }
}