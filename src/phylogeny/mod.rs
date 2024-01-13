//! Creation and manipulation of the [Phylogeny], as an ancestral recombination graph (ARG).

use color_eyre::eyre::{eyre, Report, Result};
use itertools::Itertools;
use petgraph::algo::is_cyclic_directed;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{EdgeIndex, EdgeReference, Graph, NodeIndex};
use petgraph::visit::{Dfs, EdgeRef, IntoNodeReferences};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use structdoc::StructDoc;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum NodeType {
    Recombinant,
    RecombinantDescendant,
    Default,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            NodeType::Recombinant => String::from("recombinant"),
            NodeType::RecombinantDescendant => String::from("recombinant_descendant"),
            NodeType::Default => String::from("default"),
        };

        write!(f, "{}", name)
    }
}

impl NodeType {
    pub fn get_color(&self) -> &str {
        match self {
            // dark blue
            NodeType::Default => "#1f77b4",
            // dark orange
            NodeType::Recombinant => "#ff7f0e",
            // light orange
            NodeType::RecombinantDescendant => "#ffbb78",
        }
    }
}

// ----------------------------------------------------------------------------
// Phylogeny

/// Phylogenetic representation of a dataset's population history as an ancestral recombination graph (ARG).
///
/// - The population nodes (`N`) can be a wide variety of types (ex. [`&str`](core::primitive::str), [`String`](std::string::String), [`Table`](crate::Table), [`usize`](core::primitive::str), etc.).
/// - The branch lengths (`L`) can be any type that can be cast into an [`f32`](std::primitive::f32) float.
/// - See the [Implementation](#impl-Phylogeny<N>-1) section for the allowed types based on traits.
///
/// ## [Toy1](crate::dataset::toy1)
///
/// The following is a [Mermaid](https://mermaid.js.org/) diagram representation of the [Toy1](crate::dataset::toy1) dataset. Dark blue represents non-recombinants, dark orange are recombinants, and light orange are descendants of recombinants.
///
#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
/// ---
/// title: Toy1
/// ---
/// graph LR;
///   A-->|1|B:::default;
///   A-->|1|C:::default;
///   A-.->|1|D:::recombinant;
///   B-.->|1|D:::recombinant;
///   D-.->|1|E:::recombinant_descendant;
///   E-.->|1|G:::recombinant;
///   E-.->|1|H:::recombinant_descendant;
///   C-->|1|F:::default;
///   C-.->|1|G:::recombinant;
///   F-.->|1|G:::recombinant;
///
/// classDef default stroke:#1f77b4
/// classDef recombinant stroke:#ff7f0e
/// classDef recombinant_descendant stroke:#ffbb78
///
/// linkStyle 4,6 stroke:#ffbb78
/// linkStyle 2,3,5,8,9 stroke:#ff7f0e
/// linkStyle 0,1,7 stroke:#1f77b4
/// ```
#[derive(Clone, Debug, Deserialize, Serialize, StructDoc)]
pub struct Phylogeny<N, L> {
    /// Ancestral recombination graph (ARG) of populations as a directed graph of parents and children.
    ///
    /// `N` are population nodes and `L` are branch lengths.
    #[structdoc(skip)]
    pub graph: Graph<N, L>,
    // /// Recombinants with multiple parents.
    // pub recombinants: Vec<N>,
    // /// Recombinants with multiple parents plus their descendants
    // pub recombinants_with_descendants: Vec<N>,
}

impl<N, L> Default for Phylogeny<N, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, L> Phylogeny<N, L> {
    /// Create a new empty [Phylogeny] with nodes (`N`) and branch lengths (`L`).
    ///
    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// # Examples
    ///
    /// Let the compiler figure out the type based on subsequent commands.
    ///
    /// ```rust
    /// let mut phylo = rebar::Phylogeny::new();
    /// phylo.add_edge("A", "B", 10)?;
    /// phylo.add_edge("B", "C", 2)?;
    /// phylo.add_edge("A", "C", 1)?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// A-->|10|B:::default;
    /// B-.->|2|C:::recombinant;
    /// A-.->|1|C:::recombinant;
    ///
    /// classDef default stroke:#1f77b4
    /// classDef recombinant stroke:#ff7f0e
    /// classDef recombinant_descendant stroke:#ffbb78
    ///
    /// linkStyle default stroke:#1f77b4
    /// linkStyle 1,2 stroke:#ff7f0e
    /// ```
    ///
    /// Manually specific type at creation.
    ///
    /// ```rust
    /// let mut phylo: rebar::Phylogeny<&str, u32> = rebar::Phylogeny::new();
    /// ```
    ///
    /// Use numeric nodes, with floating point branch lengths.
    ///
    /// ```rust
    /// let mut phylo = rebar::Phylogeny::new();
    /// phylo.add_edge(1, 2, 2.5)?;
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
    /// Use a complex data type.
    ///
    /// ```rust
    /// let table_1 = rebar::Table{ headers: vec!["A"], rows: vec![vec!["1", "2"]], path: None};
    /// let table_2 = rebar::Table{ headers: vec!["B"], rows: vec![vec!["3", "4"]], path: None};
    ///
    /// let mut phylo = rebar::Phylogeny::new();
    /// phylo.add_edge(table_1, table_2, 1)?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    ///   0["{headers:[A],rows:[[1,2]],path:null}"]-.->|1|1["{headers:[B],rows:[[3,4]],path:null}"]:::default;
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
}

