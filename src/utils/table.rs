use crate::utils;
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use itertools::Itertools;
use std::cmp::PartialEq;
use std::default::Default;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Table<T>
where
    T: PartialEq + ToString,
{
    pub headers: Vec<T>,
    pub rows: Vec<Vec<T>>,
    pub path: PathBuf,
}

impl<T> Default for Table<T>
where
    T: PartialEq + ToString,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Table<T>
where
    T: PartialEq + ToString,
{
    pub fn new() -> Self {
        Table {
            path: PathBuf::new(),
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn read(path: &Path) -> Result<Table<String>, Report> {
        let mut table = Table::new();

        // lookup delimiter from file extension
        let delim = utils::path_to_delim(path)?;

        // attempt to open the file path
        let file = File::open(path).wrap_err_with(|| eyre!("Failed to read file: {path:?}"))?;

        // read and parse lines
        for line in BufReader::new(file).lines().flatten() {
            let row = line.split(delim).collect_vec().into_iter().map(String::from).collect_vec();
            // if headers are empty, this is the first line, write headers
            if table.headers.is_empty() {
                table.headers = row;
            }
            // otherwise regular row
            else {
                table.rows.push(row);
            }
        }

        table.path = path.to_path_buf();

        Ok(table)
    }

    pub fn header_position(&self, header: &str) -> Result<usize, Report> {
        let pos =
            self.headers.iter().position(|h| h == header).ok_or_else(|| {
                eyre!("Column '{header}' was not found in table: {:?}.", self.path)
            })?;

        Ok(pos)
    }

    pub fn get(&self, header: &str) -> Result<&[T], Report> {
        let header_i = self.header_position(header)?;
        Ok(&self.rows[header_i])
    }

    pub fn set(&mut self, header: &str, row: usize, value: T) -> Result<(), Report> {
        let header_i = self.header_position(header)?;
        self.rows[row][header_i] = value;
        Ok(())
    }

    pub fn filter(&self, header: &str, pattern: &str) -> Result<Table<T>, Report> {
        let mut table = Table::new();
        let header_i = self.header_position(header)?;
        table.headers = self.headers.clone();
        table.rows = self.rows.iter().filter(|row| row[header_i] == pattern).cloned().collect_vec();
        Ok(table)
    }

    /// write to file
    pub fn write(&self, path: &Path) -> Result<(), Report> {
        let mut file =
            File::create(path).wrap_err_with(|| format!("Unable to create file: {path:?}"))?;

        // Parse line delimiter from file extension
        let delim = utils::path_to_delim(path)?.to_string();

        // write headers
        let line = format!("{}\n", self.headers.iter().join(&delim));
        file.write_all(line.as_bytes())
            .wrap_err_with(|| format!("Unable to write table headers: {line}"))?;

        // write regular rows
        for row in &self.rows {
            let line = format!("{}\n", row.iter().join(&delim));
            file.write_all(line.as_bytes())
                .wrap_err_with(|| format!("Unable to write table rows: {line}"))?;
        }

        Ok(())
    }

    /// Convert table to markdown format
    ///
    /// TBD: error handling for empty rows!
    pub fn to_markdown(&self) -> Result<String, Report> {
        // get the maximum width of each column
        let col_widths = self
            // iterate through columns/headers
            .headers
            .iter()
            .enumerate()
            .map(|(col_i, header)| {
                self
                    // iterate through this column's rows,
                    // get max string width, +2 to add space on either side
                    .rows
                    .iter()
                    .map(|row| {
                        let cell_width = (*row[col_i]).len();
                        if cell_width >= header.len() {
                            cell_width + 2
                        } else {
                            header.len() + 2
                        }
                    })
                    .max()
                    .unwrap_or(header.len() + 2)
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
