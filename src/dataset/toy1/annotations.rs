use crate::utils::table::Table;
use color_eyre::eyre::{Report, Result};
use itertools::Itertools;

/// Create Toy1 genome annotations.
pub fn build() -> Result<Table<String>, Report> {

    let headers = vec!["gene", "abbreviation", "start", "end"];
    let rows = vec![
        vec!["Gene1", "g1", "1", "3"],
        vec!["Gene2", "g2", "12", "20"],
    ];

    let mut table = Table::new();
    table.headers = headers.into_iter().map(String::from).collect();
    table.rows = rows.into_iter().map(|r| r.into_iter().map(String::from).collect()).collect();

    Ok(table)
}
