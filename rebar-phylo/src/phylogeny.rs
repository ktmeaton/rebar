use crate::{newick, FromNewick, ToMermaid};

use color_eyre::eyre::{eyre, Report, Result};
use itertools::Itertools;
use num_traits::AsPrimitive;
use petgraph::algo::is_cyclic_directed;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{EdgeIndex, EdgeReference, Graph, NodeIndex};
use petgraph::visit::{Dfs, EdgeRef, IntoNodeReferences};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;

use std::fmt::{Debug, Display};
use std::hash::Hash;

/// A [`Phylogeny`] as an ancestral recombination graph (ARG).
///
/// ## Introduction
///
/// - The nodes (`N`) can be a wide variety of types (ex. [`str`], [`String`], [`usize`](core::primitive::str), [`Node`](crate::Node), etc.).
/// - The branches (`B`) must be a type that can be cast into an [`f32`] for the length.
/// - See the [Implementation](#impl-Phylogeny<N,+B>) section for the allowed types based on traits.
/// - See the [`Node`](crate::Node) and [`Branch`](crate::Branch) structs for examples of complex data types.
///
#[doc = include_str!("../../assets/docs/example_1.md")]
///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Phylogeny<N, B> {
    /// Ancestral recombination graph (ARG) of populations as a directed graph of parents and children.
    ///
    /// `N` are population nodes and `B` are branches.
    pub graph: Graph<N, B>,
}

impl<N, B> Default for Phylogeny<N, B>
where
    N: Clone + Debug + Display + Eq + Hash + PartialEq,
    B: AsPrimitive<f32> + Debug + Display,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, B> Phylogeny<N, B>
