use crate::dataset::{Dataset, SearchResult};
use crate::recombination::{validate, Recombination};
use crate::{utils, utils::table::Table};
use color_eyre::eyre::{Report, Result};
use itertools::Itertools;

// ----------------------------------------------------------------------------
// LineList

const LINELIST_HEADERS: Vec<&str> = vec![
    "strain",
    "validate",
    "validate_details",
    "population",
    "recombinant",
    "parents",
    "breakpoints",
    "edge_case",
    "unique_key",
    "regions",
    "substitutions",
    "genome_length",
    "dataset_name",
    "dataset_tag",
    "cli_version",
];

pub fn linelist(
    best_match: &SearchResult,
    recombination: &Recombination,
) -> Result<Table<String>, Report> {
    let mut table = Table::new();
    table.headers = LINELIST_HEADERS;
    table.rows.push(vec![""; table.headers.len()]);
    let row = 0;

    table.set("strain", row, recombination.sequence.id.as_str())?;
    table.set("population", row, best_match.consensus_population)?;
    table.set("recombinant", row, recombination.recombinant.unwrap_or(""))?;
    table.set("parents", row, recombination.parents.join(",").as_str())?;
    table.set(
        "breakpoints",
        row,
        recombination.breakpoints.iter().join(",").as_str(),
    )?;
    table.set(
        "edge_case",
        row,
        recombination.edge_case.to_string().as_str(),
    )?;

    // validate, currently requires phylogeny
    if !recombination.dataset.phylogeny.is_empty() {
        let validate = validate::validate(recombination.dataset, best_match, recombination)?;
        if let Some(validate) = validate {
            table.set("validate", row, validate.status.to_string().as_str())?;
            table.set(
                "validate_details",
                row,
                validate.details.iter().join(";").as_str(),
            )?
        }
    }

    table.set("unique_key", row, recombination.unique_key.as_str())?;
    table.set(
        "regions",
        row,
        recombination.regions.values().join(",").as_str(),
    )?;
    table.set(
        "genome_length",
        row,
        recombination.dataset.reference.genome_length.to_string().as_str(),
    )?;
    table.set(
        "dataset_name",
        row,
        recombination.dataset.summary.name.to_string().as_str(),
    );
    table.set(
        "dataset_tag",
        row,
        recombination.dataset.summary.tag.to_string().as_str(),
    );
    table.set("cli_version", row, env!("CARGO_PKG_VERSION"));

    // --------------------------------------------------------------------
    // Substitutions, annotated by parental origin or private

    let subs_by_origin = recombination.get_substitution_origins(best_match)?;

    // origin order: primary parent, secondary parent, recombinant, private
    let mut origins = match recombination.recombinant.is_some() {
        true => recombination.parents,
        false => vec![best_match.consensus_population],
    };
    origins.push("private");

    let substitutions = origins
        .into_iter()
        .filter_map(|o| {
            let subs = subs_by_origin.get(o).cloned().unwrap_or_default();
            let subs_format = format!("{}|{o}", subs.iter().join(","));
            (!subs.is_empty()).then_some(subs_format)
        })
        .join(";");
    table.set("substitutions", row, substitutions.as_str())?;

    // convert to Table of owned strings
    let mut table = Table::new();
    table.headers = table.headers.into_iter().map(String::from).collect_vec();
    table.rows = table
        .rows
        .into_iter()
        .map(|row| row.into_iter().map(String::from).collect_vec())
        .collect_vec();

    Ok(table)
}
