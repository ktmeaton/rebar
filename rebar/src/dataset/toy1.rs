//! Download test dataset Toy1.
//!
//! More description goes here!

// ----------------------------------------------------------------------------
// Annotations
// ----------------------------------------------------------------------------

/// Genome annotations.
pub mod annotations {

    use crate::Table;

    const HEADERS: &[&str] = &["gene", "abbreviation", "start", "end"];
    const ROWS: &[&[&str]] = &[&["Gene1", "g1", "1", "3"], &["Gene2", "g2", "12", "20"]];

    /// Returns a table of genome annotations.
    ///
    /// ```rust
    /// use rebar::dataset::toy1;
    ///
    /// let table = toy1::annotations::get();
    /// println!("{}", table.to_markdown()?);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// | gene  | abbreviation | start | end |
    /// |-------|--------------|-------|-----|
    /// | Gene1 |      g1      |   1   |  3  |
    /// | Gene2 |      g2      |  12   | 20  |
    pub fn get() -> Table<&'static str> {
        Table {
            headers: HEADERS.to_vec(),
            rows: ROWS.iter().map(|row| row.to_vec()).collect(),
            path: None,
        }
    }
}

// ----------------------------------------------------------------------------
// Download
// ----------------------------------------------------------------------------

/// Download the dataset.
pub mod download {
    use crate::dataset::attributes::Tag;
    use crate::utils::remote_file::RemoteFile;
    use chrono::Local;
    use color_eyre::eyre::{Report, Result, WrapErr};
    use indoc::formatdoc;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    /// Create and write Toy1 reference sequence.
    pub fn reference(_tag: &Tag, path: &Path) -> Result<RemoteFile, Report> {
        let sequences = formatdoc!(
            "
            >Reference
            AAAAAAAAAAAAAAAAAAAA
            "
        );

        let mut file =
            File::create(path).wrap_err_with(|| format!("Unable to create file: {path:?}"))?;
        file.write_all(sequences.as_bytes())
            .wrap_err_with(|| format!("Unable to write file: {path:?}"))?;

        let remote_file = RemoteFile {
            local_path: path.to_owned(),
            date_created: Local::now().into(),
            ..Default::default()
        };

        Ok(remote_file)
    }

    /// Create and write Toy1 populations sequence.
    pub fn populations(_tag: &Tag, path: &Path) -> Result<RemoteFile, Report> {
        let sequences = formatdoc!(
            "
            >A
            CCCCCCAACCCCCCCCCCCC
            >B
            TTTTTTTTTTTTTTTTTTAA
            >C
            AAGGGGGGGGGGGGGGGGGG
            >D
            CCCCCCAACCCTTTTTTTAA
            >E
            AAGCCCAACCCTTTTTTTAA
            "
        );

        let mut file =
            File::create(path).wrap_err_with(|| format!("Unable to create file: {path:?}"))?;
        file.write_all(sequences.as_bytes())
            .wrap_err_with(|| format!("Unable to write file: {path:?}"))?;

        let remote_file = RemoteFile {
            local_path: path.to_owned(),
            date_created: Local::now().into(),
            ..Default::default()
        };

        Ok(remote_file)
    }
}

// ----------------------------------------------------------------------------
// Edge Cases
// ----------------------------------------------------------------------------

/// Recombinant population edge cases.
pub mod edge_cases {

    use crate::run;

    /// Returns recombinant edge cases.
    ///
    /// ```rust
    /// use rebar::dataset::toy1;
    ///
    /// let edge_cases = toy1::edge_cases::get();
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get() -> Vec<run::Args> {
        Vec::new()
    }
}

// ----------------------------------------------------------------------------
// Phylogeny
// ----------------------------------------------------------------------------

/// Phylogenetic representation as an Ancestral Recombination Graph (ARG).
pub mod phylogeny {

    use crate::Phylogeny;
    use color_eyre::eyre::{Report, Result};

    /// Returns the phylogeny of toy1 populations.
    pub fn get() -> Result<Phylogeny<&'static str, u16>, Report> {
        let data = vec![
            ("A", "B", 1),
            ("A", "C", 1),
            ("A", "D", 1),
            ("B", "D", 1),
            ("C", "F", 1),
            ("C", "G", 1),
            ("D", "E", 1),
            ("E", "G", 1),
            ("E", "H", 1),
            ("F", "G", 1),
        ];

        Phylogeny::from_vec(data)
    }
}