where
    N: Clone + Debug + Display + Eq + Hash + PartialEq,
    B: AsPrimitive<f32> + Debug + Display,
{
    /// Returns a new empty [`Phylogeny`] with nodes (`N`) and branches (`B`).
    ///
    /// ## Examples
    ///
    /// Let the compiler figure out the type based on subsequent commands.
    ///
    /// ```rust
    /// let mut phylo = rebar_phylo::Phylogeny::new();
    /// phylo.add_branch("A", "B", 10)?;
    /// phylo.add_branch("B", "C", 2)?;
    /// phylo.add_branch("A", "C", 1)?;
    ///
    /// # assert_eq!(phylo.get_nodes()?, [&"A", &"B", &"C"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   A-->|10|B:::default;
    ///   B-.->|2|C:::recombinant;
    ///   A-.->|1|C:::recombinant;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    ///
    /// linkStyle default stroke:#1f77b4
    /// linkStyle 1,2 stroke:#ff7f0e
    /// ```
    ///
    /// Manually specify the types at creation, with [`str`] nodes (`N`) and [`u32`] branches (`B`).
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// let mut phylo: Phylogeny<&str, u32> = Phylogeny::new();
    /// phylo.add_branch("N1", "N2", 1234)?;
    /// # assert_eq!(phylo.get_nodes()?, [&"N1", &"N2"]);
    /// # assert_eq!(phylo.get_branches()?, [&1234]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   N1-->|1234|N2:::default;
    ///
    /// classDef default stroke:#1f77b4
    /// linkStyle default stroke:#1f77b4
    /// ```
    ///
    /// Use numeric nodes, with floating point branch lengths.
    ///
    /// ```rust
    /// let mut phylo = rebar_phylo::Phylogeny::new();
    /// phylo.add_branch(1, 2, 2.5)?;
    /// # assert_eq!(phylo.get_nodes()?,    [&1, &2]);
    /// # assert_eq!(phylo.get_branches()?, [&2.5]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// 1-->|2.5|2:::default;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    /// classDef recombinant_descendant stroke:#ffbb78
    ///
    /// linkStyle 0 stroke:#1f77b4
    /// ```
    ///
    /// Use custom data types, such as [`Node`](crate::Node) and [`Branch`](crate::Branch).
    ///
    /// ```rust
    /// use rebar_phylo::{Phylogeny, Node, Branch};
    /// let node_1 = Node { label: "A" };
    /// let node_2 = Node { label: "B" };
    /// let branch = Branch { length: 1.0, confidence: 0.0 };
    /// # let (n1, n2, b) = (node_1.clone(), node_2.clone(), branch.clone());
    ///
    /// let mut phylo = rebar_phylo::Phylogeny::new();
    /// phylo.add_branch(node_1, node_2, branch)?;
    /// # assert_eq!(phylo.get_nodes()?, [&n1, &n2]);
    /// # assert_eq!(phylo.get_branches()?, [&b]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   A-->|1.0|B;
    ///
    ///   classDef default stroke:#1f77b4
    ///   linkStyle 0 stroke:#1f77b4
    /// ```
    ///
    pub fn new() -> Self {
        Phylogeny { graph: Graph::new() }
    }

    /// Creates a branch (`B`) between the parent and child nodes (`N`) and returns the [`EdgeIndex`].
    ///
    /// - If the parent and child nodes don't exist yet in the phylogeny, these nodes are created.
    /// - If a branch already exists between parent and child, updates the branch and returns the existing node index.
    /// - If the new edge will create a cycle, returns an Error.
    ///
    /// ## Arguments
    ///
    /// - `source` : Starting node (`N`) (ex. parent).
    /// - `target` : Destination node (`N`) (ex. child).
    /// - `branch` : The branch (`B`) to add between source and target nodes (`N`).
    ///
    /// ## Examples
    ///
    /// If the parent and child nodes don't exist yet in the phylogeny, these nodes are created.
    ///
    /// ```rust
    /// let mut phylo = rebar_phylo::Phylogeny::new();
    /// phylo.add_branch("B", "C", 1)?;
    /// phylo.add_branch("A", "B", 2)?;
    /// # assert_eq!(phylo.get_nodes()?,    [&"A", &"B", &"C"]);
    /// # assert_eq!(phylo.get_branches()?, [&1, &2]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// A-->|2|B:::default;
    /// B-->|1|C:::default;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    /// classDef recombinant_descendant stroke:#ffbb78
    ///
    /// linkStyle default stroke:#1f77b4
    /// ```
    ///
    /// If a branch already exists between parent and child, updates the branch and returns the existing node index.
    ///
    /// ```rust
    /// # let mut phylo = rebar_phylo::Phylogeny::new();
    /// # phylo.add_branch("B", "C", 1)?;
    /// # phylo.add_branch("A", "B", 2)?;
    /// phylo.add_branch("A", "B", 50)?;
    /// # assert_eq!(phylo.get_nodes()?,    [&"A", &"B", &"C"]);
    /// # assert_eq!(phylo.get_branches()?, [&1, &50]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// A-->|50|B:::default;
    /// B-->|1|C:::default;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    /// classDef recombinant_descendant stroke:#ffbb78
    ///
    /// linkStyle default stroke:#1f77b4
    /// ```
    ///
    /// If the new edge will create a cycle, returns an Error.
    ///
    /// ```rust
    /// # let mut phylo = rebar_phylo::Phylogeny::new();
    /// # phylo.add_branch("B", "C", 1)?;
    /// # phylo.add_branch("A", "B", 2)?;
    /// let result = phylo.add_branch("C", "A", 1);
    /// assert!(result.is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// A-->|2|B:::default;
    /// B-->|1|C:::default;
    /// C-->|1|A:::default;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    /// classDef recombinant_descendant stroke:#ffbb78
    ///
    /// linkStyle default stroke:#1f77b4
    /// ```
    pub fn add_branch(&mut self, source: N, target: N, branch: B) -> Result<EdgeIndex, Report> {
        // check if parent in phylogeny, add node if not
        let parent_node_index = match self.get_node_index(&source) {
            Ok(node_index) => node_index,
            Err(_) => self.add_node(source.clone()),
        };

        // check if child in phylogeny, add node if not
        let child_node_index = match self.get_node_index(&target) {
            Ok(node_index) => node_index,
            Err(_) => self.add_node(target.clone()),
        };

        // add edge between parent to child, or update existing
        let edge_index = self.graph.update_edge(parent_node_index, child_node_index, branch);

        // check if edge introduced a cycle
        if is_cyclic_directed(&self.graph) {
            Err(eyre!("New edge between {source} and {target} introduced a cycle."))?
        }

        Ok(edge_index)
    }

    /// Adds a new node (`N`) to the [`Phylogeny`] and returns the [`NodeIndex`].
    ///
    /// - If the node already exists in the phylogeny, returns the existing [`NodeIndex`].
    ///
    /// ## Arguments
    ///
    /// - `node` - Node (`N`) to add to the phylogeny.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    ///
    /// let mut phylo: Phylogeny<&str, u32> = Phylogeny::new();
    /// let a_i = phylo.add_node("A (i=0)");
    /// let b_i = phylo.add_node("B (i=1)");
    /// # use petgraph::graph::NodeIndex;
    /// # assert_eq!(a_i, NodeIndex::new(0));
    /// # assert_eq!(b_i, NodeIndex::new(1));
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// A["A (i=0)"]:::default;
    /// B["B (i=1)"]:::default;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    /// classDef recombinant_descendant stroke:#ffbb78
    ///
    /// linkStyle default stroke:#1f77b4
    /// ```
    ///
    /// If the node already exists in the phylogeny, returns the existing [`NodeIndex`].
    ///
    /// ```rust
    /// # use rebar_phylo::Phylogeny;
    /// # let mut phylo: Phylogeny<&str, u32> = Phylogeny::new();
    /// # let a_i = phylo.add_node("A (i=0)");
    /// # let b_i = phylo.add_node("B (i=1)");
    /// use petgraph::graph::NodeIndex;
    ///
    /// // The index of B will still be 1, because it already exists
    /// let b_i = phylo.add_node("B (i=1)");
    /// assert_eq!(b_i, NodeIndex::new(1));
    /// ```
    pub fn add_node(&mut self, node: N) -> NodeIndex {
        match self.get_node_index(&node) {
            Ok(node_index) => node_index,
            Err(_) => self.graph.add_node(node),
        }
    }

    /// Returns all paths from the node (`N`) towards the root node (`N`).
    ///
    /// ## Arguments
    ///
    /// * `node` - A node (`N`) in the phylogeny.
    /// * `recombination` - `true` if recombination branches should be included.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// let phylo = rebar_phylo::examples::example_1();
    /// ```
    #[doc = include_str!("../../assets/docs/example_1.md")]
    ///
    /// With Recombination (`true`)
    ///
    /// ```rust
    /// # let phylo = rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_ancestors(&"B", true)?, [[&"A"]]);
    /// assert_eq!(phylo.get_ancestors(&"D", true)?, vec![vec![&"B", &"A"], vec![&"A"]]);
    /// assert_eq!(phylo.get_ancestors(&"E", true)?, vec![vec![&"D", &"B", &"A"], vec![&"D", &"A"]]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// Without Recombination (`false`)
    ///
    /// ```rust
    /// # let phylo = rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_ancestors(&"D", false)?, Vec::<Vec<&&str>>::new());
    /// assert_eq!(phylo.get_ancestors(&"E", false)?, [[&"D"]]);
    /// assert_eq!(phylo.get_ancestors(&"H", false)?, [[&"E", &"D"]]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_ancestors<'p>(
        &'p self,
        node: &'p N,
        recombination: bool,
    ) -> Result<Vec<Vec<&N>>, Report> {
        let root = self.get_root()?;

        // remove self name (first element) from paths, and then reverse order
        // so that it's ['root'.... name]
        let paths = self
            .get_paths(node, root, petgraph::Incoming)?
            .into_iter()
            .filter_map(|path| {
                let mut path = path;
                if !recombination {
                    // get index of first recombinant
                    let result = path.iter().position(|n| self.is_recombinant(n).unwrap_or(false));
                    if let Some(i) = result {
                        path = path[0..=i].to_vec();
                    }
                }
                //path.remove(0);
                path.retain(|n| *n != node);
                (!path.is_empty()).then_some(path)
            })
            .unique()
            .collect();

        Ok(paths)
    }

    /// Returns a branch (`B`) in the [`Phylogeny`] that corresponds to the [`EdgeIndex`].
    ///
    /// ## Arguments
    ///
    /// * `edge` - The [`EdgeIndex`] of a branch in the phylogeny.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// let mut phylo = rebar_phylo::Phylogeny::new();
    /// let edge_index = phylo.add_branch("A", "B", 1)?;
    /// assert_eq!(phylo.get_branch(&edge_index)?, &1);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_branch(&self, edge_index: &EdgeIndex) -> Result<&B, Report> {
        self.graph
            .edge_weight(*edge_index)
            .ok_or_else(|| eyre!("Failed to get branch of edge index: {edge_index:?}"))
    }

    /// Returns all branches (`B`) in the [`Phylogeny`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// let mut phylo = rebar_phylo::Phylogeny::new();
    /// phylo.add_branch("A", "B", 1)?;
    /// phylo.add_branch("B", "C", 2)?;
    /// assert_eq!(phylo.get_branches()?, [&1, &2]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_branches(&self) -> Result<Vec<&B>, Report> {
        Ok(Vec::from(self).into_iter().map(|(_, _, b)| b).collect())
    }

    /// Returns immediate child nodes (`N`) of the requested node (`N`).
    ///
    /// ## Arguments
    ///
    /// * `node` - A node (`N`) in the phylogeny.
    /// * `recombination` -  `true` if recombination branches should be included.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// let phylo = rebar_phylo::examples::example_1();
    /// ```
    ///
    #[doc = include_str!("../../assets/docs/example_1.md")]
    ///
    /// With Recombination (`true`)
    ///
    /// ```rust
    /// # let phylo = rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_children(&"A", true)?,  [&"B", &"C", &"D"]);
    /// assert_eq!(phylo.get_children(&"B", true)?,  [&"D"]);
    /// assert_eq!(phylo.get_children(&"C", true)?,  [&"F", &"G"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// Without Recombination (`false`)
    ///
    /// ```rust
    /// # let phylo = rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_children(&"A", false)?,  [&"B", &"C"]);
    /// assert_eq!(phylo.get_children(&"B", false)?,  Vec::<&&str>::new());
    /// assert_eq!(phylo.get_children(&"D", false)?,  [&"E"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_children(&self, node: &N, recombination: bool) -> Result<Vec<&N>, Report> {
        let node_index = self.get_node_index(node)?;
        let neighbors = self.graph.neighbors(node_index);
        let mut children = neighbors
            .into_iter()
            .map(|node_index| self.get_node(&node_index))
            .collect::<Result<Vec<&N>, Report>>()?;

        if !recombination {
            children.retain(|n| !self.is_recombinant(n).unwrap_or(false));
        }

        // children order is last added to first added, reverse this
        children.reverse();

        Ok(children)
    }

    /// Returns all descendant nodes (`N`) of a requested node as a big pile ([`Vec`]), following all paths to tips.
    ///
    /// ## Arguments
    ///
    /// * `node` - A node (`N`) in the phylogeny.
    /// * `recombination` - `true` if recombination branches should be included.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// let phylo = rebar_phylo::examples::example_1();
    /// ```
    #[doc = include_str!("../../assets/docs/example_1.md")]
    ///
    /// With Recombination (`true`)
    ///
    /// ```rust
    /// # let phylo = rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_descendants(&"E", true)?, [&"G", &"H"]);
    /// assert_eq!(phylo.get_descendants(&"D", true)?, [&"E", &"G", &"H"]);
    /// assert_eq!(phylo.get_descendants(&"B", true)?, [&"D", &"E", &"G", &"H"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// Without Recombination (`false`)
    ///
    /// ```rust
    /// # let phylo = rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_descendants(&"E", false)?, [&"H"]);
    /// assert_eq!(phylo.get_descendants(&"D", false)?, [&"E", &"H"]);
    /// assert_eq!(phylo.get_descendants(&"B", false)?, Vec::<&&str>::new());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_descendants(&self, node: &N, recombination: bool) -> Result<Vec<&N>, Report> {
        // Construct a depth-first-search (Dfs)
        let node_index = self.get_node_index(node)?;
        let mut dfs = Dfs::new(&self.graph, node_index);

        // get all descendants, initially including recombination
        let mut descendants = Vec::new();
        while let Some(node_index) = dfs.next(&self.graph) {
            // Exclude self
            if node_index == self.get_node_index(node)? {
                continue;
            }
            // Get node name
            let node_data = self.get_node(&node_index)?;
            descendants.push(node_data);
        }

        // if recombination is false, exclude descendants that are novel recombinants
        if !recombination {
            let anc = match self.is_recombinant(node)? {
                true => Some(node),
                false => self.get_recombinant_ancestor(node)?,
            };
            // exclude recombinants or descendants with a different recombinant ancestor
            descendants.retain(|d| {
                !self.is_recombinant(d).unwrap_or(false)
                    && anc == self.get_recombinant_ancestor(d).unwrap_or(None)
            });
        }

        Ok(descendants)
    }

    /// Returns the node (`N`) that corresponds to the [`NodeIndex`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// use petgraph::graph::NodeIndex;
    ///
    /// let mut phylo = Phylogeny::new();
    /// phylo.add_branch("A", "B", 1);
    /// // B was the second node added to the tree, with a 0-based index of 1
    /// let node_index = NodeIndex::new(1);
    /// assert_eq!(phylo.get_node(&node_index)?, &"B");
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_node(&self, node_index: &NodeIndex) -> Result<&N, Report> {
        self.graph
            .node_weight(*node_index)
            .ok_or_else(|| eyre!("Failed to get node data for node index {node_index:?}"))
    }

    /// Returns all nodes (`N`) in the [`Phylogeny`].
    ///
    /// - Order is based on a [Depth-First Search (Dfs)](Dfs).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// let phylo =  rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_nodes()?, [&"A", &"B", &"D", &"E", &"G", &"H", &"C", &"F"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_nodes(&self) -> Result<Vec<&N>, Report> {
        // get nodes via a depth-first search from the root
        let root_index = self.get_root_index()?;
        let mut dfs = Dfs::new(&self.graph, root_index);
        let mut nodes = Vec::new();
        while let Some(node_index) = dfs.next(&self.graph) {
            let node = self.get_node(&node_index)?;
            nodes.push(node);
        }
        Ok(nodes)
    }

    /// Returns the node index that corresponds to the node data.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// use petgraph::graph::NodeIndex;
    ///
    /// let v = [("A", "B", 1), ("A", "C", 3),  ("B", "C", 2) ];
    /// let phylo = Phylogeny::from(v);
    ///
    /// assert_eq!(phylo.get_node_index(&"B")?, NodeIndex::new(1));
    /// assert!(phylo.get_node_index(&"X").is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_node_index(&self, node: &N) -> Result<NodeIndex, Report> {
        self.graph
            .node_references()
            .filter_map(|(i, n)| (*n == *node).then_some(i))
            .next()
            .ok_or_else(|| eyre!("Failed to get node index of node {node}"))
    }

    /// Returns the node (`N`) representing the most recent ancestor in the [`Phylogeny`] that is a recombinant.
    ///
    /// ## Arguments
    ///
    /// - `node` - A node (`N`) in the phylogeny.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// let phylo = rebar_phylo::examples::example_1();
    /// ```
    #[doc = include_str!("../../assets/docs/example_1.md")]
    ///
    /// ```rust
    /// # let phylo = rebar_phylo::examples::example_1();
    /// assert_eq!(phylo.get_recombinant_ancestor(&"B")?, None);
    /// assert_eq!(phylo.get_recombinant_ancestor(&"D")?, None);
    /// assert_eq!(phylo.get_recombinant_ancestor(&"E")?, Some(&"D"));
    /// assert_eq!(phylo.get_recombinant_ancestor(&"G")?, Some(&"D"));
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_recombinant_ancestor<'p>(&'p self, node: &'p N) -> Result<Option<&N>, Report> {
        let mut recombinant = None;

        let root = self.get_root()?;
        let ancestor_paths = self.get_paths(node, root, petgraph::Incoming)?;

        // iterate along the paths
        for path in ancestor_paths {
            for n in path {
                // skip self node
                if n == node {
                    continue;
                }
                // once a recombinant ancestor has been found, break
                else if self.is_recombinant(n)? {
                    recombinant = Some(n);
                    break;
                }
            }
            // stop as soon as we find a recombinant ancestor
            if recombinant.is_some() {
                break;
            }
        }

        Ok(recombinant)
    }

    /// Returns node data of immediate parents.

    /// ```mermaid
    /// ---
    /// title: Toy1
    /// ---
    /// graph LR;
    ///   A-->B;
    ///   A-->C;
    ///   A-.->D:::recombinant;
    ///   B-.->D:::recombinant;
    ///   D-->E;
    ///   E-.->G:::recombinant;
    ///   C-->F;
    ///   C-.->G:::recombinant;
    ///   F-.->G:::recombinant;
    ///   classDef recombinant stroke:#ff7f0e;
    /// ```
    ///
    /// ```rust
    /// use rebar_phylo::examples::*;;
    ///
    /// let phylo = example_1();
    /// assert_eq!(phylo.get_parents(&"B")?, [&"A"]);
    /// assert_eq!(phylo.get_parents(&"D")?, [&"A", &"B"]);
    /// assert_eq!(phylo.get_parents(&"G")?, [&"C", &"E", &"F",]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_parents(&self, node: &N) -> Result<Vec<&N>, Report> {
        let node_index = self.get_node_index(node)?;
        let neighbors = self.graph.neighbors_directed(node_index, Direction::Incoming);
        let mut parents = neighbors
            .into_iter()
            .map(|node_index| {
                let node_index = self.get_node(&node_index)?;
                Ok(node_index)
            })
            .collect::<Result<Vec<&N>, Report>>()?;

        parents.reverse();

        Ok(parents)
    }

    /// Get all paths from the source node to the target node, always traveling
    /// in the specified direction (Incoming towards root, Outgoing towards tips)
    ///

    /// ```mermaid
    /// ---
    /// title: Toy1
    /// ---F
    /// graph LR;
    ///   A-->B;
    ///   A-->C;
    ///   A-.->D:::recombinant;
    ///   B-.->D:::recombinant;
    ///   D-->E;
    ///   E-.->G:::recombinant;
    ///   C-->F;
    ///   C-.->G:::recombinant;
    ///   F-.->G:::recombinant;
    ///   classDef recombinant stroke:#ff7f0e;
    /// ```
    ///
    /// ```rust
    /// use rebar_phylo::examples::*;;
    /// use petgraph::Direction;
    ///
    /// let phylo = example_1();
    /// let observed = phylo.get_paths(&"B", &"A", Direction::Incoming)?;
    /// let expected = [[&"B", &"A"]];
    /// assert_eq!(observed, expected);
    ///
    /// let observed = phylo.get_paths(&"A", &"E", Direction::Outgoing)?;
    /// let expected = vec![vec![&"A", &"D", &"E"], vec![&"A", &"B", &"D", &"E"]];
    /// assert_eq!(observed, expected);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_paths<'p>(
        &'p self,
        source: &'p N,
        target: &'p N,
        direction: petgraph::Direction,
    ) -> Result<Vec<Vec<&'p N>>, Report> {
        // container to hold the paths we've found, is a vector of vectors
        // because there might be recombinants with multiple paths
        let mut paths = Vec::new();

        // check that the source and target actually exist in the graph
        let source_node_index = self.get_node_index(source)?;
        let _ = self.get_node_index(target)?;

        // Check if we've reached the destination
        if source == target {
            paths.push(vec![source]);
        }
        // Otherwise, continue the search recursively
        else {
            let neighbors = self.graph.neighbors_directed(source_node_index, direction);
            neighbors.into_iter().try_for_each(|node_index| {
                let parent_node = self.get_node(&node_index)?;

                // recursively get path of each parent to the destination
                let mut parent_paths = self.get_paths(parent_node, target, direction)?;

                // prepend the origin to the paths
                parent_paths.iter_mut().for_each(|p| {
                    p.insert(0, source);
                    paths.push(p.clone());
                });

                Ok::<(), Report>(())
            })?;
        }

        Ok(paths)
    }

    /// Returns the node (`N`) corresponding to the root.
    ///
    /// - If multiple root nodes (`N`) are found, returns an Error.
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// let v = [("B", "C", 1), ("A", "B", 1)];
    /// let phylo = Phylogeny::from(v);
    ///
    /// let mut phylo = Phylogeny::new();
    /// phylo.add_branch("B", "C", 1);
    /// phylo.add_branch("A", "B", 1);
    /// assert_eq!(phylo.get_root()?, &"A");
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_root(&self) -> Result<&N, Report> {
        let root_index = self.get_root_index()?;
        self.get_node(&root_index)
    }

    /// Returns the node index of the root.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// use petgraph::graph::NodeIndex;
    ///
    /// let phylo = Phylogeny::from([("B", "C", 2), ("A", "B", 1)]);
    /// // "A" is the root, and it was the 3rd node add, with a 0-based index of 2
    /// assert_eq!(phylo.get_root_index()?, NodeIndex::new(2));
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// If multiple roots are found returns an error.
    ///
    /// ```rust
    /// # use rebar_phylo::Phylogeny;
    /// # use petgraph::graph::NodeIndex;
    /// let v = [("B", "C", 2), ("A1", "B", 1), ("A2", "B", 1)];
    /// let phylo = Phylogeny::from(v);
    /// assert!(phylo.get_root_index().is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_root_index(&self) -> Result<NodeIndex, Report> {
        if self.is_empty() {
            Err(eyre!("Failed to locate root node index in phylogeny as graph is empty!."))?
        }
        // reverse the graph to walk backwards towards the root
        let mut graph = self.graph.clone();
        graph.reverse();

        // get all nodes with no parents, could be root
        let root_indices: Vec<_> = self
            .graph
            .node_indices()
            .filter(|i| 0 == self.graph.edges_directed(*i, Direction::Incoming).count())
            .collect();

        match root_indices.len() {
            0 => Err(eyre!("Failed to locate root node index in phylogeny."))?,
            1 => Ok(root_indices[0]),
            _ => Err(eyre!("Failed to locate root node index in phylogeny, multiple roots found: {root_indices:?}"))?
        }
    }

    /// Returns true if the [`Phylogeny`] graph has no data.
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// let mut phylo = Phylogeny::new();
    /// assert_eq!(true, phylo.is_empty());
    ///
    /// phylo.add_branch("A", "B", 1);
    /// assert_eq!(false, phylo.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Returns true if a node is a recombinant with multiple immediate parents.
    ///
    /// Checks the number of incoming edges to a node, as recombinants will have more than 1 (ie. more than 1 parent).

    /// ```mermaid
    /// ---
    /// title: Toy1
    /// ---
    /// graph LR;
    ///   A-->B;
    ///   A-->C;
    ///   A-.->D:::recombinant;
    ///   B-.->D:::recombinant;
    ///   D-->E;
    ///   E-.->G:::recombinant;
    ///   C-->F;
    ///   C-.->G:::recombinant;
    ///   F-.->G:::recombinant;
    ///   classDef recombinant stroke:#ff7f0e;
    /// ```
    ///
    /// ```rust
    /// use rebar_phylo::examples::*;;
    ///
    /// let phylo = example_1();
    /// assert_eq!(phylo.is_recombinant(&"A")?, false);
    /// assert_eq!(phylo.is_recombinant(&"D")?, true);;
    /// assert_eq!(phylo.is_recombinant(&"E")?, false);
    /// assert_eq!(phylo.is_recombinant(&"G")?, true);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn is_recombinant(&self, node: &N) -> Result<bool, Report> {
        let node_index = self.get_node_index(node)?;
        let neighbors = self.graph.neighbors_directed(node_index, Direction::Incoming);
        let num_edges = neighbors.count();
        Ok(num_edges > 1)
    }

    /// Returns true if a node is a descendant of a recombinant.

    /// ```mermaid
    /// ---
    /// title: Toy1
    /// ---
    /// graph LR;
    ///   A-->B;
    ///   A-->C;
    ///   A-.->D:::recombinant;
    ///   B-.->D:::recombinant;
    ///   D-->E;
    ///   E-.->G:::recombinant;
    ///   C-->F;
    ///   C-.->G:::recombinant;
    ///   F-.->G:::recombinant;
    ///   classDef recombinant stroke:#ff7f0e;
    /// ```
    ///
    /// ```rust
    /// use rebar_phylo::examples::*;;
    ///
    /// let phylo = example_1();
    /// assert_eq!(phylo.is_recombinant_descendant(&"B")?, false);
    /// assert_eq!(phylo.is_recombinant_descendant(&"D")?, false);;
    /// assert_eq!(phylo.is_recombinant_descendant(&"E")?, true);
    /// assert_eq!(phylo.is_recombinant_descendant(&"G")?, true);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn is_recombinant_descendant(&self, node: &N) -> Result<bool, Report> {
        match self.get_recombinant_ancestor(node)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Returns the phylogeny as a [Dot](https://graphviz.org/doc/info/lang.html) graphviz String.

    /// ```mermaid
    /// ---
    /// title: Toy1
    /// ---
    /// graph LR;
    ///   A-->B;
    ///   A-->C;
    ///   A-.->D:::recombinant;
    ///   B-.->D:::recombinant;
    ///   D-->E;
    ///   E-.->G:::recombinant;
    ///   C-->F;
    ///   C-.->G:::recombinant;
    ///   F-.->G:::recombinant;
    ///   classDef recombinant stroke:#ff7f0e;
    /// ```
    ///
    /// ```rust
    /// use rebar_phylo::examples::*;;
    ///
    /// let phylo = example_1();
    /// println!("{}", phylo.to_dot()?);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```test
    /// digraph {
    ///     rankdir="LR"
    ///     0 [ label=A recombinant=false ]
    ///     1 [ label=B recombinant=false ]
    ///     2 [ label=C recombinant=false ]
    ///     3 [ label=D recombinant=true color=orange]
    ///     4 [ label=F recombinant=false ]
    ///     5 [ label=G recombinant=true color=orange]
    ///     6 [ label=E recombinant=false ]
    ///     0 -> 1 [ parent=A child=B, style=solid, weight=1 ]
    ///     0 -> 2 [ parent=A child=C, style=solid, weight=1 ]
    ///     0 -> 3 [ parent=A child=D, style=dashed, weight=1 ]
    ///     1 -> 3 [ parent=B child=D, style=dashed, weight=1 ]
    ///     2 -> 4 [ parent=C child=F, style=solid, weight=1 ]
    ///     2 -> 5 [ parent=C child=G, style=dashed, weight=1 ]
    ///     3 -> 6 [ parent=D child=E, style=solid, weight=1 ]
    ///     6 -> 5 [ parent=E child=G, style=dashed, weight=1 ]
    ///     4 -> 5 [ parent=F child=G, style=dashed, weight=1 ]
    /// }
    /// ```
    pub fn to_dot(&self) -> Result<String, Report> {
        let config = &[Config::NodeNoLabel, Config::EdgeNoLabel];
        let edges = |_, e: EdgeReference<'_, B>| {
            let source = self
                .get_node(&e.source())
                .expect("Failed to get source node of edge reference {e:?}");
            let target = self
                .get_node(&e.target())
                .expect("Failed to get target node of edge reference {e:?}");
            let is_recombinant = self
                .is_recombinant(target)
                .expect("Failed to determine if target node {target} is a recombinant.");
            format!(
                "parent={source} child={target}, style={style}, weight={weight} ",
                style = match is_recombinant {
                    true => "dashed",
                    false => "solid",
                },
                weight = e.weight(),
            )
        };
        let nodes = |_, (_i, node): (NodeIndex, &N)| {
            let is_recombinant = self
                .is_recombinant(node)
                .expect("Failed to determine if node {node} is a recombinant.");
            let color = match is_recombinant {
                true => "color=orange",
                false => "",
            };
            format!("label={node} recombinant={is_recombinant} {color}")
        };
        let dot = Dot::with_attr_getters(&self.graph, config, &edges, &nodes).to_string();

        // add direction LR
        let dot = dot.replace("digraph {", "digraph {\n    rankdir=\"LR\"");

        Ok(dot)
    }
}

