//! A library for ancestral recombination graphs (ARG).

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
#[cfg(feature = "dev")]
use structdoc::StructDoc;

// ----------------------------------------------------------------------------
// Phylogeny

/// # Introduction
///
/// A phylogeny as an ancestral recombination graph (ARG).
///
/// - The nodes (`N`) can be a wide variety of types (ex. [`str`], [`String`], [`usize`](core::primitive::str), [`Node`], etc.).
/// - The branches (`B`) must be a type that can be cast into an [`f32`] for the length.
/// - See the [Implementation](#impl-Phylogeny<N,+B>) section for the allowed types based on traits.
/// - See the [`Node`] and [`Branch`] structs for examples of complex data types.
#[cfg_attr(feature = "doc", aquamarine::aquamarine)]
/// ```mermaid
/// graph TD;
///
///   subgraph Legend
///     direction LR;
///     D1[ ] --->|Non-Recombination| D2[ ];
///     style D1 height:0px;
///     style D2 height:0px;
///     D3[ ] -..->|Recombination| R1[ ];
///     style D3 height:0px;
///     style R1 height:0px;
///   end
///
///   subgraph Toy1
///     direction LR;
///     0["A"]:::default-->|1|1["B"]:::default;
///     0["A"]:::default-->|1|2["C"]:::default;
///     0["A"]:::default-.->|1|3["D"]:::recombinant;
///     1["B"]:::default-.->|1|3["D"]:::recombinant;
///     3["D"]:::recombinant-->|1|6["E"]:::default;
///     6["E"]:::default-.->|1|5["G"]:::recombinant;
///     6["E"]:::default-->|1|7["H"]:::default;
///     2["C"]:::default-->|1|4["F"]:::default;
///     2["C"]:::default-.->|1|5["G"]:::recombinant;
///     4["F"]:::default-.->|1|5["G"]:::recombinant;
///   end
///
/// classDef default stroke:#1f77b4
/// classDef recombinant stroke:#ff7f0e
///
/// linkStyle 1,4,5,7,10,11 stroke:#ff7f0e
/// linkStyle 0,2,3,6,8,9 stroke:#1f77b4
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "dev", derive(StructDoc))]
pub struct Phylogeny<N, B> {
    /// Ancestral recombination graph (ARG) of populations as a directed graph of parents and children.
    ///
    /// `N` are population nodes and `B` are branches.
    #[cfg_attr(feature = "dev", structdoc(skip))]
    pub graph: Graph<N, B>,
    // /// Recombinants with multiple parents.
    // pub recombinants: Vec<N>,
    // /// Recombinants with multiple parents plus their descendants
    // pub recombinants_with_descendants: Vec<N>,
}

