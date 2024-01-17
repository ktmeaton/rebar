//! Create and manipulate the [Table].

use crate::utils;
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::default::Default;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use structdoc::StructDoc;

/// A row-based table of generic data.
///
/// # Examples
///
/// ```
/// use rebar::Table;
///
/// let mut table = Table::new();
/// table.headers = vec!["1", "2", "3"];
/// table.add_row(vec!["A", "B", "C"]);
///
/// println!("{}", table.to_markdown().unwrap());
/// ```
///
/// | 1 | 2 | 3 |
/// |---|---|---|
/// | A | B | C |
///
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, StructDoc)]
pub struct Table<T> {
    /// Names of the table columns.
    pub headers: Vec<T>,
    /// Rows of table values.
    pub rows: Vec<Vec<T>>,
    /// Optional file path for where the table was read from.
    pub path: Option<PathBuf>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'t, T> std::fmt::Display for Table<T>
where
    T: Deserialize<'t> + Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).expect("Failed to serialize table."))
    }
}

impl<T> Table<T> {
    /// Create a new table with empty headers and rows.
    pub fn new() -> Self {
        Table { headers: Vec::new(), rows: Vec::new(), path: None }
    }
}

/// Methods for when the table data can be compared to strings and displayed as a string.
impl<'t, T> Table<T>
where
    T: PartialEq<&'t str> + std::fmt::Display,
{
    /// Add a new row to the table.
    ///
    /// # Arguments
    ///
    /// * `row` - A vector of new data to add as a row.
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    /// table.add_row(vec!["D", "E", "F"]);
    ///
    /// println!("{}", table.to_markdown().unwrap());
    /// ```
    ///
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    /// | D | E | F |
    ///
    pub fn add_row(&'t mut self, row: Vec<T>) -> Result<(), Report> {
        // if table already has rows, check that the new row is the correct length
        if !self.rows.is_empty() {
            let new = row.len();
            let ex = self.rows[0].len();
            if ex != new {
                return Err(eyre!(
                    "New row size ({new}) does not matching existing table ({ex})."
                ))?;
            }
        }
        self.rows.push(row);
        Ok(())
    }

    /// Add a new column to the table.
    ///
    /// # Arguments
    ///
    /// * `column` - A vector of new data to add as a column.
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    /// table.add_column("4", vec!["D"]);
    ///
    /// println!("{}", table.to_markdown().unwrap());
    /// ```
    ///
    /// | 1 | 2 | 3 | 4 |
    /// |---|---|---|---|
    /// | A | B | C | D |
    ///
    pub fn add_column(&'t mut self, header: T, column: Vec<T>) -> Result<(), Report> {
        // if table already has rows, check that the new column is the correct length
        let ex = self.rows.len();
        let new = column.len();

        if ex != new {
            return Err(eyre!("New column size ({new}) does not matching existing table ({ex})."));
        }

        self.headers.push(header);

        column.into_iter().enumerate().for_each(|(i, val)| {
            self.rows[i].push(val);
        });
        Ok(())
    }

    /// Get table value at a particular column and row index.
    ///
    /// # Arguments
    ///
    /// * `header` - Column name.
    /// * `row` - Row index (0-based).
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    ///
    /// let expected = &"B";
    /// let observed = table.get("2", 0).unwrap();
    /// assert_eq!(expected, observed);
    /// ```
    ///
    pub fn get(&'t self, header: &'t str, row: usize) -> Result<&T, Report> {
        let header_i = self.get_header_index(header)?;
        let row = self.get_row(row)?;
        Ok(&row[header_i])
    }

    /// Return a vector of table values in a column.
    ///
    /// # Arguments
    ///
    /// * `header` - Column name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    /// table.add_row(vec!["D", "E", "F"]);
    ///
    /// let observed = table.get_column("3").unwrap();
    /// let expected = vec![&"C", &"F"];
    /// assert_eq!(expected, observed)
    /// ```
    pub fn get_column(&'t self, header: &'t str) -> Result<Vec<&T>, Report> {
        let header_i = self.get_header_index(header)?;
        let column = self.rows.iter().map(|row| &row[header_i]).collect();
        Ok(column)
    }

    /// Return a vector of table values in a row.
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (0-based).
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    /// table.add_row(vec!["D", "E", "F"]);
    ///
    /// let expected = vec!["D", "E", "F"];
    /// let observed = table.get_row(1).unwrap();
    /// assert_eq!(expected, observed)
    /// ```
    pub fn get_row(&'t self, i: usize) -> Result<&[T], Report> {
        if i >= self.rows.len() {
            Err(eyre!("Row ({i}) does not exist in the table."))
        } else {
            Ok(&self.rows[i])
        }
    }

    /// Get the column index (0-based) correponding to the header.
    ///
    /// # Arguments
    ///
    /// * `header` - Header name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    ///
    /// let expected = 2;
    /// let observed = table.get_header_index("3").unwrap();
    /// assert_eq!(expected, observed)
    /// ```
    pub fn get_header_index(&'t self, header: &'t str) -> Result<usize, Report> {
        let pos =
            self.headers.iter().position(|h| *h == header).ok_or_else(|| {
                eyre!("Column '{header}' was not found in table: {:?}.", self.path)
            })?;

        Ok(pos)
    }

    /// Update all values in a row.
    ///
    /// # Arguments
    ///
    /// * `i` - Row index (0-based)
    /// * `row` - New values for row.
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    /// table.set_row(0, vec!["AA", "BB", "CC"]);
    ///
    /// println!("{}", table.to_markdown().unwrap());
    /// ```
    /// | 1  | 2  | 3  |
    /// |----|----|----|
    /// | AA | BB | CC |
    ///
    pub fn set_row(&'t mut self, i: usize, row: Vec<T>) -> Result<(), Report> {
        if i >= self.rows.len() {
            return Err(eyre!("Row ({i}) does not exist in the table."));
        }
        self.rows[i] = row;
        Ok(())
    }

    /// Read a TSV or CSV file into a Table.
    ///
    /// # Arguments
    ///
    /// * `path` - File path.
    /// * `delim` - Optional delimiter. Otherwise, will be identified based on path suffix (.tsv or .csv).
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    /// use std::io::Write;
    /// use tempfile::NamedTempFile;
    ///
    /// let mut file = NamedTempFile::new().unwrap();
    /// writeln!(file, "1\t2\t3\nA\tB\tC");
    /// let table = Table::read(file.path(), Some('\t')).unwrap();
    /// println!("{}", table.to_markdown().unwrap());
    /// ```
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    ///
    pub fn read(path: &Path, delim: Option<char>) -> Result<Table<String>, Report> {
        let mut table = Table::new();

        // if not provided, lookup delimiter from file extension
        let delim = match delim {
            Some(c) => c,
            None => utils::path_to_delim(path)?,
        };

        // attempt to open the file path
        let file = File::open(path).wrap_err_with(|| eyre!("Failed to read file: {path:?}"))?;

        // read and parse lines
        for line in BufReader::new(file).lines().flatten() {
            let row = line.split(delim).map(String::from).collect_vec();
            // if headers are empty, this is the first line, write headers
            if table.headers.is_empty() {
                table.headers = row;
            }
            // otherwise regular row
            else {
                table.rows.push(row);
            }
        }

        table.path = Some(path.to_path_buf());

        Ok(table)
    }

    /// Write table to file.
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    /// use tempfile::NamedTempFile;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    /// table.add_row(vec!["D", "E", "F"]);
    ///
    /// let mut file = NamedTempFile::new().unwrap();
    /// table::write(table, file.path(), Some('\t'));
    /// ```
    ///
    pub fn write(table: &Table<T>, path: &Path, delim: Option<char>) -> Result<(), Report> {
        let mut file =
            File::create(path).wrap_err_with(|| format!("Unable to create file: {path:?}"))?;

        // if not provided, lookup delimiter from file extension
        let delim = match delim {
            Some(c) => c,
            None => utils::path_to_delim(path)?,
        };

        // write headers
        let line = format!("{}\n", table.headers.iter().join(delim.to_string().as_str()));
        file.write_all(line.as_bytes())
            .wrap_err_with(|| format!("Unable to write table headers: {line}"))?;

        // write regular rows
        for row in &table.rows {
            let line = format!("{}\n", row.iter().join(delim.to_string().as_str()));
            file.write_all(line.as_bytes())
                .wrap_err_with(|| format!("Unable to write table rows: {line}"))?;
        }

        Ok(())
    }
}