/// Returns a [`Phylogeny`] created from an iteratable object `I`.
impl<I, N, B> From<I> for Phylogeny<N, B>
where
    I: IntoIterator<Item = (N, N, B)>,
    N: Clone + Debug + Display + Eq + Hash + PartialEq,
    B: AsPrimitive<f32> + Debug + Display,
{
    /// Returns a [Phylogeny] from a vector of nodes (`N`) and branches (`B`).
    ///
    /// ## Arguments
    ///
    /// - `v` : Vector of tuples in the form: (Parent (`N`), Child (`N`), Branch (`B`))
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// let v = [("A", "B", 1), ("A", "C", 3),  ("B", "C", 2) ];
    /// let phylo = Phylogeny::from(v);
    /// # assert_eq!(phylo.get_nodes()?,    [&"A", &"B", &"C"]);
    /// # assert_eq!(phylo.get_branches()?, [&1, &3, &2];
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// A-->|1|B:::default;
    /// B-.->|2|C:::default;
    /// A-.->|3|C:::recombinant;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    ///
    /// linkStyle default stroke:#1f77b4
    /// linkStyle 1,2 stroke:#ff7f0e
    /// ```
    ///
    /// Errors encountered will cause a [panic].
    ///
    /// ```rust
    /// # // panic test
    /// # use rebar_phylo::Phylogeny;
    /// let v = [("A", "B", 1), ("A", "C", 3),  ("B", "A", 2) ];
    /// let result = std::panic::catch_unwind(|| Phylogeny::from(v));
    /// # assert!(result.is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    fn from(it: I) -> Self {
        let mut phylogeny = Phylogeny::new();
        let msg = "Failed to convert vector into phylogeny";
        // create initial
        it.into_iter().for_each(|(p, c, l)| {
            let result = phylogeny.add_branch(p.clone(), c.clone(), l);
            result.expect(msg);
        });
        phylogeny
    }
}

