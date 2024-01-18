//! Create and manipulate a row-based [`Table`].

use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use itertools::Itertools;
use rebar_utils as utils;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
// use std::default::Default;
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Write;

/// A row-based [`Table`] of generic data.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Table<T, P> {
    /// Names of the table columns.
    pub headers: Vec<T>,
    /// Rows of table values.
    pub rows: Vec<Vec<T>>,
    /// Optional file path for where the table was read from.
    pub path: Option<P>,
}

impl<T, P> Default for Table<T, P>
where
    T: Clone + Display + Debug + PartialEq<T>,
    P: AsRef<OsStr> + AsRef<std::path::Path> + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> Table<T, P>
where
    T: Clone + Display + Debug + PartialEq<T>,
    P: AsRef<OsStr> + AsRef<std::path::Path> + Debug,
{
    /// Returns a new row-based [`Table`] with empty headers and rows.
    ///
    /// ## Examples
    ///
    /// Let the compiler figure out the type from subsequent commands.
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    /// # assert_eq!(table.rows, vec![vec!["A", "B", "C"]]);
    /// ```
    ///
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    pub fn new() -> Self {
        Table { headers: Vec::new(), rows: Vec::new(), path: None }
    }

    /// Add a new row to the table.
    ///
    /// ## Arguments
    ///
    /// * `row` - A iterable object of new data (`T`) to add as a row.
    ///
    /// ## Examples
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(["A", "B", "C"]);
    /// table.add_row(["D", "E", "F"]);
    /// # assert_eq!(table.rows, [["A", "B", "C"], ["D", "E", "F"]]);
    /// ```
    ///
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    /// | D | E | F |
    ///
    pub fn add_row<I>(&mut self, row: I) -> Result<(), Report>
    where
        I: Clone + IntoIterator<Item = T>,
    {
        // if table already has rows, check that the new row is the correct length
        if !self.rows.is_empty() {
            let new = row.clone().into_iter().count();
            let ex = self.rows[0].len();
            if ex != new {
                return Err(eyre!(
                    "New row size ({new}) does not matching existing table ({ex})."
                ))?;
            }
        }
        let row = row.into_iter().collect::<Vec<T>>();
        self.rows.push(row);
        Ok(())
    }

    /// Adds a new column to the [`Table`].
    ///
    /// ## Arguments
    ///
    /// * `column` - An iterable object of new data (`T`) to add as a column.
    ///
    /// ## Examples
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(["A", "B", "C"]);
    /// table.add_column("4", vec!["D"]);
    ///
    /// assert_eq!(table.get_column(&"4")?, [&"D"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// | 1 | 2 | 3 | 4 |
    /// |---|---|---|---|
    /// | A | B | C | D |
    ///
    pub fn add_column<I>(&mut self, header: T, column: I) -> Result<(), Report>
    where
        I: Clone + IntoIterator<Item = T>,
    {
        // if table already has rows, check that the new column is the correct length
        let ex = self.rows.len();
        let new = column.clone().into_iter().count();

        if ex != new {
            return Err(eyre!("New column size ({new}) does not matching existing table ({ex})."));
        }

        self.headers.push(header);

        column.into_iter().enumerate().for_each(|(i, val)| {
            self.rows[i].push(val);
        });
        Ok(())
    }

    /// Returns the [`Table`] value under a particular header and row index.
    ///
    /// ## Arguments
    ///
    /// - `header` - Column name.
    /// - `row` - Row index (0-based).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(["A", "B", "C"]);
    ///
    /// assert_eq!(table.get(&"2", 0)?, &"B");
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    pub fn get(&self, header: &T, row: usize) -> Result<&T, Report> {
        let header_i = self.get_header_index(header)?;
        let row = self.get_row(row)?;
        Ok(&row[header_i])
    }

    /// Returns a [`Vec`] of [`Table`] values under a header.
    ///
    /// ## Arguments
    ///
    /// * `header` - Column name.
    ///
    /// ## Examples
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(["A", "B", "C"]);
    /// table.add_row(["D", "E", "F"]);
    ///
    /// assert_eq!(table.get_column(&"1")?, [&"A", &"D"]);
    /// assert_eq!(table.get_column(&"2")?, [&"B", &"E"]);
    /// assert_eq!(table.get_column(&"3")?, [&"C", &"F"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    /// | D | E | F |
    pub fn get_column(&self, header: &T) -> Result<Vec<&T>, Report> {
        let header_i = self.get_header_index(header)?;
        let column = self.rows.iter().map(|row| &row[header_i]).collect();
        Ok(column)
    }

    /// Returns the column index (0-based) of the header in the [`Table`].
    ///
    /// # Arguments
    ///
    /// * `header` - Header name.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(vec!["A", "B", "C"]);
    ///
    /// assert_eq!(table.get_header_index(&"1")?, 0);
    /// assert_eq!(table.get_header_index(&"2")?, 1);
    /// assert_eq!(table.get_header_index(&"3")?, 2);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    pub fn get_header_index(&self, header: &T) -> Result<usize, Report> {
        let pos =
            self.headers.iter().position(|h| h == header).ok_or_else(|| {
                eyre!("Column '{header}' was not found in table: {:?}.", self.path)
            })?;

        Ok(pos)
    }

    /// Return a row of [`Table`] values from a row index.
    ///
    /// ## Arguments
    ///
    /// * `row` - Row index (0-based).
    ///
    /// ## Examples
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(["A", "B", "C"]);
    /// table.add_row(["D", "E", "F"]);
    ///
    /// assert_eq!(table.get_row(0)?, ["A", "B", "C"]);
    /// assert_eq!(table.get_row(1)?, ["D", "E", "F"]);
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// | 1 | 2 | 3 |
    /// |---|---|---|
    /// | A | B | C |
    /// | D | E | F |
    pub fn get_row(&self, i: usize) -> Result<&[T], Report> {
        if i >= self.rows.len() {
            Err(eyre!("Row ({i}) does not exist in the table."))
        } else {
            Ok(&self.rows[i])
        }
    }

    /// Update all values in a row.
    ///
    /// ## Arguments
    ///
    /// * `i` - Row index (0-based)
    /// * `row` - New values for row.
    ///
    /// ## Examples
    ///
    /// ```
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(["A", "B", "C"]);
    /// table.set_row(0, ["AA", "BB", "CC"]);
    /// assert_eq!(table.get_row(0)?, ["AA", "BB", "CC"]);
    /// assert!(table.set_row(10, ["AA", "BB", "CC"]).is_err());
    /// # Ok::<(), color_eyre::eyre::Report>(())
    /// ```
    ///
    /// | 1  | 2  | 3  |
    /// |----|----|----|
    /// | AA | BB | CC |
    pub fn set_row<I>(&mut self, i: usize, row: I) -> Result<(), Report>
    where
        I: Clone + IntoIterator<Item = T>,
    {
        let new = row.clone().into_iter().count();
        if i >= new {
            return Err(eyre!("Row ({i}) does not exist in the table."));
        }
        let row = row.into_iter().collect::<Vec<T>>();
        self.rows[i] = row;
        Ok(())
    }

    //     /// Read a TSV or CSV file into a Table.
    //     ///
    //     /// # Arguments
    //     ///
    //     /// * `path` - File path.
    //     /// * `delim` - Optional delimiter. Otherwise, will be identified based on path suffix (.tsv or .csv).
    //     ///
    //     /// # Examples
    //     ///
    //     /// ```
    //     /// use rebar::Table;
    //     /// use std::io::Write;
    //     /// use tempfile::NamedTempFile;
    //     ///
    //     /// let mut file = NamedTempFile::new().unwrap();
    //     /// writeln!(file, "1\t2\t3\nA\tB\tC");
    //     /// let table = Table::read(file.path(), Some('\t')).unwrap();
    //     /// println!("{}", table.to_markdown().unwrap());
    //     /// ```
    //     /// | 1 | 2 | 3 |
    //     /// |---|---|---|
    //     /// | A | B | C |
    //     ///
    //     pub fn read(path: &Path, delim: Option<char>) -> Result<Table<String>, Report> {
    //         let mut table = Table::new();

    //         // if not provided, lookup delimiter from file extension
    //         let delim = match delim {
    //             Some(c) => c,
    //             None => utils::path_to_delim(path)?,
    //         };

    //         // attempt to open the file path
    //         let file = File::open(path).wrap_err_with(|| eyre!("Failed to read file: {path:?}"))?;

    //         // read and parse lines
    //         for line in BufReader::new(file).lines().flatten() {
    //             let row = line.split(delim).map(String::from).collect_vec();
    //             // if headers are empty, this is the first line, write headers
    //             if table.headers.is_empty() {
    //                 table.headers = row;
    //             }
    //             // otherwise regular row
    //             else {
    //                 table.rows.push(row);
    //             }
    //         }

    //         table.path = Some(path.to_path_buf());

    //         Ok(table)
    //     }

    /// Write [`Table`] to file [`Path`].
    ///
    /// ## Examples
    ///
    /// ```
    /// use tempfile::NamedTempFile;
    ///
    /// let mut table = rebar_table::Table::new();
    /// table.headers = vec!["1", "2", "3"];
    /// table.add_row(["A", "B", "C"]);
    /// table.add_row(["D", "E", "F"]);
    ///
    /// let mut file = NamedTempFile::new().unwrap();
    /// table.write(&file.path(), Some("\t"));
    /// ```
    pub fn write(&self, path: &P, delim: Option<&str>) -> Result<(), Report>
    where
        P: std::convert::AsRef<std::path::Path> + Debug,
    {
        let mut file =
            File::create(path).wrap_err_with(|| eyre!("Unable to create file: {path:?}"))?;

        // if not provided, lookup delimiter from file extension
        let path_delim = utils::path_to_delim(path)?;
        let delim = match delim {
            Some(c) => c,
            None => &path_delim,
        };

        // write headers
        let line = format!("{}\n", self.headers.iter().join(delim.to_string().as_str()));
        file.write_all(line.as_bytes())
            .wrap_err_with(|| eyre!("Unable to write table headers: {line}"))?;

        // write regular rows
        self.rows.iter().try_for_each(|row| {
            let line = format!("{}\n", row.iter().join(delim.to_string().as_str()));
            file.write_all(line.as_bytes())
                .wrap_err_with(|| format!("Unable to write table rows: {line}"))?;
            Ok::<(), Report>(())
        })?;

        Ok(())
    }
}

