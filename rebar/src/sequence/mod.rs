use color_eyre::eyre::{eyre, ContextCompat, Report, Result, WrapErr};
use color_eyre::Help;
use indicatif::{style::ProgressStyle, ProgressBar};
use noodles::{core::Position, fasta};
#[cfg(feature = "rayon")]
use rayon::iter::{ParallelBridge, ParallelIterator};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    default::Default,
    fmt::{Debug, Display, Formatter},
    fs::File,
    io::BufReader,
    path::Path,
};

// ----------------------------------------------------------------------------
// Alphabet
// ----------------------------------------------------------------------------

/// Collection of characters that represent sequence bases.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Alphabet {
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "dna"))]
    Dna,
    #[cfg_attr(feature = "serde", serde(rename = "rna"))]
    Rna,
    #[cfg_attr(feature = "serde", serde(rename = "deletion"))]
    Deletion,
    #[cfg_attr(feature = "serde", serde(rename = "missing"))]
    Missing,
}

impl Alphabet {
    fn get_bases(&self) -> &[char] {
        match self {
            Alphabet::Dna => &['A', 'C', 'G', 'T'],
            Alphabet::Rna => &['A', 'C', 'G', 'U'],
            Alphabet::Deletion => &['-'],
            Alphabet::Missing => &['N'],
        }
    }
}

impl Display for Alphabet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Alphabet::Dna => String::from("dna"),
            Alphabet::Rna => String::from("rna"),
            Alphabet::Deletion => String::from("deletion"),
            Alphabet::Missing => String::from("missing"),
        };

        write!(f, "{}", name)
    }
}

// ----------------------------------------------------------------------------
// Deletion
// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Deletion {
    pub coord: usize,
    pub reference: char,
    pub alt: char,
}

// ----------------------------------------------------------------------------
// Substitution
// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Substitution {
    pub coord: usize,
    pub reference: char,
    pub alt: char,
}

// ----------------------------------------------------------------------------
// Record
// ----------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Record {
    pub id: String,
    pub sequence: Vec<char>,
    pub alphabet: Alphabet,
    pub genome_length: usize,
    pub substitutions: Vec<Substitution>,
    pub deletions: Vec<Deletion>,
    pub missing: Vec<usize>,
}

impl Record {
    /// Create a new DNA sequence [`Record`].
    pub fn new() -> Self {
        Record {
            id: String::new(),
            sequence: Vec::new(),
            alphabet: Alphabet::default(),
            genome_length: 0,
            substitutions: Vec::new(),
            deletions: Vec::new(),
            missing: Vec::new(),
        }
    }

    /// Create a [`rebar`] sequence [`Record`] from a [`noodles`] [`fasta::Record`].
    ///
    /// ## Arguments
    ///
    /// - `record` - [`noodles`] [`fasta::Record`]
    /// - `reference` - Optional [`Record`] of the reference genome.
    /// - `retain_full_sequence` - `true` if the full sequence should be stored, otherwise discard. Can be useful if memory is limited.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use tokio_test::{block_on, assert_ok};
    /// use rebar::{sequence, utils};
    ///
    /// // download a fasta
    /// # assert_ok!(block_on(async {
    /// let url = "https://raw.githubusercontent.com/nextstrain/ncov/v13/data/references_sequences.fasta";
    /// let path = "test/utils/download_file/reference.fasta";
    /// utils::download_file(&url, &path).await?;
    ///
    /// // read the fasta with a progress bar
    /// let progress = true;
    /// let (mut reader, count) = sequence::read(&path, progress)?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// # }));
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn from_noodles(
        record: fasta::Record,
        reference: Option<&Record>,
        alphabet: Alphabet,
    ) -> Result<Self, Report> {
        let mut sample = Record { alphabet, ..Default::default() };

        sample.id = record.name().to_string();

        // convert sequence to vec of char bases, noodle positions are 1-based!
        let start = Position::try_from(1)?;
        sample.sequence = record
            .sequence()
            .get(start..)
            .wrap_err(format!("Failed to parse sequence record {}", &sample.id))?
            .iter()
            .map(|b| *b as char)
            .collect::<Vec<_>>();

        sample.genome_length = sample.sequence.len();

        // compare to reference
        if let Some(reference) = reference {
            if reference.genome_length != sample.genome_length {
                Err(eyre!(
                    "Reference and {} are different lengths ({} vs {})!",
                    sample.id,
                    reference.genome_length,
                    sample.genome_length
                )
                .suggestion(format!("Are you sure {} is aligned correctly?", sample.id)))?;
            }

            // get coordinates of reference deletions
            let ref_del_coords = reference.deletions.iter().map(|d| d.coord).collect::<Vec<_>>();

            // Construct iterator to traverse sample and reference bases together
            sample.sequence.iter().enumerate().for_each(|(i, base)| {
                // Genomic coordinates are 1-based
                let coord: usize = i + 1;
                // reference base
                let r = &reference.sequence[i];

                // Option 1. Sample is Missing
                // Or reference is missing or deletion
                if Alphabet::Missing.get_bases().contains(base)
                    || reference.missing.contains(&coord)
                    || ref_del_coords.contains(&coord)
                {
                    sample.missing.push(coord);
                }
                // Option 2. Sample has a deletion
                else if Alphabet::Deletion.get_bases().contains(base) {
                    sample.deletions.push(Deletion { coord, reference: *r, alt: *base });
                }
                // Option 3. Sample has a subsitution
                else if sample.alphabet.get_bases().contains(base) && r != base {
                    sample.substitutions.push(Substitution { coord, reference: *r, alt: *base });
                }
                // Option 4. Sample has an ambiguous bases, treat as missing
                else if !sample.alphabet.get_bases().contains(base) {
                    sample.missing.push(coord);
                }
            });
        }

        Ok(sample)
    }
}