impl<'p, N, B> From<&'p Phylogeny<N, B>> for Vec<(&'p N, &'p N, &'p B)>
where
    N: Clone + Debug + Display + Eq + Hash + PartialEq,
    B: AsPrimitive<f32> + Debug + Display,
{
    /// Returns references to all branches in the [`Phylogeny`].
    ///
    /// ## Arguments
    ///
    /// - `phylogeny` : Phylogeny to convert to vec.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// let v_in = [("A", "B", 1), ("A", "C", 3) ];
    /// let phylo = Phylogeny::from(v_in);
    /// let v_out = Vec::from(&phylo);
    /// assert_eq!(v_out, [(&"A", &"B", &1), (&"A", &"C", &3)]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    fn from(phylogeny: &'p Phylogeny<N, B>) -> Self {
        let msg = "Failed to convert phylogeny into vec";
        phylogeny
            .graph
            .edge_indices()
            .map(|edge_index| {
                let branch = phylogeny.graph.edge_weight(edge_index).expect(msg);
                let (source_i, target_i) = phylogeny.graph.edge_endpoints(edge_index).expect(msg);
                let source = phylogeny.get_node(&source_i).expect(msg);
                let target = phylogeny.get_node(&target_i).expect(msg);
                (source, target, branch)
            })
            .collect()
    }
}