// /// Methods for when the table data can be compared to strings, cloned, and can be displayed.
// impl<'t, T> Table<T>
// where
//     T: PartialEq<&'t str> + Clone + std::fmt::Display,
// {
//     /// Convert table to markdown format.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use rebar::Table;
//     ///
//     /// let mut table = Table::new();
//     /// table.headers = vec!["1", "2", "3"];
//     /// table.add_row(vec!["A", "B", "C"]);
//     ///
//     /// println!("{}", table.to_markdown().unwrap());
//     /// ```
//     /// | 1 | 2 | 3 |
//     /// |---|---|---|
//     /// | A | B | C |
//     ///
//     pub fn to_markdown(&self) -> Result<String, Report> {
//         // get the maximum width of each column
//         let col_widths = self
//             // iterate through columns/headers
//             .headers
//             .iter()
//             .enumerate()
//             .map(|(col_i, header)| {
//                 let header_width = header.to_string().len();
//                 self
//                     // iterate through this column's rows,
//                     // get max string width, +2 to add space on either side
//                     .rows
//                     .iter()
//                     .map(|row| {
//                         let cell_width = row[col_i].to_string().len();
//                         if cell_width >= header_width {
//                             cell_width + 2
//                         } else {
//                             header_width + 2
//                         }
//                     })
//                     .max()
//                     .unwrap_or(header_width + 2)
//             })
//             .collect_vec();

