pub mod attributes;
pub mod download;
pub mod list;
pub mod load;
pub mod sarscov2;
pub mod toy1;

use crate::cli::run;
use crate::phylogeny::Phylogeny;
use crate::{sequence, sequence::parsimony};
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use color_eyre::Help;
use indoc::formatdoc;
use itertools::Itertools;
use log::debug;
use noodles::fasta;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// ----------------------------------------------------------------------------
// Dataset

/// A dataset is a collection of named sequences, which have been aligned to a reference.
///
/// The dataset will persist for the entire runtime of the program. It must own all it's own data.
#[derive(Debug, Deserialize, Serialize)]
pub struct Dataset<'phylo, 'pop> {
    /// Dataset metadata summary
    pub summary: attributes::Summary,
    /// Reference sequence record, with sequence bases kept
    pub reference: sequence::Record,
    /// Dataset populations, map of names to sequences.
    /// This is the primary data that the dataset 'owns'
    pub populations: BTreeMap<String, sequence::Record>,
    /// Dataset mutations, map of substitutions to named sequences.
    /// References data owned by dataset.populations
    #[serde(skip_deserializing)]
    pub mutations: BTreeMap<&'pop sequence::Substitution, Vec<&'pop str>>,
    /// Phylogenetic representation, as an ancestral recombination graph (ARG)
    pub phylogeny: Phylogeny<'phylo>,
    /// Edge cases of problematic populations
    pub edge_cases: Vec<run::Args>,
}

impl<'phylo, 'pop> fmt::Display for Dataset<'phylo, 'pop> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {}, tag: {}", self.summary.name, self.summary.tag)
    }
}

impl<'phylo, 'pop> Default for Dataset<'phylo, 'pop> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'phylo, 'pop> Dataset<'phylo, 'pop> {
    /// Create a new dataset.
    pub fn new() -> Self {
        Dataset {
            summary: attributes::Summary::new(),
            reference: sequence::Record::new(),
            populations: BTreeMap::new(),
            mutations: BTreeMap::new(),
            phylogeny: Phylogeny::new(),
            edge_cases: Vec::new(),
        }
    }

    /// Create a consensus sequence from populations.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the output sequence ID.
    /// * `populations` - Dataset population to use as sequences.
    /// * `discard_sequence` - True if the actual sequence bases should not be saved. Good for memory limiting.
    ///
    pub fn create_consensus(
        &self,
        name: &str,
        populations: &[&str],
        discard_sequence: bool,
    ) -> Result<sequence::Record, Report> {
        // collect individual population sequences
        let sequences = populations
            .iter()
            .filter_map(|pop| {
                (self.populations.contains_key(*pop)).then_some(&self.populations[*pop])
            })
            .collect_vec();

        // construct consensus
        let consensus = (0..self.reference.genome_length)
            .map(|coord| {

                let bases = sequences.iter().map(|s| s.sequence[coord]).unique().collect_vec();
                if bases.len() == 1 {
                    bases[0]
                } else {
                    'N'
                }
            })
            .join("");

        // create new fasta record
        let definition = fasta::record::Definition::new(name, None);
        let sequence = fasta::record::Sequence::from(consensus.as_bytes().to_vec());
        let record = fasta::Record::new(definition, sequence);

        // parse and create Sequence record
        // dataset is already masked, no need
        let mask = Vec::new();
        let sequence =
            sequence::Record::from_fasta(record, Some(&self.reference), &mask, discard_sequence)?;

        Ok(sequence)
    }

