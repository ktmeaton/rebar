# Development

- `run::run`
- `dataset::expand_populations`
- `phylogeny::get_descendants`
- `phylogeny::get_ancestors`

- Phylogeny, give lifetime `<'phylo'>` and change methods from `String` to `&str`.
- Dataset, move attributes `name` and `tag` inside larger `summary`. This helps declutter the number of dataset attributes.
- Recombination, move support/conflict attributes into larger parsimony.
- `utils::table::Table`: Generalize format from String to any `<T>`. Should put trait limitations ToString though
- rename `sequence::Sequence` to `sequence::Record`
- rename `Sequence::from_record` to `Record::from_fasta`
- add param `discard_sequence` to `Record::from_fasta`.

## Lifetime Model

This is the general model for owned data and lifetimes.

- `Dataset`: Will persist for the entire duration of the program. Must own all data.
- `Sequence`, `Best Match`, `Recombination`, `Linelist`: Will live just for the duration of that record.
- `Barcodes`: How to do the collating...
