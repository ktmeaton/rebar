//! Download test dataset Toy1.
//!
//! More description goes here!

use crate::dataset::{Tag, VersionedFile};
use rebar_phylo::{ToDot, ToJson, ToMermaid};

use color_eyre::eyre::{Report, Result, WrapErr};
use std::fmt::Debug;
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
    // create output dir
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;

    // create file content
    let file_name = "annotations.tsv";
    let mut builder = tabled::builder::Builder::default();
    ANNOTATIONS.iter().for_each(|row| {
        builder.push_record(row.to_vec());
    });
    let content = builder.build().to_string();

    // write file file content
    let path = output_dir.join(file_name);
    std::fs::write(&path, content.as_bytes()).wrap_err(format!("Failed to write: {path:?}"))?;

    // return versioned file info
    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(chrono::Local::now().into()),
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
    // create output dir
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;

    // create file content
    let file_name = "reference.fasta";
    let content = REFERENCE;

    // write file file content
    let path = output_dir.join(file_name);
    std::fs::write(&path, content.as_bytes()).wrap_err(format!("Failed to write: {path:?}"))?;

    // return versioned file info
    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(chrono::Local::now().into()),
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
/// # assert_eq!(std::fs::read_to_string(&path)?, toy1::POPULATIONS);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn populations<P>(_tag: &Tag, output_dir: &P) -> Result<VersionedFile, Report>
where
    P: AsRef<Path> + Clone + Debug,
{
    // create output dir
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;

    // create file content
    let file_name = "populations.fasta";
    let content = POPULATIONS;

    // write file file content
    let path = output_dir.join(file_name);
    std::fs::write(&path, content.as_bytes()).wrap_err(format!("Failed to write: {path:?}"))?;

    // return versioned file info
    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(chrono::Local::now().into()),
        ..Default::default()
    };

    Ok(versioned_file)
}

// ----------------------------------------------------------------------------
// Edge Cases
// ----------------------------------------------------------------------------

/// Create and write Toy1 edge cases.
///
/// ## Examples
///
/// ```rust
/// use rebar::dataset::{Tag, toy1};
/// let versioned_file = toy1::edge_cases(&Tag::Custom, &"test/dataset/toy1")?;
/// # let path = std::path::PathBuf::from("test/dataset/toy1").join(versioned_file.local);
/// # assert!(path.exists());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
#[cfg(feature = "serde")]
pub fn edge_cases<P>(_tag: &Tag, output_dir: &P) -> Result<VersionedFile, Report>
where
    P: AsRef<Path> + Clone + Debug,
{
    // create output dir
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;

    let file_name = "edge_cases.json";
    let _path = output_dir.join(file_name);
    // todo!() once run args are back
    // let edge_cases = RunArgs::default();
    // let content = serde_json::to_string_pretty(edge_cases).wrap_err(format!("Failed to convert phylogeny to JSON."))?;
    // std::fs::write(&path, content.as_bytes()).wrap_err(format!("Failed to write: {path:?}"))?;

    // return versioned file info
    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(chrono::Local::now().into()),
        ..Default::default()
    };

    Ok(versioned_file)
}

// ----------------------------------------------------------------------------
// Phylogeny
// ----------------------------------------------------------------------------

/// Create and write Toy1 phylogeny.
///
/// ## Examples
///
/// ```rust
/// use rebar::dataset::{Tag, toy1};
/// let versioned_file = toy1::phylogeny(&Tag::Custom, &"test/dataset/toy1")?;
/// # let path = std::path::PathBuf::from("test/dataset/toy1").join(versioned_file.local);
/// # assert!(path.exists());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
#[cfg(feature = "phylo")]
pub fn phylogeny<P>(_tag: &Tag, output_dir: &P) -> Result<VersionedFile, Report>
where
    P: AsRef<Path> + Clone + Debug,
{
    // create output dir
    let output_dir: PathBuf = output_dir.as_ref().into();
    std::fs::create_dir_all(&output_dir).wrap_err("Failed to create directory: {output_dir:?}")?;

    let phylogeny = rebar_phylo::example_1();

    // (optional) mermaid content
    let content = phylogeny.to_mermaid()?;
    let file_name = "phylogeny.mermaid";
    let path = output_dir.join(file_name);
    std::fs::write(&path, content.as_bytes()).wrap_err(format!("Failed to write: {path:?}"))?;

    // (optional) dot content
    let file_name = "phylogeny.dot";
    let path = output_dir.join(file_name);
    let content = phylogeny.to_dot()?;
    std::fs::write(&path, content.as_bytes()).wrap_err(format!("Failed to write: {path:?}"))?;

    // (mandatory) json content
    let file_name = "phylogeny.json";
    let path = output_dir.join(file_name);
    let content = phylogeny.to_json()?;
    std::fs::write(&path, content.as_bytes()).wrap_err(format!("Failed to write: {path:?}"))?;

    // return versioned file info
    let versioned_file = VersionedFile {
        local: file_name.into(),
        date_created: Some(chrono::Local::now().into()),
        ..Default::default()
    };

    Ok(versioned_file)
}