impl<N, B> Phylogeny<N, B>
where
    N: Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + PartialEq,
    B: std::fmt::Debug + std::fmt::Display + AsPrimitive<f32>,
{
    /// Returns a new empty [Phylogeny] with nodes (`N`) and branches (`B`).
    ///
    /// # Examples
    #[cfg_attr(feature = "doc", aquamarine::aquamarine)]
    ///
    /// Manually specify the type at creation, with [`str`] nodes (`N`) and [`u32`] branches (`B`).
    ///
    /// ```rust
    /// use rebarg::Phylogeny;
    /// let mut phylo: Phylogeny<&str, u32> = Phylogeny::new();
    /// phylo.add_branch("N1", "N2", 1234)?;
    /// # assert_eq!(phylo.get_nodes()?, vec!["N1", "N2"].iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches(), vec![1234].iter().collect::<Vec<_>>());
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
    /// Let the compiler figure out the type based on subsequent commands.
    ///
    /// ```rust
    /// let mut phylo = rebarg::Phylogeny::new();
    /// phylo.add_branch("A", "B", 10)?;
    /// phylo.add_branch("B", "C", 2)?;
    /// phylo.add_branch("A", "C", 1)?;
    ///
    /// # assert_eq!(phylo.get_nodes()?, vec!["A", "B", "C"].iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches(), vec![10, 2, 1].iter().collect::<Vec<_>>());
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
    /// Use numeric nodes, with floating point branch lengths.
    ///
    /// ```rust
    /// let mut phylo = rebarg::Phylogeny::new();
    /// phylo.add_branch(1, 2, 2.5)?;
    /// # assert_eq!(phylo.get_nodes()?, vec![1, 2].iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches(), vec![2.5].iter().collect::<Vec<_>>());
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
    /// Use custom data types, such as [`Node`] and [`Branch`].
    ///
    /// ```rust
    /// use rebarg::{Phylogeny, Node, Branch};
    /// let node_1 = Node { label: "A" };
    /// let node_2 = Node { label: "B" };
    /// let branch = Branch { length: 1.0 };
    ///
    /// let mut phylo = rebarg::Phylogeny::new();
    /// # let (n1, n2, b) = (node_1.clone(), node_2.clone(), branch.clone());
    /// phylo.add_branch(node_1, node_2, branch)?;
    /// # assert_eq!(phylo.get_nodes()?, vec![n1, n2].iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches(), vec![b].iter().collect::<Vec<_>>());
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
        Phylogeny {
            graph: Graph::new(),
            //recombinants: Vec::new(),
            //recombinants_with_descendants: Vec::new(),
        }
    }

    /// Returns the `Example 1` [Phylogeny], shown in the documentation [Introduction](#introduction).
    ///
    /// # Examples
    ///
    /// ```rust
    /// let phylo = rebarg::Phylogeny::example_1();
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

        Phylogeny::from_vec(data).unwrap()
    }

    /// Creates a branch (`B`) between the parent and child nodes (`N`) and returns the [EdgeIndex](petgraph::graph::EdgeIndex).
    ///
    /// # Arguments
    ///
    /// - `source` : Starting node (`N`) (ex. parent).
    /// - `target` : Destination node (`N`) (ex. child).
    /// - `branch` : The branch (`B`) to add between source and target nodes (`N`).
    ///
    /// # Examples
    #[cfg_attr(feature = "doc", aquamarine::aquamarine)]
    ///
    /// If the parent and child nodes don't exist yet in the phylogeny, these nodes are created.
    ///
    /// ```rust
    /// let mut phylo = rebarg::Phylogeny::new();
    /// phylo.add_branch("B", "C", 1)?;
    /// phylo.add_branch("A", "B", 2)?;
    /// # assert_eq!(phylo.get_nodes()?, vec!["A", "B", "C"].iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches(), vec![1, 2].iter().collect::<Vec<_>>());
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
    /// # let mut phylo = rebarg::Phylogeny::new();
    /// # phylo.add_branch("B", "C", 1)?;
    /// # phylo.add_branch("A", "B", 2)?;
    /// phylo.add_branch("A", "B", 50)?;
    /// # assert_eq!(phylo.get_nodes()?, vec!["A", "B", "C"].iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches(), vec![1, 50].iter().collect::<Vec<_>>());
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
    /// # let mut phylo = rebarg::Phylogeny::new();
    /// # phylo.add_branch("B", "C", 1)?;
    /// # phylo.add_branch("A", "B", 2)?;
    /// phylo.add_branch("C", "A", 1);
    /// # assert!(phylo.add_branch("C", "A", 1).is_err());
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
            Err(eyre!(
                "New edge between {source} and {target} introduced a cycle."
            ))?
        }

        Ok(edge_index)
    }

    /// Adds a new node to the phylogeny and returns the node index.
    ///
    /// - If the node already exists in the phylogeny, returns the existing node index.
    ///
    /// # Arguments
    ///
    /// - `node` - Node (`N`) to add to the phylogeny.
    ///
    /// # Examples
    #[cfg_attr(feature = "doc", aquamarine::aquamarine)]
    /// ```rust
    /// use rebarg::Phylogeny;
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
    pub fn add_node(&mut self, node: N) -> NodeIndex {
        match self.get_node_index(&node) {
            Ok(node_index) => node_index,
            Err(_) => self.graph.add_node(node),
        }
    }

    /// Returns a [Phylogeny] from a vector of parent and child nodes (`N`).
    ///
    /// # Arguments
    ///
    /// * `data` : Vector of tuples in the form: (Parent (`N`), Child (`N`), Branch Length (`L`))
    ///
    /// # Examples

    /// ```rust
    /// use rebar::Phylogeny;
    /// let v = vec![("A", "B", 1), ("A", "C", 3),  ("B", "C", 2) ];
    /// let phylo = Phylogeny::from_vec(v)?;
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
    /// classDef recombinant_descendant stroke:#ffbb78
    ///
    /// linkStyle default stroke:#1f77b4
    /// linkStyle 1,2 stroke:#ff7f0e
    /// ```
    pub fn from_vec(data: Vec<(N, N, B)>) -> Result<Self, Report> {
        let mut phylogeny = Phylogeny::new();

        // add edges between parent to child, creating new nodes as needed
        data.into_iter().try_for_each(|(p, c, l)| {
            phylogeny.add_branch(p, c, l)?;
            Ok::<(), Report>(())
        })?;

        // // set recombinants for quick lookup
        // todo!()
        // phylogeny.recombinants = phylogeny.get_recombinants(false).into_iter().cloned().collect();
        // phylogeny.recombinants_with_descendants = phylogeny.get_recombinants(true).into_iter().cloned().collect();

        Ok(phylogeny)
    }

    /// Returns all paths from the node to the root.
    ///
    /// # Arguments
    ///
    /// * `node` - A node (`N`) in the phylogeny.
    /// * `recombination` - `true` if descendants arising from recombination should be included. In the [Mermaid](Phylogeny::to_mermaid) representation, this means we are allowed to follow dashed, orange edges when `true`.
    ///
    /// # Examples

    /// **Note**: See the [Toy1](#toy1) diagram to help interpret the results visually.
    ///

    /// ```rust
    /// use rebar::dataset::toy1;
    ///
    /// let mut phylo = toy1::phylogeny::get()?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ## With Recombination (`true`)
    ///
    /// ```rust
    /// # use rebar::dataset::toy1;
    /// # let mut phylo = toy1::phylogeny::get()?;
    /// phylo.get_ancestors(&"B", true)?;
    /// # assert_eq!(phylo.get_ancestors(&"B", true)?,  vec![vec![&"A"]]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   A:::default;
    /// linkStyle default stroke:#1f77b4
    /// ```
    ///
    /// ```rust
    /// # use rebar::dataset::toy1;
    /// # let mut phylo = toy1::phylogeny::get()?;
    /// phylo.get_ancestors(&"D", true)?;
    /// # assert_eq!(phylo.get_ancestors(&"D", true)?,  vec![vec![&"B", &"A"], vec![&"A"]]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   A-->|1|B:::default;
    /// linkStyle default stroke:#1f77b4
    /// ```
    ///
    /// ## Without Recombination (`false`)
    ///
    /// ```rust
    /// # use rebar::dataset::toy1;
    /// # let mut phylo = toy1::phylogeny::get()?;
    /// phylo.get_ancestors(&"D", false)?;
    /// # assert!(phylo.get_ancestors(&"D", false)?.is_empty());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// Node `D` is a recombinant, and thus has no ancestors if set `recombination=false`.
    ///
    /// ```rust
    /// # use rebar::dataset::toy1;
    /// # let mut phylo = toy1::phylogeny::get()?;
    /// phylo.get_ancestors(&"E", false)?;
    /// assert_eq!(phylo.get_ancestors(&"E", false)?,  vec![vec![&"D"]]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   D:::recombinant;
    /// classDef recombinant stroke:#ff7f0e
    /// ```
    ///
    /// ```rust
    /// # use rebar::dataset::toy1;
    /// # let mut phylo = toy1::phylogeny::get()?;
    /// phylo.get_ancestors(&"E", false)?;
    /// assert_eq!(phylo.get_ancestors(&"H", false)?,  vec![vec![&"E", &"D"]]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   D:::recombinant-.->|1|E:::recombinant_descendant;
    ///
    ///   classDef recombinant stroke:#ff7f0e
    ///   classDef recombinant_descendant stroke:#ffbb78
    ///
    ///   linkStyle 0 stroke:#ff7f0e
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
                    let result =
                        path.iter().position(|n| self.is_recombinant(n).unwrap_or(false) == true);
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

    /// Returns nodes (`N`) of immediate children of the requested node (`N`).
    ///
    /// # Arguments
    ///
    /// * `node` - A node (`N`) in the phylogeny.
    /// * `recombination` - `true` if children arising from recombination should be included. In the [Mermaid](Phylogeny::to_mermaid) representation, this means we are allowed to follow dashed, orange edges when `true`.
    ///
    /// # Examples
    ///
    /// > **Note**: See the [Toy1](#toy1) diagram to interpret the results visually.
    ///
    /// ```rust
    /// use rebar::dataset::toy1;
    /// let phylo = toy1::phylogeny::get()?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ## With Recombination (`true`)
    ///
    /// ```rust
    /// # let phylo = rebar::dataset::toy1::phylogeny::get()?;
    /// phylo.get_children(&"A", true)?;
    /// assert_eq!(phylo.get_children(&"A", true)?, [&"B", &"C", &"D"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```rust
    /// //assert_eq!(phylo.get_children(&"B")?, [&"D"]);
    /// //assert_eq!(phylo.get_children(&"C")?, [&"F", &"G"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_children(&self, node: &N, recombination: bool) -> Result<Vec<&N>, Report> {
        let node_index = self.get_node_index(node)?;
        let neighbors = self.graph.neighbors_directed(node_index, Direction::Outgoing);
        let mut children = neighbors
            .into_iter()
            .map(|node_index| {
                let node_data = self.get_node(&node_index)?;
                Ok(node_data)
            })
            .collect::<Result<Vec<&N>, Report>>()?;

        if !recombination {
            children.retain(|n| !self.is_recombinant(n).unwrap_or(false));
        }

        // children order is last added to first added, reverse this
        children.reverse();

        Ok(children)
    }

    /// Returns all descendants of a node as a big pile (single vector), following all paths to tips.
    ///
    /// # Arguments
    ///
    /// * `node` - A node in the phylogeny.
    /// * `recombination` - True if descendants arising from recombination should be included.
    ///

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
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
    ///
    /// assert_eq!(phylo.get_descendants(&"E", true)?,  vec![&"G"]);
    /// assert_eq!(phylo.get_descendants(&"C", true)?,  vec![&"F", &"G"]);
    /// assert_eq!(phylo.get_descendants(&"A", false)?, vec![&"B", &"C", &"F"]);
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
            if node_index == self.get_node_index(&node)? {
                continue;
            }
            // Get node name
            let node_data = self.get_node(&node_index)?;
            descendants.push(node_data);
        }

        // if recombination is false, exclude descendants that are novel recombinants
        if !recombination {
            let anc = self.get_recombinant_ancestor(node).ok();
            // exclude descendants with a different recombinant ancestor
            descendants.retain(|d| anc == self.get_recombinant_ancestor(d).ok());
        }

        Ok(descendants)
    }

    /// Returns the node data that corresponds to the node index.
    ///
    /// ```rust
    /// use rebar::Phylogeny;
    /// use petgraph::graph::NodeIndex;
    ///
    /// let mut phylo = Phylogeny::new();
    /// phylo.add_branch("A", "B", 1);
    /// // B was the second node added to the tree, with a 0-based index of 1
    /// let node_index = NodeIndex::new(1);
    /// let expected = &"B";
    /// let observed = phylo.get_node(&node_index)?;
    /// assert_eq!(observed, expected);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_node(&self, node_index: &NodeIndex) -> Result<&N> {
        self.graph
            .node_references()
            .filter_map(|(i, n)| (i == *node_index).then_some(n))
            .next()
            .ok_or_else(|| eyre!("Failed to get node data for node index {node_index:?}"))
    }

    /// Returns all branches (`L`) in the phylogeny.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
    /// phylo.get_branches();
    /// ```
    pub fn get_branches(&self) -> Vec<&B> {
        self.graph.edge_references().map(|e| e.weight()).collect()
    }

    /// Returns all node data in the phylogeny.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
    /// let observed = phylo.get_nodes();
    /// let expected: Vec<_> = ["A", "B", "C", "D", "F", "G", "E"].iter().collect();
    /// assert_eq!(observed, expected);
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
    /// ```rust
    /// use rebar::Phylogeny;
    /// use petgraph::graph::NodeIndex;
    ///
    /// let v = vec![("A", "B", 1), ("A", "C", 3),  ("B", "C", 2) ];
    /// let phylo = Phylogeny::from_vec(v);
    ///
    /// let observed =  phylo.get_node_index(&"B")?;
    /// // "B" was the second node added, with a 0-based index of 1
    /// let expected =  petgraph::graph::NodeIndex::new(1);
    /// assert_eq!(observed, expected);
    ///
    /// let observed =  phylo.get_node_index(&"X");
    /// assert!(observed.is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_node_index(&self, node: &N) -> Result<NodeIndex> {
        self.graph
            .node_references()
            .filter_map(|(i, n)| (*n == *node).then_some(i))
            .next()
            .ok_or_else(|| eyre!("Failed to get node index of node {node}"))
    }

    /// Returns all node indices in the phylogeny.
    ///
    /// ```rust
    /// use rebar::Phylogeny;
    /// use petgraph::graph::NodeIndex;
    ///
    /// let mut phylo = Phylogeny::new();
    /// phylo.add_branch("B", "C", 1);
    /// phylo.add_branch("A", "B", 1);
    ///
    /// let observed = phylo.get_node_indices();
    /// let expected = vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2)];
    /// assert_eq!(observed, expected);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_node_indices(&self) -> Vec<NodeIndex> {
        self.graph.node_indices().collect()
    }

    /// Returns the node representing the most recent ancestor that is a recombinant.

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
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
    ///
    /// let observed = phylo.get_recombinant_ancestor(&"E")?;
    /// let expected = Some(&"D");
    /// assert_eq!(observed, expected);
    ///
    /// let observed = phylo.get_recombinant_ancestor(&"G")?;
    /// let expected = Some(&"D");
    /// assert_eq!(observed, expected);
    ///
    /// let observed = phylo.get_recombinant_ancestor(&"B")?;
    /// let expected = None;
    /// assert_eq!(observed, expected);
    ///
    /// let observed = phylo.get_recombinant_ancestor(&"D")?;
    /// let expected = None;
    /// assert_eq!(observed, expected);
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
                } else if self.is_recombinant(n)? {
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

    /// Returns the node data of the root.
    ///
    /// ```rust
    /// use rebar::Phylogeny;
    ///
    /// let mut p = Phylogeny::new();
    /// p.add_branch("B", "C", 1);
    /// p.add_branch("A", "B", 1);
    ///
    /// let observed = p.get_root()?;
    /// let expected = "A";
    ///
    /// assert_eq!(*observed, expected);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_root(&self) -> Result<&N, Report> {
        let root_index = self.get_root_index()?;
        self.get_node(&root_index)
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
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
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
    /// use rebar::dataset::toy1;
    /// use petgraph::Direction;
    ///
    /// let phylo = toy1::phylogeny::get()?;
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
                let mut parent_paths = self.get_paths(&parent_node, target, direction)?;

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

    /// Returns the node index of the root.
    ///
    /// ```rust
    /// use rebar::Phylogeny;
    /// use petgraph::graph::NodeIndex;
    ///
    /// let mut p = Phylogeny::new();
    /// p.add_branch("B", "C", 1);
    /// p.add_branch("A", "B", 1);
    ///
    /// let observed = p.get_root_index()?;
    /// // "A" is the root, and it was the 3rd node add, with a 0-based index of 2
    /// let expected = NodeIndex::new(2);
    ///
    /// assert_eq!(observed, expected);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_root_index(&self) -> Result<NodeIndex, Report> {
        if self.is_empty() {
            Err(eyre!(
                "Failed to locate root node index in phylogeny as graph is empty!."
            ))?
        }
        // reverse the graph to walk backwards towards the root
        let mut graph = self.graph.clone();
        graph.reverse();
        let mut root_index = NodeIndex::new(0);
        // Start at the first node added (index=0), and try to go deeper
        let mut dfs = Dfs::new(&graph, root_index);
        while let Some(node_index) = dfs.next(&graph) {
            root_index = node_index;
        }

        Ok(root_index)
    }

    /// Return true if the phylogeny graph has no data.
    ///
    /// ```rust
    /// use rebar::Phylogeny;
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
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
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
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
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
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
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

    /// Returns the phylogeny as a [Mermaid](https://mermaid.js.org/) graph String.
    ///
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
    /// use rebar::dataset::toy1;
    ///
    /// let phylo = toy1::phylogeny::get()?;
    /// println!("{}", phylo.to_mermaid()?);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    #[doc = include_str!("../../assets/docs/toy1_mermaid.md")]
    pub fn to_mermaid(&self) -> Result<String, Report> {
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

    //     /// Get all all paths from the node to the root.
    //     pub fn get_ancestors(&self, node: &T) -> Result<Vec<Vec<&T>>, Report> {
    //         let root = self.get_root_data()?;
    //         let mut paths = self.get_paths(node, root, petgraph::Incoming)?;

    //         // remove self name (first element) from paths, and then reverse order
    //         // so that it's ['root'.... name]
    //         paths.iter_mut().for_each(|p| {
    //             p.remove(0);
    //             p.reverse();
    //         });

    //         Ok(paths)
    //     }

    //     /// Identify the most recent common ancestor(s) (MRCA) shared between all node names.
    //     pub fn get_mrca(&self, nodes: &[&T], recombination: bool) -> Result<Vec<&T>, Report> {

    //         Ok(nodes.to_vec())
    //         // // if only one node name was provided, just return it
    //         // if nodes.len() == 1 {
    //         //     return Ok(nodes.to_vec());
    //         // }

    //         // // mass pile of all ancestors of all named nodes
    //         // let root = self.get_root_data()?;
    //         // let ancestors = nodes
    //         //     .into_iter()
    //         //     .map(|node| {
    //         //         let paths = self.get_paths(node, root, Direction::Incoming)?;
    //         //         let ancestors = paths.into_iter().flatten().unique().collect_vec();
    //         //         Ok(ancestors)
    //         //     })
    //         //     .collect::<Result<Vec<_>, Report>>()?
    //         //     .into_iter()
    //         //     .flatten()
    //         //     .collect::<Vec<_>>();

    //         // // get ancestors shared by all input nodes
    //         // let common_ancestors = ancestors
    //         //     .into_iter()
    //         //     .unique()
    //         //     .filter(|anc| {
    //         //         let count = ancestors.iter().filter(|node| *node == anc).count();
    //         //         count == nodes.len()
    //         //     })
    //         //     .collect_vec();

    //         // // get the depths (distance to root) of the common ancestors
    //         // let depths = common_ancestors
    //         //     .into_iter()
    //         //     .map(|node| {
    //         //         let paths = self.get_paths(node, root, Direction::Incoming)?;
    //         //         let longest_path =
    //         //             paths.into_iter().max_by(|a, b| a.len().cmp(&b.len())).unwrap_or_default();
    //         //         let depth = longest_path.len();
    //         //         Ok((node, depth))
    //         //     })
    //         //     .collect::<Result<Vec<_>, Report>>()?;

    //         // // get the mrca(s) (ie. most recent common ancestor) deepest from root
    //         // // tuple (population name, depth)
    //         // let max_depth = depths.into_iter().map(|(pop, depth)| depth).max().unwrap_or(Err(eyre!("Failed to get mrca"))?);
    //         // let mrca = depths.into_iter().filter_map(|(pop, depth)| (depth == max_depth).then_some(pop)).collect();
    //         // Ok(mrca)
    //     }

    //     /// Get non-recombinants node data.
    //     ///
    //     /// descendants: true if recombinant descendants should be excluded
    //     pub fn get_non_recombinants(&self, descendants: bool) -> Vec<&T> {
    //         let recombinants = self.get_recombinants(descendants);
    //         self.get_nodes().into_iter().filter(|n| !self.recombinants.contains(n)).collect()
    //     }

    //     /// Get problematic recombinants, where the parents are not sister taxa.
    //     /// They might be parent-child instead.
    //     // pub fn get_problematic_recombinants(&self) -> Result<Vec<&T>, Report> {
    //     //     let mut problematic_recombinants = Vec::new();
    //     //     let recombination = true;

    //     //     for recombinant in &self.recombinants {
    //     //         let parents = self.get_parents(recombinant)?;
    //     //         for i1 in 0..parents.len() - 1 {
    //     //             let p1 = &parents[i1];
    //     //             for p2 in parents.iter().skip(i1 + 1) {
    //     //                 let mut descendants = self.get_descendants(p2, recombination)?;
    //     //                 let ancestors =
    //     //                     self.get_ancestors(p2)?.to_vec().into_iter().flatten().collect_vec();
    //     //                 descendants.extend(ancestors);

    //     //                 if descendants.contains(p1) {
    //     //                     problematic_recombinants.push(recombinant.clone());
    //     //                     break;
    //     //                 }
    //     //             }
    //     //         }
    //     //     }

    //     //     Ok(problematic_recombinants)
    //     // }

    //     /// Get recombinants node data.
    //     ///
    //     /// descendants: true if recombinant descendants should be included
    //     pub fn get_recombinants(&self, descendants: bool) -> Vec<&T> {
    //         // construct iterator over node data
    //         let nodes = self.get_nodes().into_iter();

    //         match descendants {
    //             // include all descendants of recombinant nodes
    //             true => nodes.filter(|n| self.get_recombinant_ancestor(n).is_ok()).collect(),
    //             // include only the primary recombinant nodes
    //             false => nodes.filter(|n| self.is_recombinant(n).unwrap_or(false)).collect(),
    //         }
    //     }

    //     /// Remove a node in the graph.
    //     ///
    //     /// If prune is true, removes entire clade from graph.
    //     /// If prune is false, connects parents to children to fill the hole.
    //     pub fn remove(&mut self, node: &T, prune: bool) -> Result<Vec<N>, Report> {

    //         let mut removed_nodes = Vec::new();

    //         // pruning a clade is simple removal of all descendants
    //         if prune {
    //             let mut descendants = self.get_descendants(node, true)?.into_iter().cloned().collect_vec();

    //             // prune the clade
    //             descendants.into_iter().try_for_each(|node_data| {
    //                 let node_index = self.get_node_index(&node_data)?;
    //                 self.graph.remove_node(*node_index);
    //                 Ok::<(), Report>(())
    //             })?;

    //             // Update the recombinants attributes
    //             self.recombinants.retain(|n| !descendants.contains(&n));
    //             self.recombinants_with_descendants.retain(|n| !descendants.contains(&n));

    //             // update return value
    //             removed_nodes.append(&mut descendants);
    //         }
    //         // removing a single node is tricky if it's not a tip
    //         // the hole needs to be filled in between
    //         else {
    //             // get some attributes before we remove it
    //             let parents = self.get_parents(node)?;
    //             let mut children = self.get_children(node)?.into_iter().cloned().collect_vec();
    //             let is_recombinant = self.is_recombinant(node)?;

    //             // Delete the node
    //             let node_index = self.get_node_index(node)?;
    //             // todo!() branch length
    //             let branch_length = 1;
    //             //let branch_length = self.graph.node_weight(node_index)?;
    //             debug!("Deleting node {node}");
    //             self.graph.remove_node(*node_index);

    //             // If it was an interior node, connect parents and children
    //             children.into_iter().try_for_each(|child| {
    //                 let child_node_index = self.get_node_index(&child)?;
    //                 parents.iter().try_for_each(|parent| {
    //                     debug!("Connecting child {child} to new parent: {parent}");
    //                     let parent_node_index = self.get_node_index(parent)?;
    //                     // todo!() Branch length!
    //                     self.graph.add_branch(*parent_node_index, *child_node_index, branch_length);
    //                     Ok::<(), Report>(())
    //                 })?;
    //                 Ok::<(), Report>(())
    //             })?;

    //             // If it was a primary recombinant node, make all children primary recombinants
    //             if is_recombinant {
    //                 self.recombinants.append(&mut children);
    //             }

    //             // Update the recombinants attributes
    //             self.recombinants.retain(|n| n != node);
    //             self.recombinants_with_descendants.retain(|n| n != node);

    //             // Update return value
    //             removed_nodes.push(*node);
    //         }

    //         Ok(removed_nodes)
    //     }
}