    /// Expand list of populations with wildcarding.
    /// * for all descendants (including novel recombination)
    /// *-r for all descendants (excluding novel recombination)
    ///
    /// Returns only population names that have sequence data.
    pub fn expand_populations(&self, populations: &[&str]) -> Result<&[&str], Report> {
        // expand wildcards ("*" and "*-r")
        let expanded = populations
            .into_iter()
            .map(|p| {
                // remove wildcarding suffixes, to get the name of the parent
                let strict = p.replace("*-r", "").replace('*', "");
                // decide whether recombination is allowed for descendants
                let recombination = p.ends_with('*');

                // "" = root: "X" = all recombinants, other treat as plain name
                let populations = match strict.as_str() {
                    "" => self.populations.keys().map(|p| p.as_str()).collect(),
                    "X" => match recombination {
                        true => self.phylogeny.recombinants_all.to_vec(),
                        false => self.phylogeny.recombinants.to_vec(),
                    },
                    _ => self.phylogeny.get_descendants(&strict, recombination)?.to_vec(),
                };
                Ok(populations)
            })
            // handle the `Result` layer, stop and throw errors if needed
            .collect::<Result<Vec<_>, Report>>()?
            .into_iter()
            // handle the `Vec` layer, flatten and dedup
            .flatten()
            .unique()
            // restrict to populations with sequence data
            .filter(|p| self.populations.contains_key(*p))
            .collect_vec();

        if expanded.is_empty() {
            return Err(eyre!("Populations {populations:?} could not be expanded.")
                .suggestion("Is there a typo?")
                .suggestion("If not, perhaps this population has no sequence data?"));
        }

        Ok(&expanded)
    }

    /// Search dataset for a population parsimony match to the sequence.
    pub fn search(
        &self,
        sequence: &sequence::Record,
        populations: Option<&[&str]>,
        coordinates: Option<&[usize]>,
    ) -> Result<SearchResult, Report> {
        // initialize an empty result, this will be the final product of this function
        let mut result = SearchResult::new(sequence);

        // --------------------------------------------------------------------
        // Candidate Matches

        // Restrict our initial search to particular substitutions
        let search_subs = sequence.substitutions
            .iter()
            // optionally filter on input coordinates
            .filter(|s| {
                if let Some(coordinates) = coordinates {
                    coordinates.contains(&s.coord)
                } else {
                    true
                }
            })
            .collect_vec();

        // Identify populations with at least one matching sub
        let population_matches = self.mutations
            .into_iter()
            .filter_map(|(sub, pops)| (search_subs.contains(&sub)).then_some(pops))
            .flatten()
            .collect_vec();

        if population_matches.is_empty() {
            return Err(eyre!("No mutations matched a population in the dataset."));
        }

        // --------------------------------------------------------------------
        // Conflict

        // check which populations have extra subs/lacking subs
        population_matches.into_iter().try_for_each(|pop| {
            let pop_seq = &self.populations[pop];
            let summary = parsimony::Summary::from_records(sequence, pop_seq, coordinates)
                .unwrap_or(Err(eyre!("Failed to summarize parsimony between {} and {}", sequence.id, pop_seq.id))?);
            result.parsimony.insert(pop, summary);
            Ok(())
        });

        // --------------------------------------------------------------------
        // Top Populations

        // Tie breaking, prefer matches with the highest score (support - conflict)
        // beyond that, prefer matches with highest support or lowest conflict?
        // Ex. XCU parent #1 could be FL.23 (highest support) or XBC.1 (lowest conflict)

        // which population(s) has the highest score?
        // reminder: it can be negative when extreme recombinant genomic size

        let mut top_populations = population_matches;

        let max_score = result.parsimony
            .values()
            .map(|p| p.score())
            .max_by(|a, b| a.cmp(&b))
            .unwrap_or(Err(eyre!("Failed to get max score of sequence {}", sequence.id))?);

        top_populations.retain(|p| result.parsimony[p].score() == max_score);

        // break additional ties by max support
        let max_support = result.parsimony
            .iter()
            .filter(|(pop, pars)| top_populations.contains(pop))
            .map(|(_pop, pars)| pars.support.len())
            .max_by(|a, b| a.cmp(&b))
            .unwrap_or(Err(eyre!("Failed to get max support of sequence {}", sequence.id))?);

        top_populations.retain(|p| result.parsimony[p].support.len() == max_support);

        result.top_populations = &top_populations;

        // --------------------------------------------------------------------
        // Consensus Population

        // summarize top populations by common ancestor
        let consensus_population = if self.phylogeny.is_empty() {
            //result.top_populations.iter().join("|")
            // todo!() just take first when we don't have a phylogeny?
            result.top_populations[0]
        } else {
            self.phylogeny.get_common_ancestor(&result.top_populations)?
        };
        result.consensus_population = consensus_population;

        // if the common_ancestor was not in the populations list, add it
        let consensus_sequence = if !result.top_populations.contains(&consensus_population) {
            let pop = &consensus_population;

            // // Option #1. Actual sequence of the internal MRCA node?
            // let pop_seq = &self.populations[pop];
            // let summary = parsimony::Summary::from_sequence(sequence, pop_seq, coordinates)?;

            // Option #2. Consensus sequence of top populations?
            let top_populations = result.top_populations.iter().map(|s| s.as_ref()).collect_vec();
            debug!("Creating {pop} consensus genome from top populations.");
            let discard_sequence = true;
            let pop_seq = self.create_consensus(pop, &top_populations, discard_sequence)?;
            let summary = parsimony::Summary::from_records(sequence, &pop_seq, coordinates)?;
            result.parsimony.insert(pop, summary);
            pop_seq
        } else {
            self
                .populations
                .get(consensus_population)
                .cloned()
                .unwrap_or_else(|| panic!("Consensus population {consensus_population} is not in the dataset populations."))
        };

        // Filter out non-top populations from parsimony summaries
        // helps cut down on verbosity in debug log and data stored
        // Ex. XE, lots of BA.2 candidates
        result.parsimony.retain(|p, _| top_populations.contains(p) || p == &consensus_population );

        // Check if the consensus population is a known recombinant or descendant of one
        result.recombinant = match self.phylogeny.is_empty() {
            true => None,
            false => self.phylogeny.get_recombinant_ancestor(&consensus_population)?,
        };

        debug!("Search Result:\n{}", result.pretty_print());
        Ok(result)
    }

