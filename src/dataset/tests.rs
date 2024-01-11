use crate::dataset::list;

use color_eyre::eyre::{Report, Result};

fn list_default() -> Result<(), Report> {
    let args = list::Args::default();
    list::datasets(&args)
}
