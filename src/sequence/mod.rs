pub mod parsimony;

use color_eyre::eyre::{eyre, ContextCompat, Report, Result, WrapErr};
use color_eyre::Help;
use itertools::Itertools;
use noodles::{core::Position, fasta};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::default::Default;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Mutation {
    Substitution,
    Deletion,
}

// ----------------------------------------------------------------------------
// Deletion
// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Deletion {
    pub coord: usize,
    pub reference: char,
    pub alt: char,
}

impl std::fmt::Display for Deletion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}{}", self.reference, self.coord, self.alt)
    }
}

impl PartialEq for Deletion {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord && self.reference == other.reference && self.alt == other.alt
    }
}

impl Eq for Deletion {}

impl Ord for Deletion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coord.cmp(&other.coord)
    }
}

impl PartialOrd for Deletion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ----------------------------------------------------------------------------
// Substitution
// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Hash, Serialize, Deserialize, PartialEq)]
pub struct Substitution {
    pub coord: usize,
    pub reference: char,
    pub alt: char,
}

impl std::fmt::Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}{}", self.reference, self.coord, self.alt)
    }
}

impl FromStr for Substitution {
    type Err = Report;

    fn from_str(text: &str) -> Result<Self, Report> {
        let reference = text.chars().next().unwrap();
        let alt = text.chars().nth(text.len() - 1).unwrap();
        let coord = text[1..text.len() - 1].parse().unwrap();
        let substitution = Substitution {
            reference,
            alt,
            coord,
        };

        Ok(substitution)
    }
}

impl Eq for Substitution {}

impl Ord for Substitution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coord.cmp(&other.coord)
    }
}

impl PartialOrd for Substitution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Substitution {
    pub fn to_deletion(&self) -> Deletion {
        Deletion {
            coord: self.coord,
            reference: self.reference,
            alt: '-',
        }
    }
}

// ----------------------------------------------------------------------------
// Substitution
// ----------------------------------------------------------------------------

/// Reduced representation of an aligned sequence record.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub sequence: Vec<char>,
    pub alphabet: Vec<char>,
    pub genome_length: usize,
    pub substitutions: Vec<Substitution>,
    pub deletions: Vec<Deletion>,
    pub missing: Vec<usize>,
}

impl Record {
    pub fn new() -> Self {
        Record {
            id: String::new(),
            sequence: Vec::new(),
            alphabet: vec!['A', 'C', 'G', 'T'],
            genome_length: 0,
            substitutions: Vec::new(),
            deletions: Vec::new(),
            missing: Vec::new(),
        }
    }

    /// Parse fasta record into a rebar sequence record.
    pub fn from_fasta(
        fasta: fasta::Record,
        reference: Option<&Record>,
        mask: &Vec<usize>,
        discard_sequence: bool,
    ) -> Result<Self, Report> {
        let mut record = Record::new();
        record.id = fasta.name().to_string();

        // convert sequence to vec of bases, noodle positions are 1-based!
        let start = Position::try_from(1).unwrap();
        record.sequence = fasta
            .sequence()
            .get(start..)
            .context(format!("Failed to parse sequence record {}", record.id))?
            .into_iter()
            .map(|b| *b as char)
            .collect();
        record.genome_length = record.sequence.len();

        // check mask coord
        for bases in mask {
            if *bases > record.genome_length {
                return Err(eyre!(
                    "5' and 3' masking ({mask:?}) is incompatible with {} sequence length {}",
                    record.id,
                    record.genome_length
                )
                .suggestion("Please change your --mask parameter.")
                .suggestion("Maybe you want to disable masking all together with --mask 0,0 ?"));
            }
        }

        // if reference wasn't supplied, compare to self
        let reference = if let Some(reference) = reference {
            reference
        } else {
            &record
        };

        // compare reference and sequence lengths
        let ref_len = reference.genome_length;
        if record.genome_length != ref_len {
            return Err(eyre!(
                "Reference and {} are different lengths ({ref_len} vs {})!",
                record.id,
                record.genome_length
            )
            .suggestion(format!("Are you sure {} is aligned correctly?", record.id)));
        }

        // parse bases
        record.sequence = record
            .sequence
            .into_iter()
            .enumerate()
            .map(|(i, mut base)| {
                // Genomic coordinates are 1-based
                let coord: usize = i + 1;
                // reference base
                let r = reference.sequence[coord];

                // Mask: 5' and 3' ends, IUPAC ambiguity, and sites where reference is missing or deletion
                if (!mask.is_empty() && coord <= mask[0])
                    || (mask.len() == 2 && coord > record.genome_length - mask[1])
                    || (base != '-' && !record.alphabet.contains(&base))
                    || !record.alphabet.contains(&r)
                {
                    base = 'N';
                }

                match base {
                    // Missing (N)
                    'N' => record.missing.push(coord),
                    // Deletion (-)
                    '-' => {
                        let deletion = Deletion {
                            coord,
                            reference: r,
                            alt: '-',
                        };
                        record.deletions.push(deletion);
                    }
                    // Substitution
                    base if base != r => {
                        let substitution = Substitution {
                            coord,
                            reference: r,
                            alt: base,
                        };
                        record.substitutions.push(substitution);
                    }
                    // Reference
                    _ => (),
                }

                base
            })
            .collect_vec();

        if discard_sequence {
            record.sequence = Vec::new();
        }

        Ok(record)
    }
}

// ----------------------------------------------------------------------------
// Functions
// ----------------------------------------------------------------------------

/// Read first record of fasta path into sequence record.
pub fn read_reference(path: &Path, mask: &Vec<usize>) -> Result<Record, Report> {
    // start reading in the reference as fasta, raise error if file doesn't exist
    let mut reader = File::open(path).map(BufReader::new).map(fasta::Reader::new)?;

    // parse just the first record from the reference
    // 1. raise error if record iterator doesn't work
    // 2. raise error if first record is not proper fasta format.
    let reference = reader
        .records()
        .next()
        .ok_or_else(|| eyre!("Unable to read reference records: {path:?}"))?
        .wrap_err_with(|| eyre!("Unable to read first fasta record: {path:?}"))?;

    // convert to sequence
    let discard_sequence = false;
    let reference = Record::from_fasta(reference, None, mask, discard_sequence)?;

    Ok(reference)
}