/// Methods for when the table data can be compared to strings, cloned, and can be displayed.
impl<'t, T> Table<T>
where
    T: PartialEq<&'t str> + Clone + std::fmt::Display,
{
    /// Convert table to markdown format.
    ///
    /// # Examples
    ///
    /// ```
    /// use rebar::Table;
    ///
    /// let mut table = Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    ///
    /// println!("{}", table.to_markdown().unwrap());
    /// ```
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    ///
    pub fn to_markdown(&self) -> Result<String, Report> {
        // get the maximum width of each column
        let col_widths = self
            // iterate through columns/headers
            .headers
            .iter()
            .enumerate()
            .map(|(col_i, header)| {
                let header_width = header.to_string().len();
                self
                    // iterate through this column's rows,
                    // get max string width, +2 to add space on either side
                    .rows
                    .iter()
                    .map(|row| {
                        let cell_width = row[col_i].to_string().len();
                        if cell_width >= header_width {
                            cell_width + 2
                        } else {
                            header_width + 2
                        }
                    })
                    .max()
                    .unwrap_or(header_width + 2)
            })
            .collect_vec();

        let mut markdown = String::from("|");
        // frame in between headers and rows
        let mut header_frame = String::from("|");

        // Create the header line
        for it in self.headers.iter().zip(col_widths.iter()) {
            let (header, col_width) = it;
            let cell = format!("{:^width$}|", header, width = col_width);
            markdown.push_str(&cell);

            let frame = format!("{}|", "-".repeat(*col_width));
            header_frame.push_str(&frame);
        }
        markdown.push('\n');
        markdown.push_str(&header_frame);
        markdown.push('\n');

        // Create the row lines
        for row in &self.rows {
            markdown.push('|');
            for (col_i, col_width) in col_widths.iter().enumerate() {
                let cell = format!("{:^width$}|", row[col_i], width = col_width);
                markdown.push_str(&cell);
            }
            markdown.push('\n');
        }

        Ok(markdown)
    }
}

impl<T> Table<T>
where
    T: ToString,
{
    /// Create a new table with all values converted to owned String.
    pub fn to_string_values(&self) -> Table<String> {
        let mut table = Table::new();
        table.headers = self.headers.iter().map(|s| s.to_string()).collect();
        table.rows =
            self.rows.iter().map(|row| row.iter().map(|s| s.to_string()).collect()).collect();
        table
    }
}

// impl<'t, T> Table<T>
// where
//     T: PartialEq<&'t str> + Clone,
// {

//     pub fn filter_column(&'t self, header: &'t str, pattern: &'t str) -> Result<Table<T>, Report> {
//         let mut table = Table::new();
//         let header_i = self.get_header_index(header)?;
//         table.headers = self.headers.clone();
//         table.rows = self.rows.iter().filter(|row| row[header_i] == pattern).cloned().collect_vec();
//         Ok(table)
//     }
// }
