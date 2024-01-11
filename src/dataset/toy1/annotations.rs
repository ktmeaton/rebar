use crate::utils::table::Table;
use color_eyre::eyre::{Report, Result};

const HEADERS: &'static [&str] = &["gene", "abbreviation", "start", "end"];
const ROWS: &'static [&'static [&'static str]] =
    &[&["Gene1", "g1", "1", "3"], &["Gene2", "g2", "12", "20"]];

/// Create Toy1 genome annotations.
pub fn build() -> Table<&'static str> {
    Table {
        headers: HEADERS.to_vec(),
        rows: ROWS.into_iter().map(|row| row.to_vec()).collect(),
        path: None,
    }
}
