//! Run recombination detection algorithm on input sequences and populations.

#[cfg(feature = "cli")]
use clap::{Args as ClapArgs, Parser};
use color_eyre::eyre::{Report, Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

/// Detect recombination in a dataset population and/or input alignment.
pub fn run(_args: &RunArgs) -> Result<(), Report> {
    Ok(())
}
/// ---------------------------------------------------------------------------
/// RunArgs
/// ---------------------------------------------------------------------------

/// Detect recombination in a dataset population and/or input alignment.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct RunArgs {
    /// Dataset directory.
    #[cfg_attr(feature = "cli", clap(short = 'd', long, required = true))]
    #[serde(skip_serializing_if = "RunArgs::is_default_dataset_dir", skip_deserializing)]
    pub dataset_dir: PathBuf,

    #[cfg_attr(feature = "cli", command(flatten))]
    #[serde(skip_serializing_if = "RunArgs::is_default_input", skip_deserializing)]
    pub input: Input,

    /// Restrict parent search to just these candidate parents.
    #[cfg_attr(feature = "cli", arg(long, value_delimiter = ','))]
    pub parents: Option<Vec<String>>,

    /// Remove these populations from the dataset.
    ///
    /// Regardless of whether you use '*' or not, all descendants of the
    /// specified populations will be removed.
    #[cfg_attr(feature = "cli", arg(short = 'k', long))]
    pub knockout: Option<Vec<String>>,

    /// Number of bases to mask at the 5' and 3' ends.
    ///
    /// Comma separated. Use --mask 0,0 to disable masking.
    #[cfg_attr(feature = "cli", arg(short = 'm', long, value_delimiter = ',', default_values_t = RunArgs::default().mask))]
    pub mask: Vec<usize>,

    /// Maximum number of search iterations to find each parent.
    #[cfg_attr(feature = "cli", arg(short = 'i', long, default_value_t = RunArgs::default().max_iter))]
    pub max_iter: usize,

    /// Maximum number of parents.
    #[cfg_attr(feature = "cli", arg(long, default_value_t = RunArgs::default().max_parents))]
    pub max_parents: usize,

    /// Minimum number of parents.
    #[cfg_attr(feature = "cli", arg(long, default_value_t = RunArgs::default().min_parents))]
    pub min_parents: usize,

    /// Minimum number of consecutive bases in a parental region.
    #[cfg_attr(feature = "cli", arg(short = 'c', long, default_value_t = RunArgs::default().min_consecutive))]
    pub min_consecutive: usize,

    /// Minimum length of a parental region.
    #[cfg_attr(feature = "cli", arg(short = 'l', long, default_value_t = RunArgs::default().min_length))]
    pub min_length: usize,

    /// Minimum number of substitutions in a parental region.
    #[cfg_attr(feature = "cli", arg(short = 's', long, default_value_t = RunArgs::default().min_subs))]
    pub min_subs: usize,

    /// Run a naive search, which does not use information about edge cases or known recombinant parents.
    #[cfg_attr(feature = "cli", arg(short = 'u', long, default_value_t = RunArgs::default().naive))]
    pub naive: bool,

    /// Output directory.
    ///
    /// If the directory does not exist, it will be created.
    #[cfg_attr(feature = "cli", clap(short = 'o', long, required = true))]
    #[serde(skip_serializing_if = "RunArgs::is_default_output_dir", skip_deserializing)]
    pub output_dir: PathBuf,

    /// Number of CPU threads to use.
    #[cfg_attr(feature = "cli", clap(short = 't', long, default_value_t = RunArgs::default().threads))]
    #[serde(skip)]
    pub threads: usize,
}

impl Default for RunArgs {
    fn default() -> Self {
        RunArgs {
            dataset_dir: PathBuf::new(),
            input: Input::default(),
            knockout: None,
            mask: vec![100, 200],
            max_iter: 3,
            max_parents: 2,
            min_consecutive: 3,
            min_length: 500,
            min_parents: 2,
            min_subs: 1,
            naive: false,
            output_dir: PathBuf::new(),
            parents: None,
            threads: 1,
        }
    }
}

impl RunArgs {
    pub fn new() -> Self {
        RunArgs {
            dataset_dir: PathBuf::new(),
            input: Input::default(),
            knockout: None,
            mask: vec![0, 0],
            max_iter: 0,
            min_parents: 0,
            max_parents: 0,
            min_consecutive: 0,
            min_length: 0,
            min_subs: 0,
            output_dir: PathBuf::new(),
            parents: None,
            threads: 0,
            naive: false,
        }
    }

    /// Check if input is default.
    pub fn is_default_dataset_dir(path: &Path) -> bool {
        path == RunArgs::default().dataset_dir
    }

    /// Check if input is default.
    pub fn is_default_input(input: &Input) -> bool {
        input == &RunArgs::default().input
    }
    /// Check if output directory is default.
    pub fn is_default_output_dir(path: &Path) -> bool {
        path == RunArgs::default().output_dir
    }

    /// Override RunArgs for edge case handling of particular recombinants.
    pub fn apply_edge_case(&self, new: &RunArgs) -> Result<RunArgs, Report> {
        let mut output = self.clone();

        output.max_iter = new.max_iter;
        output.max_parents = new.max_parents;
        output.min_consecutive = new.min_consecutive;
        output.min_length = new.min_length;
        output.min_subs = new.min_subs;
        output.parents = new.parents.clone();
        output.naive = new.naive;

        Ok(output)
    }

    /// Reads [`RunArgs`] from a JSON file.
    pub fn read<P>(path: &P) -> Result<Vec<RunArgs>, Report>
    where
        P: AsRef<Path> + Debug,
    {
        let input = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read run arguments: {path:?}."))?;
        let run_args = serde_json::from_str(&input)
            .wrap_err_with(|| format!("Failed to deserialize run arguments: {input}"))?;
        Ok(run_args)
    }

    /// Write [`RunArgs`] to a JSON file.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rebar::RunArgs;
    /// RunArgs::write(&RunArgs::default(), &"test/run/RunArgs/run_args.json")?;
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn write<P>(&self, path: &P) -> Result<(), Report>
    where
        P: AsRef<Path> + Debug,
    {
        crate::utils::create_parent_dir(path)?;
        let output = serde_json::to_string_pretty(self)
            .wrap_err(format!("Failed to serialize run arguments: {self:?}"))?;
        std::fs::write(path, output)
            .wrap_err(format!("Failed to write run arguments: {path:?}"))?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "cli", derive(ClapArgs))]
#[cfg_attr(feature = "cli", group(required = true, multiple = true))]
pub struct Input {
    /// Input fasta alignment.
    #[cfg_attr(feature = "cli", arg(long, value_delimiter = ','))]
    pub populations: Option<Vec<String>>,

    /// Input dataset population.
    #[cfg_attr(feature = "cli", arg(long))]
    pub alignment: Option<PathBuf>,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Input { populations: None, alignment: None }
    }
}
