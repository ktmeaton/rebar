use crate::sequence;
use color_eyre::eyre::{Report, Result};
use indoc::formatdoc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
// Population Parsimony Summary

/// Summarize support and conflicts between two sequences.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Summary<'primary, 'secondary> {
    #[serde(skip_deserializing)]
    pub support: &'primary [&'primary sequence::Substitution],
    #[serde(skip_deserializing)]
    pub conflict_ref: &'secondary [&'secondary sequence::Substitution],
    #[serde(skip_deserializing)]
    pub conflict_alt: &'primary [&'primary sequence::Substitution],
    #[serde(skip_deserializing)]
    pub private: &'primary [&'primary sequence::Substitution],
}

impl<'primary, 'secondary> Summary<'primary, 'secondary> {
    pub fn new() -> Self {
        Summary {
            support: &[],
            conflict_ref: &[],
            conflict_alt: &[],
            private: &[],
        }
    }

    /// Calculate parsimony score = support - conflict_alt - conflict_ref
    pub fn score(&self) -> isize {
        // why did we previously use only conflict_ref and not conflict_alt?
        // individual isize conversion otherwise: "attempt to subtract with overflow"
        self.support.len() as isize
            - self.conflict_ref.len() as isize
            - self.conflict_alt.len() as isize
    }

    /// Summarize support and conflicts between two sequence records.
    pub fn from_records(
        primary: &sequence::Record,
        secondary: &sequence::Record,
        coordinates: Option<&[usize]>,
    ) -> Result<Self, Report> {
        let mut summary = Summary::new();

        // get all the substitutions found in the primary sequence
        let mut primary_subs = &primary.substitutions;

        // exclude coordinates that are in the primary sequence missing or deletions
        let mut exclude_coordinates = primary.deletions.iter().map(|d| d.coord).collect_vec();
        exclude_coordinates.extend(primary.missing);
        // get all the substitutions found in the secondary sequence
        // exclude missing and deletion coordinates
        let mut secondary_subs = &secondary.substitutions;
        secondary_subs.retain(|s| !exclude_coordinates.contains(&s.coord));

        // optionally filter coordinates
        if let Some(coordinates) = coordinates {
            secondary_subs.retain(|sub| coordinates.contains(&sub.coord));
            primary_subs.retain(|sub| coordinates.contains(&sub.coord));
        }

        // support: sub in primary that is also in secondary
        // conflict_alt: sub in primary that is not in secondary
        primary_subs.iter().for_each(|sub| match secondary_subs.contains(sub) {
            true => summary.support.push(sub),
            false => summary.conflict_alt.push(sub),
        });

        // conflict_ref: sub in secondary that is not in primary
        summary.conflict_ref = secondary_subs.iter().filter(|sub| !primary_subs.contains(sub)).collect();

        // private subs (conflict_alt and conflict_ref reversed)
        let mut private = summary.conflict_alt.to_vec();
        private = summary.conflict_ref.iter().map(|s| {
            sequence::Substitution {coord: s.coord, reference: s.alt, alt: s.reference}
        }).collect();
        private.sort();
        summary.private = &private;

        Ok(summary)
    }

    pub fn pretty_print(&self) -> String {
        formatdoc!(
            "score:\n  {}
            support:\n  {}
            conflict_ref:\n  {}
            conflict_alt:\n  {},
            private:\n  {}",            
            self.score(),
            self.support.iter().join(", "),
            self.conflict_ref.iter().join(", "),
            self.conflict_alt.iter().join(", "),
            self.private.iter().join(", "),            
        )
    }
}

impl<'primary, 'secondary> Default for Summary<'primary, 'secondary> {
    fn default() -> Self {
        Self::new()
    }
}
