use crate::dataset::{attributes::Name};

use chrono::NaiveDate;
use color_eyre::eyre::{Report, Result};

#[test]
fn sarscov2_min_date() -> Result<(), Report> {

    let observed = Name::SarsCov2.get_compatibility()?.min_date;
    let expected = Some(NaiveDate::parse_from_str("2023-02-09", "%Y-%m-%d")?);
    assert_eq!(expected, observed);
    Ok(())
}