    /// If a population name is in the phylogeny but not in the sequences,
    /// find the closest parent that is in the sequences. Might be itself!
    ///
    /// I don't love this function name, need better!
    pub fn get_ancestor_with_sequence(&self, population: &str) -> Result<&str, Report> {
        if self.populations.contains_key(population) {
            return Ok(population);
        }
        // ancestors can have multiple paths to root, because of recombination
        let ancestors = self.phylogeny.get_ancestors(population)?;
        // filter the ancestor paths to just populations we have sequences for
        // prefer the ancestor path that is the longest
        let ancestors_filter = ancestors
            .into_iter()
            .map(|path| {
                path.into_iter().filter(|p| self.populations.contains_key(*p)).collect_vec()
            })
            .max_by(|a, b| a.len().cmp(&b.len()))
            .unwrap_or_default();

        // use the last element in the path (closest parent)
        let ancestor = ancestors_filter.last().cloned();
        ancestor.ok_or(Err(eyre!(
            "No ancestor of {population} has sequence data."
        ))?)
    }

    /// Write mapping of mutations to populations, coordinate sorted.
    pub fn write_mutations(&self, path: &Path) -> Result<(), Report> {
        // convert to vector for coordinate sorting
        let mut mutations = self.mutations.iter().collect_vec();
        mutations.sort_by(|a, b| a.0.coord.cmp(&b.0.coord));

        // convert substitution to string for serde pretty
        let mutations = mutations.iter().map(|(sub, pops)| (sub.to_string(), pops)).collect_vec();
        // create output file
        let mut file =
            File::create(path).wrap_err_with(|| format!("Failed to create file: {path:?}"))?;

        // parse to string
        let output = serde_json::to_string_pretty(&mutations)
            .wrap_err_with(|| "Failed to parse mutations.".to_string())?;

        // write to file
        file.write_all(format!("{}\n", output).as_bytes())
            .wrap_err_with(|| format!("Failed to write file: {path:?}"))?;

        Ok(())
    }

