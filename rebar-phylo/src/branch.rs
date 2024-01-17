use crate::FromNewick;

use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fmt::{Display, Formatter};

/// A [`Branch`] in the [`Phylogeny`](crate::Phylogeny).
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Branch {
    /// [`Branch`] length (ex. 1.0).
    pub length: f32,
    /// [`Branch`] confidence (ex. 90.0).
    pub confidence: f32,
}

#[rustfmt::skip]
impl AsPrimitive<f32> for Branch { fn as_(self) -> f32 { self.length } }
#[rustfmt::skip]
impl Default for Branch { fn default() -> Self { Self::new() } }
#[rustfmt::skip]
impl Display for Branch { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.length) } }
#[rustfmt::skip]
impl Branch { pub fn new() -> Self { Branch { length: 0.0, confidence: 0.0 } } }

impl FromNewick for Branch {
    /// Returns a branch (`B`) created from a [Newick](https://en.wikipedia.org/wiki/Newick_format) node [`str`].
    ///
    /// # Examples
    ///
    /// Just a node name.
    ///
    /// ```rust
    /// use rebar_phylo::{Branch, FromNewick};
    ///
    /// let branch = Branch::from_newick(&"A")?;
    /// assert_eq!(branch, Branch { length: 0.0, confidence: 0.0 });
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// Just branch attributes.
    ///
    /// ```rust
    /// # use rebar_phylo::{Branch, FromNewick};
    /// let branch = Branch::from_newick(&":2:90")?;
    /// assert_eq!(branch, Branch { length: 2.0, confidence: 90.0 });
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// Branch confidence as a decimal.
    ///
    /// ```rust
    /// # use rebar_phylo::{Branch, FromNewick};
    /// let branch = Branch::from_newick(&":2:0.75")?;
    /// assert_eq!(branch, Branch { length: 2.0, confidence: 75.0 });
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    fn from_newick(newick: &str) -> Result<Branch, Report> {
        let attributes: Vec<_> = newick.replace(';', "").split(':').map(String::from).collect();
        let length = match attributes.len() >= 2 {
            true => attributes[1]
                .parse()
                .wrap_err_with(|| eyre!("Failed to parse branch length from newick: {newick}"))?,
            false => 0.0,
        };
        let confidence = match attributes.len() >= 3 {
            true => {
                let confidence = attributes[2]
                    .parse()
                    .wrap_err_with(|| eyre!("Failed to parse confidence from newick: {newick}"))?;
                // if confidence is a decimal, multiple by 100
                match confidence < 1.0 {
                    true => confidence * 100.0,
                    false => confidence,
                }
            }
            false => 0.0,
        };

        Ok(Branch { length, confidence })
    }
}