impl<N, B> From<Phylogeny<N, B>> for Vec<(N, N, B)>
where
    N: Clone + Debug + Display + Eq + Hash + PartialEq,
    B: AsPrimitive<f32> + Debug + Display,
{
    /// Returns values of all branches in the [`Phylogeny`].
    ///
    /// ## Arguments
    ///
    /// - `phylogeny` : [`Phylogeny`] to convert to vec.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar_phylo::Phylogeny;
    /// let v_in = vec![("A", "B", 1), ("A", "C", 3) ];
    /// let phylo = Phylogeny::from(v_in);
    /// let v_out = Vec::from(phylo);
    /// assert_eq!(v_out, [("A", "B", 1), ("A", "C", 3)]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[allow(clippy::clone_on_copy)]
    fn from(phylogeny: Phylogeny<N, B>) -> Self {
        Vec::from(&phylogeny)
            .into_iter()
            .map(|(n1, n2, b)| (n1.clone(), n2.clone(), b.clone()))
            .collect()
    }
}

impl<N, B> FromNewick for Phylogeny<N, B>
where
    N: Clone + Debug + Display + Eq + Hash + PartialEq + FromNewick,
    B: AsPrimitive<f32> + Debug + Display + FromNewick,
{
    /// Returns a [`Phylogeny`] created from a [Newick](https://en.wikipedia.org/wiki/Newick_format) string.
    ///
    /// ## Arguments
    ///
    /// - `newick` - A Newick [`str`] (ex. `"(A,B);"`)
    ///
    /// ## Examples
    ///
    /// A Newick [`str`] with only tip names.
    ///
    /// ```rust
    /// use rebar_phylo::{Phylogeny, Node, Branch, FromNewick};
    /// let newick = "(A,B);";
    /// let phylo: Phylogeny<Node<String>, Branch> = Phylogeny::from_newick(&newick)?;
    /// # let nodes    = ["NODE_0", "A", "B"].map(|n| Node::from_newick(n).unwrap());
    /// # let branches = [0.0, 0.0].map(|n| Branch::from_newick(&format!(":{n}")).unwrap());
    /// # assert_eq!(phylo.get_nodes()?,    nodes.iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches()?, branches.iter().collect::<Vec<_>>());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// NODE_0-->|0|A:::default;
    /// NODE_0-->|0|B:::default;
    ///
    /// classDef default stroke:#1f77b4
    /// linkStyle default stroke:#1f77b4
    /// ```
    fn from_newick(newick: &str) -> Result<Phylogeny<N, B>, Report> {
        let data = newick::str_to_vec(newick, None, 0)?;
        let phylo = Phylogeny::from(data);
        Ok(phylo)
    }
}

