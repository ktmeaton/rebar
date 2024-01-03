use crate::utils::table::Table;
use color_eyre::eyre::{Report, Result};
use itertools::Itertools;

/// Create SARS-CoV-2 genome annotations.
pub fn build() -> Result<Table<String>, Report> {

    let headers = vec!["gene", "abbreviation", "start", "end"];
    let rows = vec![
        vec!["ORF1a", "1a", "266", "13468"],
        vec!["ORF1b", "1b", "13468", "21555"],
        vec!["S", "S", "21563", "25384"],
        vec!["ORF3a", "3a", "25393", "26220"],
        vec!["E", "E", "26245", "26472"],
        vec!["M", "M", "26523", "27191"],
        vec!["ORF6", "6", "27202", "27387"],
        vec!["ORF7a", "7a", "27394", "27759"],
        vec!["ORF7b", "7b", "27756", "27887"],
        vec!["ORF8", "8", "27894", "28259"],
        vec!["ORF9b", "9b", "28284", "28577"],
    ];

    let mut table = Table::new();
    table.headers = headers.into_iter().map(String::from).collect();
    table.rows = rows.into_iter().map(|r| r.into_iter().map(String::from).collect()).collect();    

    Ok(table)
}
