//use crate::utils;
//use color_eyre::eyre::{eyre, Report, Result, WrapErr};
//use color_eyre::Help;
//use itertools::Itertools;
//use log::debug;
//use petgraph::dot::{Config, Dot};
use petgraph::graph::Graph;
//use petgraph::graph::{Graph, NodeIndex};
//use petgraph::visit::{Dfs, IntoNodeReferences};
//use petgraph::Direction;
use serde::{Deserialize, Serialize};
// use serde_json;
// use std::fmt::Display;
// use std::fs::File;
// use std::hash::Hash;
// use std::io::Write;
// use std::path::Path;

// ----------------------------------------------------------------------------
// Phylogeny

/// Phylogenetic representation of a dataset's population history as an
/// ancestral recombination graph (ARG).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Phylogeny<T> {
    // Directed graph of parents and children.
    pub graph: Graph<T, usize>,
    // Recombinants with multiple parents.
    pub recombinants: Vec<T>,
    // // Recombinants with multiple parents plus their descendants
    pub recombinants_with_descendants: Vec<T>,
}

impl<T> Default for Phylogeny<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Phylogeny<T> {
    pub fn new() -> Self {
        Phylogeny {
            graph: Graph::new(),
            recombinants: Vec::new(),
            recombinants_with_descendants: Vec::new(),
        }
    }
}

// /// Phylogenetic methods for any data type that can be displayed.
// impl<T> Phylogeny<T>
// where
//     T: Clone + Display + Eq + Hash + PartialEq,
// {

//     /// Create a phylogeny from a map of parent to children
//     ///
//     /// # Arguments
//     ///
//     /// data: Vector of (Parent, Child, Branch Length)
//     pub fn create(data: Vec<(T, T, usize)>) -> Self {
//         let mut phylogeny = Phylogeny::new();

//         data.into_iter().for_each(|(parent, child, branch_length)| {
//             // add parent to phylogeny
//             let parent_node_index = match phylogeny.get_node_index(&parent) {
//                 Ok(node_index) => *node_index,
//                 Err(_) => phylogeny.graph.add_node(parent),
//             };
//             // add child to phylogeny
//             let child_node_index = match phylogeny.get_node_index(&child) {
//                 Ok(node_index) => *node_index,
//                 Err(_) => phylogeny.graph.add_node(child),
//             };
//             // add edge bewteen parent to child
//             phylogeny.graph.add_edge(parent_node_index, child_node_index, branch_length);
//         });

//         // set recombinants for quick lookup
//         phylogeny.recombinants = phylogeny.get_recombinants(false).into_iter().cloned().collect();
//         phylogeny.recombinants_with_descendants = phylogeny.get_recombinants(true).into_iter().cloned().collect();

//         phylogeny
//     }

//     /// Return true if the phylogeny graph has no data.
//     pub fn is_empty(&self) -> bool {
//         self.graph.node_count() == 0
//     }

//     /// Return true if a node is a recombinant.
//     ///
//     /// Checks the number of incoming edges to a node, as recombinants
//     /// will have more than 1 (ie. more than 1 parent).
//     pub fn is_recombinant(&self, node: &T) -> Result<bool, Report> {
//         let node_index = self.get_node_index(node)?;
//         let neighbors = self.graph.neighbors_directed(*node_index, Direction::Incoming);
//         let num_edges = neighbors.count();
//         Ok(num_edges > 1)
//     }

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

//     /// Get immediate children of node.
//     pub fn get_children(&self, node: &T) -> Result<Vec<&T>, Report> {
//         let node_index = self.get_node_index(&node)?;
//         let neighbors = self.graph.neighbors_directed(*node_index, Direction::Outgoing);
//         let mut children = neighbors.into_iter().map(|node_index| {
//             let node_data = self.get_node_data(&node_index)?;
//             Ok(node_data)
//         }).collect::<Result<Vec<&T>, Report>>()?;

//         // children order is last added to first added, reverse this
//         children.reverse();

//         Ok(children)
//     }

//     // Get all descendants of a node.
//     //
//     // Returns a big pile (single vector) of all descendants in all paths to tips.
//     // Reminder, this function will also include the node itself.
//     pub fn get_descendants(&self, node: &T, recombination: bool) -> Result<Vec<&T>, Report> {

//         // Construct a depth-first-search (Dfs)
//         let node_index = self.get_node_index(node)?;
//         let mut dfs = Dfs::new(&self.graph, *node_index);

//         let mut descendants = Vec::new();
//         while let Some(node_index) = dfs.next(&self.graph) {
//             // Get node name
//             let node_data = self.get_node_data(&node_index)?;
//             descendants.push(node_data);
//         }