impl<N, L> Phylogeny<N, L>
where
    N: Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + PartialEq,
    L: Clone + std::fmt::Debug + std::fmt::Display + num_traits::AsPrimitive<f32>,
{
    /// Adds an edge between parent and child nodes and returns the new edge index.
    ///
    /// - If the parent and child nodes don't exist yet in the phylogeny, those nodes are created.
    /// - If an edge already exists between parent and child, updatest the branch length and returns the existing node index.
    /// - If the new edge will create a cycle, returns an Error.
    ///
    /// # Arguments
    ///
    /// - `source` : Starting node (`N`) of the edge (ex. parent).
    /// - `target` : Finishing node (`N`) of the edge (ex. child).
    /// - `length` : The branch length (`L`) between source and target (ie. weight).
    ///
    /// # Examples
    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// If the parent and child nodes don't exist yet in the phylogeny, those nodes are created.
    ///
    /// ```rust
    /// let mut phylo = rebar::Phylogeny::new();
    /// p.add_edge("B", "C", 1)?;
    /// p.add_edge("A", "B", 2)?;
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
    /// If the new edge will create a cycle, returns an Error.
    ///
    /// ```rust
    /// # let mut phylo = rebar::Phylogeny::new();
    /// # p.add_edge("B", "C", 1)?;
    /// # p.add_edge("A", "B", 2)?;
    /// assert!(p.add_edge("C", "A", 1).is_err());
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
    pub fn add_edge(&mut self, source: N, target: N, length: L) -> Result<EdgeIndex, Report> {
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
        let edge_index = self.graph.update_edge(parent_node_index, child_node_index, length);

        // check if edge introduced a cycle
        if is_cyclic_directed(&self.graph) {
            Err(eyre!(
                "New edge between {source} and {target} introduced a cycle."
            ))?
        }

        Ok(edge_index)
    }

    /// Add node to the phylogeny and returns the node index.
    ///
    /// - If the node already exists in the phylogeny, returns the existing node index.
    ///
    /// # Arguments
    ///
    /// - `node` - Node (`N`) to add to the phylogeny.
    ///
    /// # Examples
    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// ```rust
    /// use petgraph::graph::NodeIndex;
    ///
    /// let mut phylo = rebar::Phylogeny::new();
    /// assert_eq!(p.add_node("A"), NodeIndex::new(0));
    /// assert_eq!(p.add_node("B"), NodeIndex::new(1));
    /// ```
    ///
    /// ```mermaid
    /// graph LR;
    /// A:::default;
    /// B:::default;
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
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
    pub fn from_vec(data: Vec<(N, N, L)>) -> Result<Self, Report> {
        let mut phylogeny = Phylogeny::new();

        // add edges between parent to child, creating new nodes as needed
        data.into_iter().try_for_each(|(p, c, l)| {
            phylogeny.add_edge(p, c, l)?;
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
    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// **Note**: See the [Toy1](#toy1) diagram to help interpret the results visually.
    ///
    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// ```rust
    /// use rebar::dataset::toy1;
    ///
    /// let mut phylo = toy1::phylogeny::get()?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// ### With Recombination (`true`)
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
    /// ### Without Recombination (`false`)
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
    ///
    /// let phylo = toy1::phylogeny::get()?;
    /// assert_eq!(phylo.get_children(&"A")?, [&"B", &"C", &"D"]);
    /// assert_eq!(phylo.get_children(&"B")?, [&"D"]);
    /// assert_eq!(phylo.get_children(&"C")?, [&"F", &"G"]);
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
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
    /// phylo.add_edge("A", "B", 1);
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

    /// Returns all node data in the phylogeny.
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
    /// let observed = phylo.get_nodes();
    /// let expected: Vec<_> = ["A", "B", "C", "D", "F", "G", "E"].iter().collect();
    /// assert_eq!(observed, expected);
    /// ```
    pub fn get_nodes(&self) -> Vec<&N> {
        self.graph.node_references().map(|(_i, n)| n).collect()
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
    /// phylo.add_edge("B", "C", 1);
    /// phylo.add_edge("A", "B", 1);
    ///
    /// let observed = phylo.get_node_indices();
    /// let expected = vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2)];
    /// assert_eq!(observed, expected);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_node_indices(&self) -> Vec<NodeIndex> {
        self.graph.node_indices().collect()
    }

    // /// Returns Hex code color of a node (`N`) based on its [NodeType].
    // pub fn get_node_color(&self, node: &N) -> Result<String, Report> {
    //     let node_type = self.get_node_type(node)?;
    //     let node_color = node_type.get_color().to_string();
    //     Ok(node_color)
    // }

    /// Returns the [NodeType] of a node (`N`) in the phylogeny.
    ///
    /// # Arguments
    ///
    /// - `node`- A node (`N`) in the phylogeny.
    ///
    /// # Examples
    ///
    /// ```rust
    /// ```
    pub fn get_node_type(&self, node: &N) -> Result<NodeType, Report> {
        let node_type = if self.is_recombinant(node)? {
            NodeType::Recombinant
        } else if self.is_recombinant_descendant(node)? {
            NodeType::RecombinantDescendant
        } else {
            NodeType::Default
        };

        Ok(node_type)
    }

    /// Returns the node representing the most recent ancestor that is a recombinant.
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
    /// p.add_edge("B", "C", 1);
    /// p.add_edge("A", "B", 1);
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
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
        let mut neighbors = self.graph.neighbors_directed(node_index, Direction::Incoming);
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
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
    /// p.add_edge("B", "C", 1);
    /// p.add_edge("A", "B", 1);
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
    /// phylo.add_edge("A", "B", 1);
    /// assert_eq!(false, phylo.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Returns true if a node is a recombinant with multiple immediate parents.
    ///
    /// Checks the number of incoming edges to a node, as recombinants will have more than 1 (ie. more than 1 parent).
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
            Some(node) => Ok(true),
            None => Ok(false),
        }
    }

    /// Returns the phylogeny as a [Dot](https://graphviz.org/doc/info/lang.html) graphviz String.
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
        let edges = |_, e: EdgeReference<'_, L>| {
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
    #[cfg_attr(doc, aquamarine::aquamarine)]
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
        let mut lines = vec!["graph LR;".to_string()];

        // start at root, construct a depth-first-search (Bfs)
        let root_index = self.get_root_index()?;
        let mut dfs = Dfs::new(&self.graph, root_index);

        // keep track of seen nodes
        let mut nodes_seen = Vec::new();

        // keep track of link styling for non-default nodes
        let mut link_style: HashMap<NodeType, Vec<_>> = HashMap::new();

        // keep track of link indices
        let mut link_i = 0;

        while let Some(node_index) = dfs.next(&self.graph) {
            let parent = self.get_node(&node_index)?;
            let parent_type = self.get_node_type(parent)?;
            let parent_label = parent.to_string().replace('"', "");
            let parent_i = node_index.index();

            nodes_seen.push(parent);

            // iterate through children
            self.get_children(parent, true)?.into_iter().try_for_each(|child| {

                let child_index = self.get_node_index(child)?;
                let child_label = child.to_string().replace('"', "");
                let child_type = self.get_node_type(child)?;
                let child_i = child_index.index();
                let edge_index = self.graph.find_edge(node_index, child_index).unwrap();
                let length = self.graph.edge_weight(edge_index).unwrap();
                let edge_char = match child_type == NodeType::Recombinant {
                    true => ".",
                    false => "",
                };

                let line = format!("  {parent_i}[\"{parent_label}\"]:::{parent_type}-{edge_char}->|{length}|{child_i}[\"{child_label}\"]:::{child_type};");
                lines.push(line);
                nodes_seen.push(child);

                link_style.entry(child_type).or_insert_with(Vec::new).push(link_i);
                link_i += 1;

                Ok::<(), Report>(())
            })?;
        }

        // add any floating nodes with no connections
        self.get_nodes().iter().filter(|node| !nodes_seen.contains(node)).for_each(|node| {
            lines.push(format!("  {node};"));
        });

        // write node and edge styles
        lines.push(String::new());
        // default coloring is a dark blue: #1f77b4
        // recombinant coloring is a dark orange: #ff7f0e
        // recombinant descendant coloring is a light orange: #ffbb78
        lines.push(format!("classDef default stroke:#1f77b4"));
        lines.push(format!("classDef recombinant stroke:#ff7f0e"));
        lines.push(format!("classDef recombinant_descendant stroke:#ffbb78"));
        lines.push(String::new());
        link_style.into_iter().for_each(|(node_type, ids)| {
            let color = node_type.get_color();
            let line = format!("linkStyle {} stroke:{color}", ids.iter().join(","));
            lines.push(line);
        });
        let mermaid = lines.into_iter().join("\n");

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
    //                     self.graph.add_edge(*parent_node_index, *child_node_index, branch_length);
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