// /// Returns the first sequence [`Record`] in a fasta file.
// ///
// /// This is used mostly for reading in a reference genome.
// ///
// /// ## Examples
// ///
// /// ```rust
// /// ```
// pub fn read_first<P>(
//     path: &P,
//     reference: Option<&Record>,
//     alphabet: Alphabet,
// ) -> Result<Record, Report>
// where
//     P: AsRef<Path> + Debug,
// {
//     let progress = false;
//     let (mut reader, _count) = read(&path, progress)?;
//     let first =
//         reader.records().next().wrap_err("Failed to read first fasta record: {path:?}")??;
//     let record = Record::from_noodles(first, reference, alphabet)?;
//     Ok(record)
// }

/// Returns a [`Reader`](fasta::reader::Reader) over the sequence records in a fasta file and the number of records.
///
/// Optionally displays a progress bar as the file is being parsed.
///
/// The purpose of this function is to assist in processing a fasta file as a stream of data, and not
/// necessarily reading it all into memory. The return value `reader` can be turned
/// into an iterator with `reader.records()`. The records can then be processed, or [`chained`](chain) with another reader.
///
/// ## Arguments
///
/// - `path` - Path to fasta file.
/// - `progress` - True if a progress bar should be used.
///
/// ## Examples
///
/// ```rust
/// # use tokio_test::{block_on, assert_ok};
/// use rebar::{sequence, utils};
///
/// // download a fasta
/// # assert_ok!(block_on(async {
/// let url = "https://raw.githubusercontent.com/nextstrain/ncov/v13/data/references_sequences.fasta";
/// let path = "test/utils/download_file/reference.fasta";
/// utils::download_file(&url, &path).await?;
///
/// // read the fasta with a progress bar
/// let progress = true;
/// let (mut reader, count) = sequence::get_reader(&path, progress)?;
/// assert_eq!(count, 2);
///
/// // get an iterator over the sequence records
/// let records = reader.records().filter_map(|r| r.ok());
/// let ids: Vec<String> = records.map(|r| r.name().to_string()).collect();
/// assert_eq!(ids, vec!["Wuhan/Hu-1/2019".to_string(), "21L".to_string()]);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn get_reader<P>(
    path: &P,
    progress: bool,
) -> Result<(fasta::Reader<BufReader<File>>, usize), Report>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    // create a progress bar, will only be displayed/updated if progress=true
    let progress_bar_style = ProgressStyle::with_template(
            "{bar:40} {pos}/{len} ({percent}%) | Sequences / Second: {per_sec} | Elapsed: {elapsed_precise}"
        ).wrap_err("Failed to create progress bar from template.")?;
    let progress_bar = ProgressBar::new(0_u64);
    progress_bar.set_style(progress_bar_style);

    // function to conditionally update the progress bar
    let f_update = |progress_bar: &ProgressBar, progress: bool| {
        if progress {
            progress_bar.inc_length(1);
            progress_bar.inc(1);
        }
    };

    // read alignment to count records
    let mut reader = File::open(path)
        .map(BufReader::new)
        .map(fasta::Reader::new)
        .wrap_err(format!("Failed to read: {path:?}"))?;

    // decide if we are using rayon multithreading
    let num_records = if cfg!(rayon) {
        reader.records().par_bridge().inspect(|_| f_update(&progress_bar, progress)).count()
    } else {
        reader.records().inspect(|_| f_update(&progress_bar, progress)).count()
    };
    progress_bar.finish();

    Ok((File::open(path).map(BufReader::new).map(fasta::Reader::new)?, num_records))
}
