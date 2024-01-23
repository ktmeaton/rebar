//! Download test dataset Toy1.
//!
//! More description goes here!

use crate::dataset::{Tag, VersionedFile};
use crate::utils;
use chrono::Local;
use color_eyre::eyre::{Report, Result, WrapErr};
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const ANNOTATIONS: &[&[&str]] = &[
    &["gene", "abbreviation", "start", "end"],
    &["Gene1", "g1", "1", "3"],
    &["Gene2", "g2", "12", "20"],
];
pub const REFERENCE: &str = ">Reference
AAAAAAAAAAAAAAAAAAAA";
pub const POPULATIONS: &str = ">A
CCCCCCAACCCCCCCCCCCC
>B
TTTTTTTTTTTTTTTTTTAA
>C
AAGGGGGGGGGGGGGGGGGG
>D
CCCCCCAACCCTTTTTTTAA
>E
AAGCCCAACCCTTTTTTTAA";

/// Create and write the toy1 annotations table.
///
/// ```rust
/// use rebar::dataset::{Tag, toy1};
/// let versioned_file = toy1::annotations(&Tag::Custom, &"test/dataset/toy1")?;
/// # let path = std::path::PathBuf::from("test/dataset/toy1").join(versioned_file.local);
/// # assert!(path.exists());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
///
/// +-------+--------------+-------+-----+
/// | gene  | abbreviation | start | end |
/// +-------+--------------+-------+-----+
/// | Gene1 | g1           | 1     | 3   |
/// +-------+--------------+-------+-----+
/// | Gene2 | g2           | 12    | 20  |
/// +-------+--------------+-------+-----+
pub fn annotations<P>(_tag: &Tag, output_dir: &P) -> Result<VersionedFile, Report>
where
    P: AsRef<Path> + Clone + Debug,
{
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;
    let file_name = "annotations.tsv";
    let path = output_dir.join(file_name);

    // build annotations table
    let mut builder = tabled::builder::Builder::default();
    ANNOTATIONS.iter().for_each(|row| {
        builder.push_record(row.to_vec());
    });
    let table = builder.build();
    utils::write_table(&table, &path, None).wrap_err("Failed to write table: {path:?}")?;

    // create versioned file
    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(Local::now().into()),
        ..Default::default()
    };

    Ok(versioned_file)
}

/// Create and write Toy1 reference sequence.
///
/// ## Examples
///
/// ```rust
/// use rebar::dataset::{Tag, toy1};
/// let versioned_file = toy1::reference(&Tag::Custom, &"test/dataset/toy1")?;
/// # let path = std::path::PathBuf::from("test/dataset/toy1").join(versioned_file.local);
/// # assert!(path.exists());
/// # assert_eq!(std::fs::read_to_string(&path)?, toy1::REFERENCE);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn reference<P>(_tag: &Tag, output_dir: &P) -> Result<VersionedFile, Report>
where
    P: AsRef<Path> + Clone + Debug,
{
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;
    let file_name = "reference.fasta";
    let path = output_dir.join(file_name);

    let mut file = File::create(path.clone())
        .wrap_err_with(|| format!("Unable to create reference: {path:?}"))?;
    file.write_all(REFERENCE.as_bytes())
        .wrap_err_with(|| format!("Unable to write reference: {path:?}"))?;

    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(Local::now().into()),
        ..Default::default()
    };

    Ok(versioned_file)
}

/// Create and write Toy1 populations sequences.
///
/// ## Examples
///
/// ```rust
/// use rebar::dataset::{Tag, toy1};
/// let versioned_file = toy1::populations(&Tag::Custom, &"test/dataset/toy1")?;
/// # let path = std::path::PathBuf::from("test/dataset/toy1").join(versioned_file.local);
/// # assert!(path.exists());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn populations<P>(_tag: &Tag, output_dir: &P) -> Result<VersionedFile, Report>
where
    P: AsRef<Path> + Clone + Debug,
{
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;
    let file_name = "populations.fasta";
    let path = output_dir.join(file_name);

    let mut file = File::create(path.clone())
        .wrap_err_with(|| format!("Unable to create populations: {path:?}"))?;
    file.write_all(POPULATIONS.as_bytes())
        .wrap_err_with(|| format!("Unable to write populations: {path:?}"))?;

    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(Local::now().into()),
        ..Default::default()
    };

    Ok(versioned_file)
}

// // ----------------------------------------------------------------------------
// // Edge Cases
// // ----------------------------------------------------------------------------

// /// Recombinant population edge cases.
// pub mod edge_cases {

//     use crate::run;

//     /// Returns recombinant edge cases.
//     ///
//     /// ```rust
//     /// use rebar::dataset::toy1;
//     ///
//     /// let edge_cases = toy1::edge_cases::get();
//     /// # Ok::<(), color_eyre::eyre::Report>(())
//     /// ```
//     pub fn get() -> Vec<run::Args> {
//         Vec::new()
//     }
// }

// // ----------------------------------------------------------------------------
// // Phylogeny
// // ----------------------------------------------------------------------------

// /// Phylogenetic representation as an Ancestral Recombination Graph (ARG).
// pub mod phylogeny {

//     use crate::Phylogeny;
//     use color_eyre::eyre::{Report, Result};

//     /// Returns the phylogeny of toy1 populations.
//     pub fn get() -> Result<Phylogeny<&'static str, u16>, Report> {
//         let data = vec![
//             ("A", "B", 1),
//             ("A", "C", 1),
//             ("A", "D", 1),
//             ("B", "D", 1),
//             ("C", "F", 1),
//             ("C", "G", 1),
//             ("D", "E", 1),
//             ("E", "G", 1),
//             ("E", "H", 1),
//             ("F", "G", 1),
//         ];

//         Phylogeny::from_vec(data)
//     }
// }