//         // // if recombination is false, exclude descendants that are novel recombinants
//         // if !recombination {
//         //     let recombinant_ancestor = self.get_recombinant_ancestor(node).ok();
//         //     descendants.retain(|node| recombinant_ancestor == )
//         //     descendants = descendants
//         //         .into_iter()
//         //         .filter(|desc| recombinant_ancestor == self.get_recombinant_ancestor(desc).ok())
//         //         .collect_vec();
//         // }

//         Ok(descendants)
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

//     /// Get node data from node index
//     pub fn get_node_data(&self, node_index: &NodeIndex) -> Result<&T, Report> {
//         self.graph
//             .node_references()
//             .into_iter()
//             .filter_map(|(i, n)| (i == *node_index).then_some(n))
//             .next()
//             .ok_or(Err(eyre!("Node index {node_index:?} is not in the phylogeny."))?)
//     }

//     /// Get node index from node data.
//     pub fn get_node_index(&self, node: &T) -> Result<&NodeIndex, Report> {
//         self.graph
//             .node_references()
//             .into_iter()
//             .filter_map(|(i, n)| (n == node).then_some(&i))
//             .next()
//             .ok_or(Err(eyre!("Node {node} is not in the phylogeny."))?)
//     }

//     /// Get all node data.
//     pub fn get_nodes(&self) -> Vec<&T> {
//         self.graph.node_references().map(|(_, n)| n).collect()
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

//     /// Identify the most recent ancestor that is a recombinant.
//     pub fn get_recombinant_ancestor(&self, node: &T) -> Result<Option<&T>, Report> {
//         let mut recombinant = None;

//         let root = self.get_root_data()?;
//         let ancestor_paths = self.get_paths(node, root, petgraph::Incoming)?;

//         for path in ancestor_paths {
//             for node in path {
//                 if self.recombinants.contains(&node) {
//                     recombinant = Some(node);
//                     break;
//                 }
//             }
//             if recombinant.is_some() {
//                 break;
//             }
//         }

//         Ok(recombinant)
//     }

//     /// Get root node index
//     pub fn get_root_index(&self) -> &NodeIndex {
//        &self.graph.node_indices().next().unwrap_or_default()
//     }

//     pub fn get_root_data(&self) -> Result<&T, Report> {
//         let node_index = self.graph.node_indices().next().unwrap_or_default();
//         let node_data = self.get_node_data(&node_index)?;
//         Ok(node_data)
//     }

//     /// Get immediate parents of a node.
//     pub fn get_parents(&self, node: &T) -> Result<Vec<&T>, Report> {
//         let node_index = self.get_node_index(node)?;
//         let mut neighbors = self.graph.neighbors_directed(*node_index, Direction::Incoming);
//         let parents = neighbors.into_iter().map(|node_index| {
//             let node_data = self.get_node_data(&node_index)?;
//             Ok(node_data)
//         }).collect::<Result<Vec<&T>, Report>>()?;

//         Ok(parents)
//     }

//     /// Get all paths from the origin node to the destination node, always traveling
//     /// in the specified direction (Incoming towards root, Outgoing towards tips)
//     pub fn get_paths(
//         &self,
//         origin: &T,
//         dest: &T,
//         direction: petgraph::Direction,
//     ) -> Result<Vec<Vec<&T>>, Report> {
//         // container to hold the paths we've found, is a vector of vectors
//         // because there might be recombinants with multiple paths
//         let mut paths = Vec::new();

//         // check that the origin and dest actually exist in the graph
//         let origin_node_index = self.get_node_index(origin)?;
//         let dest_node_index = self.get_node_index(dest)?;

//         // Check if we've reached the destination
//         if origin == dest {
//             paths.push(vec![origin]);
//         }
//         // Otherwise, continue the search!
//         else {
//             let mut neighbors = self.graph.neighbors_directed(*origin_node_index, direction);
//             neighbors.into_iter().try_for_each(|node_index| {
//                 let parent_node_data = self.get_node_data(&node_index)?;

//                 // recursively get path of each parent to the destination
//                 let mut parent_paths = self.get_paths(&parent_node_data, dest, direction)?;

//                 // prepend the origin to the paths
//                 parent_paths.iter_mut().for_each(|p| {
//                     p.insert(0, origin);
//                     paths.push(*p);
//                 });

//                 Ok::<(), Report>(())
//             })?;
//         }

//         Ok(paths)
//     }

//     /// Remove a node in the graph.
//     ///
//     /// If prune is true, removes entire clade from graph.
//     /// If prune is false, connects parents to children to fill the hole.
//     pub fn remove(&mut self, node: &T, prune: bool) -> Result<Vec<T>, Report> {

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
// }

// /// Phylogenetic methods for data types that can be serialized.
// impl<'a, T> Phylogeny<T>
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
