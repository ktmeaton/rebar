use crate::dataset::{attributes::Name, list};
use crate::utils::table::Table;
use color_eyre::eyre::{Report, Result};

const HEADERS: &'static [&str] = &["Name", "CLI Version", "Minimum Tag Date", "Maximum Tag Date"];

#[test]
fn list_all() -> Result<(), Report> {
    let args = list::Args::default();
    let observed = list::datasets(&args)?;

    let expected = Table {
        headers: HEADERS.to_vec(),
        rows: vec![
            vec!["sars-cov-2", "", "2023-02-09", "nightly"],
            vec!["toy1", ">=0.2.0", "nightly", "nightly"],
        ],
        path: None,
    }.to_string_values();

    assert_eq!(expected, observed);
    Ok(())
}

#[test]
fn list_sarscov2() -> Result<(), Report> {
    let args = list::Args { name: Some(Name::SarsCov2), ..Default::default()};
    let observed = list::datasets(&args)?;

    let expected = Table {
        headers: HEADERS.to_vec(),
        rows: vec![
            vec!["sars-cov-2", "", "2023-02-09", "nightly"],
        ],
        path: None,
    }.to_string_values();

    assert_eq!(expected, observed);
    Ok(())
}

#[test]
fn list_toy1() -> Result<(), Report> {
    let args = list::Args { name: Some(Name::Toy1), ..Default::default()};
    let observed = list::datasets(&args)?;

    let expected = Table {
        headers: HEADERS.to_vec(),
        rows: vec![
            vec!["toy1", ">=0.2.0", "nightly", "nightly"],            
        ],
        path: None,
    }.to_string_values();

    assert_eq!(expected, observed);
    Ok(())
}