impl<N, B> ToMermaid for Phylogeny<N, B>
where
    N: Clone + Debug + Display + Eq + Hash + PartialEq,
    B: AsPrimitive<f32> + Debug + Display,
{
    /// Returns a [Mermaid](https://mermaid.js.org/) [`str`] created from a [`Phylogeny`].
    fn to_mermaid(&self) -> Result<String, Report> {
        let mut mermaid = String::new();
        mermaid.push_str("graph TD;\n\n");

        // keep track of seen nodes, link styling, and indices
        let mut nodes_seen = Vec::new();
        let mut link_style = HashMap::new();
        let mut link_i = 0;

        // --------------------------------------------------------------------
        // Legend

        mermaid.push_str("  subgraph Legend\n");
        mermaid.push_str("    direction LR;\n");
        mermaid.push_str("    D1[ ] --->|Non-Recombination| D2[ ];\n");
        link_style.entry("default").or_insert_with(Vec::new).push(link_i);
        link_i += 1;
        mermaid.push_str("    style D1 height:0px;\n");
        mermaid.push_str("    style D2 height:0px;\n");
        mermaid.push_str("    D3[ ] -..->|Recombination| R1[ ];\n");
        link_style.entry("recombinant").or_insert_with(Vec::new).push(link_i);
        link_i += 1;

        mermaid.push_str("    style D3 height:0px;\n");
        mermaid.push_str("    style R1 height:0px;\n");
        mermaid.push_str("  end\n\n");

        // --------------------------------------------------------------------
        // Phylogeny

        mermaid.push_str("  subgraph Phylogeny\n");
        mermaid.push_str("    direction LR;\n");

        // start at root, construct a depth-first-search (Bfs)
        let root_index = self.get_root_index()?;
        let mut dfs = Dfs::new(&self.graph, root_index);

        // iterate through nodes in a depth-first search from root
        while let Some(node_index) = dfs.next(&self.graph) {
            // get node (parent/source) attributes
            let parent = self.get_node(&node_index)?;
            let parent_type = match self.is_recombinant(parent)? {
                true => "recombinant",
                false => "default",
            };
            let parent_label = parent.to_string().replace('"', "");
            let parent_i = node_index.index();

            // convert node attributes into mermaid format
            let parent_mermaid = format!("{parent_i}[\"{parent_label}\"]:::{parent_type}");
            nodes_seen.push(parent);

            // iterate through children of node
            self.get_children(parent, true)?.into_iter().try_for_each(|child| {
                // get child node attributes
                let child_index = self.get_node_index(child)?;
                let child_label = child.to_string().replace('"', "");
                let child_type = match self.is_recombinant(child)? {
                    true => "recombinant",
                    false => "default",
                };
                let child_i = child_index.index();
                let edge_index = self.graph.find_edge(node_index, child_index).unwrap();
                let length = self.graph.edge_weight(edge_index).unwrap();

                // convert child attributes to mermaid
                let edge_char = match child_type == "recombinant" {
                    true => ".",
                    false => "",
                };

                let child_mermaid = format!("{child_i}[\"{child_label}\"]:::{child_type}");
                let edge_mermaid = format!("-{edge_char}->|{length}|");
                let line = format!("    {parent_mermaid}{edge_mermaid}{child_mermaid};\n");
                mermaid.push_str(&line);

                // update link style map
                link_style.entry(child_type).or_insert_with(Vec::new).push(link_i);
                link_i += 1;

                Ok::<(), Report>(())
            })?;
        }

        // add any floating nodes with no connections
        self.get_nodes()?.iter().filter(|node| !nodes_seen.contains(node)).for_each(|node| {
            mermaid.push_str(&format!("    {node};\n"));
        });

        mermaid.push_str("  end\n\n");

        // --------------------------------------------------------------------
        // Style

        // default coloring is a dark blue: #1f77b4
        // recombinant coloring is a dark orange: #ff7f0e
        mermaid.push_str("classDef default stroke:#1f77b4\n");
        mermaid.push_str("classDef recombinant stroke:#ff7f0e\n\n");
        link_style.into_iter().for_each(|(node_type, ids)| {
            let color = match node_type == "recombinant" {
                true => "#ff7f0e",
                false => "#1f77b4",
            };
            let line = format!("linkStyle {} stroke:{color}\n", ids.iter().join(","));
            mermaid.push_str(&line);
        });

        Ok(mermaid)
    }
}