    /// Write dataset summary to file.
    pub fn write_summary(&self, path: &Path) -> Result<(), Report> {
        self.summary.write(&path)?;
        Ok(())
    }

    /// Write dataset edge cases to file.
    pub fn write_edge_cases(&self, path: &Path) -> Result<(), Report> {
        run::Args::write(&self.edge_cases, &path)?;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Dataset Search Result

//#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SearchResult<'dataset, 'seq> {
    pub sequence_id: &'seq str,
    pub consensus_population: &'dataset str,
    pub top_populations: &'dataset [&'dataset str],
    pub parsimony: BTreeMap<&'dataset str, parsimony::Summary<'seq, 'dataset>>,
    pub recombinant: Option<&'dataset str>,
}

impl<'dataset, 'seq> SearchResult<'dataset, 'seq> {
    pub fn new(sequence: &'seq sequence::Record) -> Self {
        SearchResult {
            sequence_id: &sequence.id,
            consensus_population: "",
            top_populations: &[],
            parsimony: &[],
            recombinant: None,
        }
    }

    pub fn pretty_print(&self) -> String {
        // Order the population lists from 'best' to 'worst'

        let max_display_items = 10;

        // score
        let mut score_order: Vec<(&str, isize)> = self.score.clone().into_iter().collect();
        score_order.sort_by(|a, b| b.1.cmp(&a.1));

        // put consensus population first, regardless of score
        let consensus_score: (&str, isize) = score_order
            .iter()
            .find(|(pop, _score)| *pop == self.consensus_population)
            .cloned()
            .expect("Failed to order consensus populations by score.");

        score_order.retain(|(pop, _score)| *pop != self.consensus_population);
        score_order.insert(0, consensus_score);

        // restrict display items for brevity
        let display_suffix = if score_order.len() > max_display_items {
            score_order = score_order[0..max_display_items].to_vec();
            "\n  ..."
        } else {
            ""
        };

        let mut support_order: Vec<&str> = Vec::new();
        let mut conflict_ref_order: Vec<&str> = Vec::new();
        let mut conflict_alt_order: Vec<&str> = Vec::new();

        score_order.iter().for_each(|(pop, _count)| {
            let subs = &self.support[pop];
            let count = subs.len();
            let display = format!("- {pop} ({count}): {}", subs.iter().join(", "));
            support_order.push(&display);

            let subs = &self.conflict_ref[pop];
            let count = subs.len();
            let display = format!("- {pop} ({count}): {}", subs.iter().join(", "));
            conflict_ref_order.push(&display);

            let subs = &self.conflict_alt[pop];
            let count = subs.len();
            let display = format!("- {pop} ({count}): {}", subs.iter().join(", "));
            conflict_alt_order.push(&display);
        });

        // Pretty string formatting for yaml
        let score_order = score_order
            .iter()
            .map(|(pop, count)| format!("- {}: {}", &pop, &count))
            .collect::<Vec<_>>();

        formatdoc!(
            "sequence_id: {}
            consensus_population: {}
            top_populations: {}
            recombinant: {}
            score:\n  {}{display_suffix}
            support:\n  {}{display_suffix}
            conflict_ref:\n  {}{display_suffix}
            conflict_alt:\n  {}{display_suffix}
            private: {}",
            self.sequence_id,
            self.consensus_population,
            self.top_populations.join(", "),
            self.recombinant.clone().unwrap_or("None"),
            score_order.join("\n  "),
            support_order.join("\n  "),
            conflict_ref_order.join("\n  "),
            conflict_alt_order.join("\n  "),
            self.private.iter().join(", ")
        )
    }
}
