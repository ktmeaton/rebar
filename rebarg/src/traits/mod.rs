use color_eyre::eyre::{eyre, Result, Report};
use crate::{Branch, Node, Phylogeny};

/// Create a [`Phylogeny`] of [`Nodes`](Node) and [`Branches`](Branch) from Newick format.
pub trait FromNewick<N, B> {
    fn from_newick(path: &std::path::Path) -> Result<Self, Report> {
        // read file to string
        let newick = std::fs::read_to_string(path)?;
        // parse newick string into vector of nodes and branches
        let data = Phylogeny::<Node<String>, Branch>::newick_str_to_vec(&newick, None, 0)?;
        // create phylogeny from vector
        let phylo = Phylogeny::from_vec(data)?;

        Ok(Self)
    }

    /// Returns a [`Phylogeny`] created from a [Newick](https://en.wikipedia.org/wiki/Newick_format) string.
    ///
    /// # Arguments
    ///
    /// - `newick` - A Newick [`str`] (ex. `"(A,B);"`)
    ///
    /// # Examples
    ///
    /// A Newick [`str`] with only node names.
    ///
    /// ```rust
    /// use rebarg::{Phylogeny, FromNewick};
    /// let newick = "(A,B);";
    /// let phylo = Phylogeny::from_newick_str(&newick)?;
    ///
    /// # use rebarg::{Node, Branch};
    /// # use std::str::FromStr;
    /// # assert_eq!(phylo.get_nodes()?,   vec![Node::from_str("A")?, Node::from_str("B")?].iter().collect::<Vec<_>>());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    ///
    ///
    fn from_newick_str(newick: &str) -> Result<Phylogeny<Node<String>, Branch>, Report> {
        let data = Phylogeny::<Node<String>, Branch>::newick_str_to_vec(newick, None, 0)?;
        let phylo = Phylogeny::from_vec(data)?;
        Ok(phylo)
    }

    /// Returns a vector of nodes (`N`) and branches (`B`) from an input Newick string.
    ///
    /// This is intended to be an internal helper function, as an intermediate between parsing a Newick and creating a [Phylogeny].
    ///
    /// # Arguments
    ///
    /// - `newick` - A Newick [`str`] (ex. `"(A,B);"`)
    /// - `parent` - The parent node (`N`) during recursion. Set to `None` on initial function call.
    ///
    /// # Examples
    ///
    /// From a Newick string with only tip names.
    /// 
    /// ```rust
    /// let data   = Phylogeny::newick_str_to_vec(&"(A,B);", None, 0)?;
    /// let phylo  = Phylogeny::from_vec(data)?;
    /// # use rebarg::{Node, Branch};
    /// # let nodes = vec![Node::from_newick_str("NODE_0")?, Node::from_newick_str("A")?, Node::from_newick_str("B")?];
    /// # let branches = vec![Branch { length: 0.0, confidence: 0.0 }, Branch { length: 0.0, confidence: 0.0 }];
    /// # assert_eq!(phylo.get_nodes()?,   nodes.iter().collect::<Vec<_>>());
    /// # assert_eq!(phylo.get_branches(), branches.iter().collect::<Vec<_>>());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    /// 
    /// ```rust
    /// use rebarg::{Phylogeny, FromNewick};
    /// //let newick = "((A.1,A.2)A,B,(C.1,C.2)C)Root;";
    /// let newick = "((A.1)A,(B.1)B)C;";
    /// let newick = "(A,B,(C,D));";
    /// let newick = "(((A.1.1)A.1)A,B);";
    /// ```
    fn newick_str_to_vec(
        newick: &str,
        parent: Option<Node<String>>,
        mut node_i: usize,
    ) -> Result<Vec<(Node<String>, Node<String>, Branch)>, Report> {
        // strip ';' characters
        let newick = newick.replace(';', "");

        // --------------------------------------------------------------------
        // Case 1: No more parentheses, recursion has bottomed out
        if !newick.contains("(") && !newick.contains(")") {
            newick
                // iterate over nodes separated by a comman (,)
                .split(",")
                // convert strings into [Node] and [Branch] types
                .map(|n| {
                    let parent = match &parent {
                        Some(node) => node.clone(),
                        None => Node { label: format!("NODE_{node_i}") },
                    };
                    let branch = Branch::from_newick_str(n)?;
                    let node = Node::from_newick_str(n)?;
                    Ok((parent, node, branch))
                })
                // collect final vec of links, handle any errors from ? operator
                .collect::<Result<Vec<_>, Report>>()
        }
        // --------------------------------------------------------------------
        // Case 2: Parentheses found, continue recursion
        else {
            // extract the content inside the first level of parentheses
            let (inner_start, inner_end) = Phylogeny::get_inside_parentheses(&newick)?;
            let inner = &newick[inner_start..=inner_end];

            // extract content before the first level of parentheses, could be sister taxa
            let before = &newick[..inner_start - 1];
            // extract content after the first level of parentheses, could be parent and sister taxa
            let after = &newick[inner_end + 2..];

            // extract the parent of the inner content, parent is found in the after content
            let parent_nwk = match after.is_empty() {
                true => "",
                false => {
                    // once we hit a ',' or ')', parent newick ends
                    let parent_end =
                        after.chars().position(|c| c == ',' || c == ')').unwrap_or(after.len());
                    &after[..parent_end]
                }
            };

            let mut inner_parent = Node::from_newick_str(after)?;
            // if the inner parent doesn't have a label, set to numeric node identifier
            if inner_parent.label.is_empty() {
                inner_parent.label = format!("NODE_{node_i}");
                node_i += 1;
            }

            // process the newick content recursively (before, inner, after)
            let data: Vec<_> = [before, inner, after]
                .into_iter()
                .zip([parent.clone(), Some(inner_parent), parent])
                .filter(|(nwk, _parent)| !nwk.is_empty())
                .map(|(nwk, parent)| {
                    let data = Phylogeny::newick_str_to_vec(&nwk, parent, node_i)?;
                    Ok(data)
                })
                // parse the result layer, handling errors
                .collect::<Result<Vec<_>, Report>>()
                // parse and flatten the data layer
                .into_iter()
                .flatten()
                .flatten()
                .collect();
            Ok(data)
        }
    }

    /// Get indices of the content inside the first set of parentheses.
    fn get_inside_parentheses(newick: &str) -> Result<(usize, usize), Report> {
        let mut start: Option<usize> = None;
        let mut end: Option<usize> = None;
        let (mut num_open, mut num_close) = (0, 0);

        for (i, c) in newick.chars().enumerate() {
            if c == '(' {
                if start.is_none() {
                    start = Some(i + 1);
                }
                num_open += 1;
            } else if c == ')' {
                num_close += 1;
                if num_open == num_close {
                    end = Some(i - 1);
                    break;
                }
            }
        }
        match (start, end) {
            (Some(s), Some(e)) => Ok((s, e)),
            _ => Err(eyre!("Failed to find matching outer parentheses from newick: {newick}"))?,
        }
    }
}