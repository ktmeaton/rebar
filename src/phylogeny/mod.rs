use crate::utils;
use color_eyre::eyre::{eyre, ContextCompat, Report, Result, WrapErr};
use color_eyre::Help;
use itertools::Itertools;
use log::debug;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::{Dfs, IntoNodeReferences};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// ----------------------------------------------------------------------------
// Phylogeny

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Phylogeny<'graph> {
    pub graph: Graph<String, isize>,
    // we will parse recombinants on load/read
    //#[serde(skip_serializing, skip_deserializing)]
    pub recombinants: &'graph [&'graph str],
    // recombinants_all includes descendants of recombinant nodes
    //#[serde(skip_serializing, skip_deserializing)]
    pub recombinants_all: &'graph [&'graph str],
}

impl<'graph> Default for Phylogeny<'graph> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'graph> Phylogeny<'graph> {
    pub fn new() -> Self {
        Phylogeny {
            graph: Graph::new(),
            recombinants: &[],
            recombinants_all: &[],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Return true if a node name is a recombinant.
    ///
    /// Checks the number of incoming edges to a node, as recombinants
    /// will have more than 1 (ie. more than 1 parent).
    pub fn is_recombinant(&self, name: &str) -> Result<bool, Report> {
        let node = self.get_node(name)?;
        let mut edges = self.graph.neighbors_directed(node, Direction::Incoming).detach();

        let mut num_edges = 0;
        while let Some(_edge) = edges.next_edge(&self.graph) {
            num_edges += 1;
            // early return, just in case it helps
            if num_edges > 1 {
                return Ok(true);
            }
        }
        Ok(num_edges > 1)
    }

    /// Get populations names (all named nodes)
    pub fn get_names(&self) -> &[&str] {
        self.graph
            .node_references()
            .into_iter()
            .filter(|(_i, n)| !n.is_empty())
            .map(|(_i, n)| n.as_str())
            .unique()
            .collect()
    }

    /// Get recombinant node names.
    ///
    /// descendants will decide if recombinant descendants should be included
    pub fn get_recombinants(&self, descendants: bool) -> &[&str] {
        match descendants {
            // include all descendants of recombinant nodes
            true => self
                .get_names()
                .into_iter()
                .filter(|n| {
                    let mut is_recombinant = false;
                    if let Ok(result) = self.get_recombinant_ancestor(&n) {
                        if let Some(recombinant) = result {
                            is_recombinant = true;
                        }
                    }
                    is_recombinant
                })
                .unique()
                .collect_vec(),
            // include only the primary recombinant nodes
            false => self
                .get_names()
                .into_iter()
                .filter(|n| self.is_recombinant(n).unwrap_or(false))
                .unique()
                .collect_vec(),
        }
    }

    /// Get non-recombinants
    pub fn get_non_recombinants_all(&self) -> &[&str] {
        self.get_names().into_iter().filter(|n| !self.recombinants_all.contains(n)).collect()
    }

    /// Remove a single named node in the graph.
    ///
    /// Connect parents to children to fill the hole.
    pub fn remove(&mut self, name: &str) -> Result<(), Report> {
        // Delete the node
        debug!("Removing node: {name}");

        let node = self.get_node(name)?;

        // get some attributes before we remove it
        let parents = self.get_parents(name)?;
        let mut children = self.get_children(name)?;
        let is_recombinant = self.is_recombinant(name)?;

        // Delete the node
        self.graph.remove_node(node).unwrap_or_default();

        // If it was an interior node, connect parents and child
        children.iter().for_each(|c| {
            let c_node = self.get_node(c).expect("Child {c} is not in graph.");
            //debug!("Connecting child {c} to new parent(s): {parents:?}");
            parents.iter().for_each(|p| {
                let p_node = self.get_node(p).expect("Parent {p} is not in graph.");
                self.graph.add_edge(p_node, c_node, 1);
            })
        });

        // If it was a primary recombinant node, make all children primary recombinants
        if is_recombinant {
            self.recombinants.append(&mut children);
        }

        // Update the recombinants attributes
        self.recombinants.retain(|n| *n != name);
        self.recombinants_all.retain(|n| *n != name);
        Ok(())
    }

    /// Prune a clade from the graph.
    ///
    /// Removes named node and all descendants.
    pub fn prune(&mut self, name: &str) -> Result<&[&str], Report> {
        let recombination = true;
        let descendants = self.get_descendants(name, recombination)?;
        for d in descendants {
            self.remove(&d)?;
        }

        Ok(&descendants[..])
    }

    /// Read phylogeny from file.
    pub fn read(path: &Path) -> Result<Phylogeny, Report> {
        let phylogeny = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read file: {path:?}."))?;
        let mut phylogeny: Phylogeny = serde_json::from_str(&phylogeny)
            .wrap_err_with(|| format!("Failed to parse file: {path:?}."))?;

        phylogeny.recombinants = phylogeny.get_recombinants(false);
        phylogeny.recombinants_all = phylogeny.get_recombinants(true);

        Ok(phylogeny)
    }

    /// Write phylogeny to file.
    pub fn write(&self, path: &Path) -> Result<(), Report> {
        // Create output file
        let mut file = File::create(path)?;
        // Check format based on extension
        let ext = utils::path_to_ext(Path::new(path))?;

        // format conversion
        let output = match ext.as_str() {
            // ----------------------------------------------------------------
            // DOT file for graphviz
            "dot" => {
                let mut output =
                    format!("{}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));
                // set graph id (for cytoscape)
                output = str::replace(&output, "digraph", "digraph G");
                // set horizontal (Left to Right) format for tree-like visualizer
                output = str::replace(&output, "digraph {", "digraph {\n    rankdir=\"LR\";");
                output
            }
            // ----------------------------------------------------------------
            // JSON for rebar
            "json" => serde_json::to_string_pretty(&self)
                .unwrap_or_else(|_| panic!("Failed to parse: {self:?}")),
            _ => {
                return Err(
                    eyre!("Phylogeny write for extension .{ext} is not supported.")
                        .suggestion("Please try .json or .dot instead."),
                )
            }
        };

        // Write to file
        file.write_all(output.as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write file: {:?}.", path));

        Ok(())
    }

    // Get all descendants of a population.
    //
    // Returns a big pile (single vector) of all descendants in all paths to tips.
    // Reminder, this function will also include name (the parent)
    pub fn get_descendants(&self, name: &str, recombination: bool) -> Result<&[&str], Report> {
        let mut descendants = Vec::new();

        // Find the node that matches the name
        let node = self.get_node(name)?;
        // Construct a depth-first-search (Dfs)
        let mut dfs = Dfs::new(&self.graph, node);

        // Skip over self?
        // dfs.next(&self.graph);
        // Iterate over descendants
        while let Some(nx) = dfs.next(&self.graph) {
            // Get node name
            let nx_name = self.get_name(&nx)?;
            descendants.push(nx_name);
        }

        // exclude descendants that are novel recombinants
        if !recombination {
            let recombinant_ancestor = self.get_recombinant_ancestor(name).ok();
            descendants = descendants
                .into_iter()
                .filter(|desc| recombinant_ancestor == self.get_recombinant_ancestor(desc).ok())
                .collect_vec();
        }

        Ok(&descendants[..])
    }

    /// Get parent names of node
    pub fn get_parents(&self, name: &str) -> Result<&[&str], Report> {
        let mut parents = Vec::new();

        let node = self.get_node(name)?;
        let mut neighbors = self.graph.neighbors_directed(node, Direction::Incoming).detach();
        while let Some(parent_node) = neighbors.next_node(&self.graph) {
            let parent_name = self.get_name(&parent_node)?;
            parents.push(parent_name);
        }

        Ok(&parents[..])
    }

    /// Get children names of node
    pub fn get_children(&self, name: &str) -> Result<&[&str], Report> {
        let mut children = Vec::new();

        let node = self.get_node(name)?;
        let mut neighbors = self.graph.neighbors_directed(node, Direction::Outgoing).detach();
        while let Some(child_node) = neighbors.next_node(&self.graph) {
            let child_name = self.get_name(&child_node)?;
            children.push(child_name);
        }

        // children order is last added to first added, reverse this
        children.reverse();

        Ok(&children[..])
    }

    /// Get problematic recombinants, where the parents are not sister taxa.
    /// They might be parent-child instead.
    pub fn get_problematic_recombinants(&self) -> Result<&[&str], Report> {
        let mut problematic_recombinants = Vec::new();
        let recombination = true;

        for recombinant in &self.recombinants {
            let parents = self.get_parents(recombinant)?;
            for i1 in 0..parents.len() - 1 {
                let p1 = &parents[i1];
                for p2 in parents.iter().skip(i1 + 1) {
                    let mut descendants = self.get_descendants(p2, recombination)?;
                    let ancestors = self.get_ancestors(p2)?.into_iter().flatten().collect_vec();
                    descendants.extend(ancestors);

                    if descendants.contains(p1) {
                        problematic_recombinants.push(recombinant.clone());
                        break;
                    }
                }
            }
        }

        Ok(&problematic_recombinants[..])
    }

    /// Get all paths from the origin node to the destination node, always traveling
    /// in the specified direction (Incoming towards root, Outgoing towards tips)/
    /// petgraph must have this already implemented, but I can't find it in docs
    pub fn get_paths(
        &self,
        origin: &str,
        dest: &str,
        direction: petgraph::Direction,
    ) -> Result<&[&[&str]], Report> {
        // container to hold the paths we've found, is a vector of vectors
        // because there might be recombinants with multiple paths
        let mut paths: Vec<Vec<&str>> = Vec::new();

        // check that the origin and dest actually exist in the graph
        let origin_node = self.get_node(origin)?;
        let _dest_node = self.get_node(dest)?;

        // Check if we've reached the destination
        if origin == dest {
            paths.push(vec![origin]);
        }
        // Otherwise, continue the search!
        else {
            let mut neighbors = self.graph.neighbors_directed(origin_node, direction).detach();
            while let Some(parent_node) = neighbors.next_node(&self.graph) {
                // convert the parent graph index to a string name
                let parent_name = self.get_name(&parent_node)?;

                // recursively get path of each parent to the destination
                let mut parent_paths = self.get_paths(&parent_name, dest, direction)?;

                // prepend the origin to the paths
                parent_paths.iter_mut().for_each(|p| p.insert(0, origin));

                // update the paths container to return at end of function
                for p in parent_paths {
                    paths.push(p);
                }
            }
        }

        Ok(paths)
    }

    /// NOTE: Don't think this will work with 3+ parents yet, to be tested.
    pub fn get_ancestors(&self, name: &str) -> Result<&[&[&str]], Report> {
        let mut paths = self.get_paths(name, "root", petgraph::Incoming)?;

        // remove self name (first element) from paths, and then reverse order
        // so that it's ['root'.... name]
        paths.iter_mut().for_each(|p| {
            p.remove(0);
            p.reverse();
        });

        Ok(paths)
    }

    /// Identify the most recent common ancestor shared between all node names.
    pub fn get_common_ancestor(&self, names: &[&str]) -> Result<&str, Report> {
        // if only one node name was provided, just return it
        if names.len() == 1 {
            return Ok(names[0]);
        }

        // mass pile of all ancestors of all named nodes
        let ancestors: Vec<_> = names
            .iter()
            .map(|pop| {
                let paths = self.get_paths(pop, "root", Direction::Incoming)?;
                let ancestors = paths.into_iter().flatten().unique().collect_vec();
                debug!("{pop}: {ancestors:?}");
                Ok(ancestors)
            })
            .collect::<Result<Vec<_>, Report>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // get ancestors shared by all sequences
        let common_ancestors: Vec<_> = ancestors
            .iter()
            .unique()
            .filter(|anc| {
                let count = ancestors.iter().filter(|pop| pop == anc).count();
                count == names.len()
            })
            .collect();

        debug!("common_ancestors: {common_ancestors:?}");

        // get the depths (distance to root) of the common ancestors
        let depths = common_ancestors
            .into_iter()
            .map(|pop| {
                let paths = self.get_paths(pop, "root", Direction::Incoming)?;
                let longest_path =
                    paths.into_iter().max_by(|a, b| a.len().cmp(&b.len())).unwrap_or_default();
                let depth = longest_path.len();
                debug!("{pop}: {depth}");
                Ok((pop, depth))
            })
            .collect::<Result<Vec<_>, Report>>()?;

        // get the deepest (ie. most recent common ancestor)
        let deepest_ancestor = depths
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .context("Failed to get common ancestor.")?;

        // tuple (population name, depth)
        let common_ancestor = deepest_ancestor.0;

        Ok(common_ancestor)
    }

    /// Identify the most recent ancestor that is a recombinant.
    pub fn get_recombinant_ancestor(&self, name: &str) -> Result<Option<&str>, Report> {
        let mut recombinant = None;

        let ancestor_paths = self.get_paths(name, "root", petgraph::Incoming)?;

        for path in ancestor_paths {
            for name in path {
                if self.recombinants.contains(&name) {
                    recombinant = Some(name);
                    break;
                }
            }
            if recombinant.is_some() {
                break;
            }
        }

        Ok(recombinant)
    }

    /// Get the node index of a named node.
    pub fn get_node(&self, name: &str) -> Result<&NodeIndex, Report> {
        self.graph
            .node_references()
            .into_iter()
            .filter_map(|(i, n)| (n == name).then_some(i))
            .next()
            .ok_or(Err(eyre!("Name {name:?} is not in the phylogeny."))?)
    }

    /// Get the node name from a node index.
    pub fn get_name(&self, node: &NodeIndex) -> Result<&str, Report> {
        self.graph
            .node_references()
            .into_iter()
            .filter_map(|(i, n)| (i == *node).then_some(n.as_str()))
            .next()
            .ok_or(Err(eyre!("Node {node:?} is not in the phylogeny."))?)
    }
}
