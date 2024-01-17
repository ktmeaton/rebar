use color_eyre::eyre::{eyre, Report, Result};
use crate::FromNewick;
use std::fmt::Display;

/// Returns a vector of nodes (`N`) and branches (`B`) from an input Newick string.
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
/// use rebar_phylo::{Phylogeny, Node, Branch, newick};
/// let nwk = "(A,B);";
/// let v: Vec<(Node<String>, Node<String>, Branch)>  = newick::str_to_vec(&nwk, None, 0)?;
/// let phylo  = Phylogeny::from(v);
/// # use rebar_phylo::FromNewick;
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
///
/// From a Newick string with tip names, internal node names, branch lengths, and confidence.
///
/// ```rust
/// use rebar_phylo::{newick, Node, Branch, Phylogeny};
/// let nwk = "(A:0.1:90,B:0.2,(C:0.3,D:0.4)E:0.5)F;";
/// let v: Vec<(Node<String>, Node<String>, Branch)>  = newick::str_to_vec(&nwk, None, 0)?;
/// let phylo  = Phylogeny::from(v);
/// # use rebar_phylo::FromNewick;
/// # let nodes    = ["NODE_0", "F", "A", "B", "E", "C", "D"].map(|n| Node::from_newick(n).unwrap());
/// # let branches = [":0.1:90", ":0.2", ":0.3", ":0.4", ":0.5", ":0.0"].map(|n| Branch::from_newick(&format!("{n}")).unwrap());
/// # assert_eq!(phylo.get_nodes()?,    nodes.iter().collect::<Vec<_>>());
/// # assert_eq!(phylo.get_branches()?, branches.iter().collect::<Vec<_>>());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
///
/// From an [Extended Newick](https://en.wikipedia.org/wiki/Newick_format#Extended_Newick) string.
///
/// ```rust
/// use rebar_phylo::{newick, Node, Branch, Phylogeny};
/// let nwk = "(A,B,((C,(Y)x#H1)c,(x#H1,D)d)e)f";
/// let v: Vec<(Node<String>, Node<String>, Branch)>  = newick::str_to_vec(&nwk, None, 0)?;
/// let phylo  = Phylogeny::from(v);
/// # use rebar_phylo::FromNewick;
/// # let nodes    = ["NODE_0", "f", "A", "B", "e", "c", "C", "x#H1", "Y", "d", "D"].map(|n| Node::from_newick(n).unwrap());
/// # let branches = (1..=11).map(|n| Branch::from_newick(&format!(":0.0")).unwrap()).collect::<Vec<_>>();
/// # assert_eq!(phylo.get_nodes()?,    nodes.iter().collect::<Vec<_>>());
/// # assert_eq!(phylo.get_branches()?, branches.iter().collect::<Vec<_>>());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn str_to_vec<N, B>(
    newick: &str,
    parent: Option<N>,
    mut node_i: usize,
) -> Result<Vec<(N, N, B)>, Report>
where
    N: Clone + Default + Display + FromNewick,
    B: Display + FromNewick,
{
    // strip ';' characters
    let newick = newick.replace(';', "");

    // --------------------------------------------------------------------
    // Case 1: No more parentheses, recursion has bottomed out
    if !newick.contains('(') && !newick.contains(')') {
        newick
            // iterate over nodes separated by a comman (,)
            .split(',')
            .filter(|n| !n.is_empty())
            // convert strings into [Node] and [Branch] types
            .map(|n| {
                let parent: N = match &parent {
                    Some(node) => node.to_owned(),
                    None => N::from_newick(&format!("NODE_{node_i}"))?,
                };
                let branch = B::from_newick(n)?;
                let node = N::from_newick(n)?;
                //println!("parent: {parent}, node: {node}, branch: {branch}, newick: {newick}");
                Ok((parent, node, branch))
            })
            // collect final vec of links, handle any errors from ? operator
            .collect::<Result<Vec<_>, Report>>()
    }
    // --------------------------------------------------------------------
    // Case 2: Parentheses found, continue recursion
    else {
        // extract the content inside the first level of parentheses
        let (inner_start, inner_end) = get_inside_parentheses(&newick)?;
        let inner = &newick[inner_start..=inner_end];

        // extract content before the first level of parentheses, could be sister taxa
        let before = &newick[..inner_start - 1];
        // extract content after the first level of parentheses, could be parent and sister taxa
        let after = &newick[inner_end + 2..];

        // extract the parent of the inner content, parent is found in the after content
        let inner_parent = match after.is_empty() {
            true => {
                let node = N::from_newick(&format!("NODE_{node_i}"))?;
                node_i += 1;
                node
            }
            false => {
                // once we hit a ',' or ')', parent newick ends
                let parent_end =
                    after.chars().position(|c| c == ',' || c == ')').unwrap_or(after.len());
                N::from_newick(&after[..parent_end])?
            }
        };

        // process the newick content recursively (before, inner, after)
        let data: Vec<_> = [before, inner, after]
            .into_iter()
            .zip([parent.clone(), Some(inner_parent), parent])
            .filter(|(nwk, _parent)| !nwk.is_empty())
            .map(|(nwk, parent)| {
                let data = str_to_vec(nwk, parent, node_i)?;
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
        _ => Err(eyre!(
            "Failed to find matching outer parentheses from newick: {newick}"
        ))?,
    }
}