// /// Phylogenetic methods for data types that can be serialized.
// impl<'a, N, T> Phylogeny<N, L>
// where
//     T: Clone + Deserialize<'a> + Display + Eq + Hash + PartialEq + Serialize,
// {
//     /// Read phylogeny from file.
//     pub fn read(path: &Path) -> Result<Phylogeny<T>, Report> {
//         let phylogeny = std::fs::read_to_string(path)
//             .wrap_err_with(|| format!("Failed to read file: {path:?}."))?;
//         let mut phylogeny: Phylogeny<T> = serde_json::from_str(&phylogeny)
//             .wrap_err_with(|| format!("Failed to parse file: {path:?}."))?;

//         // Add recombinants as quick lookup values
//         let rec = phylogeny.get_recombinants(false).into_iter().cloned().collect();
//         let non_rec = phylogeny.get_recombinants(true).into_iter().cloned().collect();

//         phylogeny.recombinants = rec;
//         phylogeny.recombinants_with_descendants = non_rec;

//         Ok(phylogeny)
//     }

//     /// Write phylogeny to file.
//     pub fn write(&self, path: &Path) -> Result<(), Report> {
//         // Create output file
//         let mut file = File::create(path)?;
//         // Check format based on extension
//         //let ext = utils::path_to_ext(Path::new(path))?;
//         // todo!() Ext restore
//         let ext = "json".to_string();