//         let mut markdown = String::from("|");
//         // frame in between headers and rows
//         let mut header_frame = String::from("|");

//         // Create the header line
//         for it in self.headers.iter().zip(col_widths.iter()) {
//             let (header, col_width) = it;
//             let cell = format!("{:^width$}|", header, width = col_width);
//             markdown.push_str(&cell);

//             let frame = format!("{}|", "-".repeat(*col_width));
//             header_frame.push_str(&frame);
//         }
//         markdown.push('\n');
//         markdown.push_str(&header_frame);
//         markdown.push('\n');

//         // Create the row lines
//         for row in &self.rows {
//             markdown.push('|');
//             for (col_i, col_width) in col_widths.iter().enumerate() {
//                 let cell = format!("{:^width$}|", row[col_i], width = col_width);
//                 markdown.push_str(&cell);
//             }
//             markdown.push('\n');
//         }

//         Ok(markdown)
//     }
// }

// impl<T> Table<T>
// where
//     T: ToString,
// {
//     /// Create a new table with all values converted to owned String.
//     pub fn to_string_values(&self) -> Table<String> {
//         let mut table = Table::new();
//         table.headers = self.headers.iter().map(|s| s.to_string()).collect();
//         table.rows =
//             self.rows.iter().map(|row| row.iter().map(|s| s.to_string()).collect()).collect();
//         table
//     }
// }

// // impl<'t, T> Table<T>
// // where
// //     T: PartialEq<&'t str> + Clone,
// // {

// //     pub fn filter_column(&'t self, header: &'t str, pattern: &'t str) -> Result<Table<T>, Report> {
// //         let mut table = Table::new();
// //         let header_i = self.get_header_index(header)?;
// //         table.headers = self.headers.clone();
// //         table.rows = self.rows.iter().filter(|row| row[header_i] == pattern).cloned().collect_vec();
// //         Ok(table)
// //     }
// // }