//         // format conversion
//         let output = match ext.as_str() {
//             // ----------------------------------------------------------------
//             // DOT file for graphviz
//             "dot" => {
//                 let mut output =
//                     format!("{}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));
//                 // set graph id (for cytoscape)
//                 output = str::replace(&output, "digraph", "digraph G");
//                 // set horizontal (Left to Right) format for tree-like visualizer
//                 output = str::replace(&output, "digraph {", "digraph {\n    rankdir=\"LR\";");
//                 output
//             }
//             // ----------------------------------------------------------------
//             // JSON for rebar
//             "json" => serde_json::to_string_pretty(&self)
//                 .unwrap_or_else(|_| panic!("Failed to parse phylogeny.")),
//             _ => {
//                 return Err(
//                     eyre!("Phylogeny write for extension .{ext} is not supported.")
//                         .suggestion("Please try .json or .dot instead."),
//                 )
//             }
//         };

//         // Write to file
//         file.write_all(output.as_bytes())
//             .unwrap_or_else(|_| panic!("Failed to write file: {:?}.", path));

//         Ok(())
//     }
// }

// ----------------------------------------------------------------------------
// Node

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Node {
    pub label: &'static str,
}

impl std::default::Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Node {
    pub fn new() -> Self {
        Node { label: "" }
    }
}

// ----------------------------------------------------------------------------
// Branch

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Branch {
    pub length: f32,
}

impl AsPrimitive<f32> for Branch {
    fn as_(self) -> f32 {
        self.length
    }
}

impl std::default::Default for Branch {
    fn default() -> Self {
        Self::new()
    }
}

impl Branch {
    pub fn new() -> Self {
        Branch { length: 0.0 }
    }
}

impl std::fmt::Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.length)
    }
}
