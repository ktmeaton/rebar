use color_eyre::eyre::{eyre, ContextCompat, Report, Result, WrapErr};
use color_eyre::Help;
use std::convert::AsRef;
use std::fmt::Debug;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use zstd::stream::read::Decoder;

pub enum Decompress {
    Zst,
}

impl FromStr for Decompress {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self, Report> {
        match s {
            "zst" => Ok(Decompress::Zst),
            _ext => Err(eyre!("Decompression for {_ext:?} is not implemented yet.")),
        }
    }
}

/// Decompress file.
///
/// ## Arguments
///
/// - `input` - Input file path.
/// - `output` - Optional output file path, if [`None`] then decompresses in place.
///
/// ## Examples
///
/// ```rust
/// # use tokio_test::{block_on, assert_ok};
/// # use rebar::utils::{download_file, decompress_file};
/// let url = "https://raw.githubusercontent.com/corneliusroemer/pango-sequences/a8596a6/data/pango-consensus-sequences_genome-nuc.fasta.zst";
/// # assert_ok!(block_on(async {
/// let output = "test/utils/decompress_file/pango-consensus-sequences_genome-nuc.fasta.zst";
/// download_file(&url, &output, false).await?;
/// decompress_file(&output)?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn decompress_file<P>(path: &P) -> Result<PathBuf, Report>
where
    P: AsRef<Path> + Debug,
{
    // extract the output file path without the last extension
    let output = path.as_ref().with_extension("");

    // select decompress algorithm based on extension
    let ext = path
        .as_ref()
        .extension()
        .wrap_err("Failed to get file extension: {path:?}")?
        .to_str()
        .wrap_err("Failed to convert file extension to str: {path:?}")?;

    match Decompress::from_str(ext)? {
        Decompress::Zst => {
            let reader = std::fs::File::open(path).wrap_err(format!("Failed to open: {path:?}"))?;
            let mut decoder =
                Decoder::new(reader).wrap_err(format!("Failed to decode: {path:?}"))?;
            let mut buffer = String::new();
            decoder
                .read_to_string(&mut buffer)
                .wrap_err(format!("Failed to decompress: {path:?}"))?;
            std::fs::write(&output, buffer).wrap_err(eyre!("Failed to write: {output:?}"))?;
            std::fs::remove_file(path)?;
        }
    }

    Ok(output)
}

/// Download file from url to path, with optional decompression.
///
/// ## Arguments
///
/// ## Examples
///
/// Download as text (`true`).
///
/// ```rust
/// # use tokio_test::{block_on, assert_ok};
/// use rebar::utils::download_file;
///
/// let url = "https://raw.githubusercontent.com/nextstrain/ncov/v13/data/references_sequences.fasta";
/// # assert_ok!(block_on(async {
/// let output = "test/utils/download_file/reference.fasta";
/// download_file(&url, &output, true).await?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
///
/// Download as binary (`false`).
///
/// ```rust
/// # use tokio_test::{block_on, assert_ok};
/// # use rebar::utils::download_file;
/// let url = "https://raw.githubusercontent.com/corneliusroemer/pango-sequences/a8596a6/data/pango-consensus-sequences_genome-nuc.fasta.zst";
/// # assert_ok!(block_on(async {
/// let output = "test/utils/download_file/pango-consensus-sequences_genome-nuc.fasta.zst";
/// download_file(&url, &output, false).await?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// # }));
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
#[cfg(feature = "download")]
pub async fn download_file<P>(
    url: &str,
    output: &P,
    download_as_text: bool,
) -> Result<PathBuf, Report>
where
    P: AsRef<Path> + Debug,
{
    // convert from generics to Path and PathBuf
    let output: PathBuf = output.as_ref().into();

    // check if we need to create an output directory
    if let Some(output_dir) = output.parent() {
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }
    }

    // get URL response
    let response = reqwest::get(url).await?;
    if response.status() != 200 {
        Err(eyre!("Failed to download: {url:?})")
            .suggestion(format!("Status code: {}", response.status())))?;
    }

    match download_as_text {
        true => std::fs::write(&output, response.text().await?),
        false => std::fs::write(&output, response.bytes().await?),
    }
    .wrap_err(format!("Unable to write: {output:?}"))?;

    Ok(output)
}

/// Get delimiter based on file extension.
///
/// ## Arguments
///
/// - `path` - File path.
///
/// ## Examples
///
/// - `.tsv` => `\t`
/// - `.txt` => `\t`
/// - `.csv` => `,`
///
/// Note that `.txt` is assumed to be tab-delimited!
///
/// ```rust
/// use rebar::utils::get_delimiter;
///
/// assert_eq!(get_delimiter(&"file.tsv")?, '\t');
/// assert_eq!(get_delimiter(&"file.csv")?, ',');
/// assert_eq!(get_delimiter(&"file.txt")?, '\t');
/// assert!(get_delimiter(&"file").is_err());
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn get_delimiter<P>(path: &P) -> Result<char, Report>
where
    P: AsRef<Path> + Debug,
{
    let ext = path
        .as_ref()
        .extension()
        .wrap_err("Failed to get file extension: {path:?}")?
        .to_str()
        .wrap_err("Failed to convert file extension to str: {path:?}")?;
    // convert extension to the expected delimiter
    match ext {
        "tsv" | "txt" => Ok('\t'),
        "csv" => Ok(','),
        _ext => {
            Err(eyre!("Unknown file extension: {_ext:?}").suggestion("Options: tsv, csv, or txt"))
        }
    }
}

// pub use crate::table::Table;

// use crate::dataset::attributes::Tag;
// use crate::utils::remote_file::RemoteFile;
// use chrono::prelude::*;
// use color_eyre::eyre::{eyre, Report, Result, WrapErr};
// use color_eyre::Help;
// use itertools::Itertools;
// use log::{debug, warn};
// use reqwest::header::{ACCESS_CONTROL_EXPOSE_HEADERS, USER_AGENT};
// use std::collections::BTreeMap;
// use std::fs::{remove_file, write, File};
// use std::io::Read;
// use std::path::{Path, PathBuf};
// use tempfile::TempDir;
// //use zip::ZipArchive;
// use zstd::stream::read::Decoder;

// /// Download file from url to path, with optional decompression.
// pub async fn download_file(url: &str, output_path: &Path, decompress: bool) -> Result<(), Report> {
//     let ext = Path::new(&url).extension().unwrap().to_str().unwrap();

//     let response = reqwest::get(url).await?;
//     if response.status() != 200 {
//         return Err(eyre!("Unable to download file: {url}\nStatus code {}.", response.status()));
//     }

//     if decompress {
//         // Write bytes to a tmp file
//         let tmp_dir = TempDir::new()?;
//         let tmp_path = PathBuf::from(tmp_dir.path()).join(format!("tmpfile.{ext}"));
//         let content = response.bytes().await?;
//         write(&tmp_path, content).wrap_err_with(|| eyre!("Unable to write file: {tmp_path:?}"))?;
//         decompress_file(&tmp_path, output_path, true)?;
//     } else {
//         let content = response.text().await?;
//         write(output_path, content)
//             .wrap_err_with(|| eyre!("Unable to write file: {output_path:?}"))?;
//     }

//     Ok(())
// }

// #[allow(dead_code)]
// pub fn check_github_response(response: &reqwest::Response) -> Result<(), Report> {
//     let url = response.url().to_string();

//     if !response.status().is_success() {
//         // --------------------------------------------------------------------
//         // Check if the API rate limit was exceeded

//         // todo!() this is some pretty risky unwrapping here
//         let rate_limit_remaining: u32 =
//             response.headers().get("x-ratelimit-remaining").unwrap().to_str()?.parse().unwrap();
//         if rate_limit_remaining == 0 {
//             let rate_limit_reset: i64 =
//                 response.headers().get("x-ratelimit-reset").unwrap().to_str()?.parse().unwrap();
//             let rate_limit_reset: DateTime<Local> =
//                 DateTime::<Utc>::from_timestamp(rate_limit_reset, 0)
//                     .expect("invalid timestamp")
//                     .into();

//             return Err(
//                 eyre!("GitHub API rate limit has been exceeded.")
//                 .suggestion(format!("Please wait for the rate limit to reset at: {rate_limit_reset:?}"))
//                 .suggestion("Alternatively, set the environment variables GITHUB_USERNAME and GITHUB_TOKEN.")
//                 .suggestion("https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens")
//             );
//         }
//         // --------------------------------------------------------------------
//         // Other unhandled errors
//         else {
//             return Err(eyre!(
//                 "GitHub query had status code {status}: {url}",
//                 status = response.status()
//             ));
//         }
//     }

//     Ok(())
// }

// /// Query and download files using the GitHub API
// #[allow(dead_code)]
// pub async fn download_github(
//     repo: &str,
//     tag: &Tag,
//     remote_path: &str,
//     output_path: &Path,
//     sha: &Option<String>,
// ) -> Result<RemoteFile, Report> {
//     // GitHub API Setup
//     let github_token: Option<String> = std::env::var("GITHUB_TOKEN").ok();
//     let github_username = std::env::var("GITHUB_USERNAME").unwrap_or("".to_string());
//     let github_api_version = "2022-11-28";
//     let user_agent = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
//     let client = reqwest::Client::new();

//     // GitHub API Query
//     let mut query = vec![("path", remote_path), ("per_page", "1"), ("page", "1")];
//     // if manual SHA was specified
//     if let Some(sha) = sha {
//         query.push(("sha", sha));
//     }

//     // convert to string
//     let mut query =
//         query.into_iter().map(|(k, v)| (String::from(k), String::from(v))).collect_vec();

//     // Note: the reqwest::RequestBuilder doesn't implement Clone for the
//     // non-blocking (asynchronous) version :( we're going to have to full
//     // define the request over and over again.

//     // --------------------------------------------------------------------------
//     // STEP 1: Archive Pagination

//     // Use the Archive Tag as a maximum date filter (&until=...)
//     if matches!(tag, Tag::Archive(_)) {
//         query.push(("until".to_string(), tag.to_string()));
//     }

//     let request = client
//         .get(format!("https://api.github.com/repos/{repo}/commits"))
//         .query(&query)
//         .header(USER_AGENT, &user_agent)
//         .header(ACCESS_CONTROL_EXPOSE_HEADERS, "Link")
//         .header("X-GitHub-Api-Version", github_api_version)
//         .basic_auth(&github_username, github_token.clone());
//     let response = request.send().await?;
//     check_github_response(&response)?;

//     let url = response.url().to_string();

//     // extract the "sha" and "date" key from the json body
//     let body: Vec<BTreeMap<String, serde_json::Value>> = response.json().await?;
//     if body.is_empty() {
//         return Err(eyre!("No GitHub commits were found for: {}", url)
//             .suggestion(format!("Does your dataset tag ({tag}) predate the creation of this file?"))
//             .suggestion(format!("Repository: https://github.com/{repo}, File: {remote_path:?}")));
//     }

//     let sha = body[0]["sha"].to_string().replace('"', "");
//     let commit_date = body[0]["commit"]["author"]["date"].to_string().replace('"', "");
//     let date_created: DateTime<Utc> = DateTime::parse_from_rfc3339(&commit_date)?.into();

//     // --------------------------------------------------------------------------
//     // STEP 2: DOWNLOAD

//     let download_url = format!("https://raw.githubusercontent.com/{repo}/{sha}/{remote_path}");

//     // Identify decompression mode
//     // TBD! todo!() make this an enum of implemented decompression types
//     let ext = path_to_ext(Path::new(&download_url))?;
//     let decompress = ext == "zst";

//     // Download the file
//     debug!("Downloading file: {download_url} to {output_path:?}");
//     download_file(&download_url, output_path, decompress).await?;

//     // Store all the information about the remote file for the dataset summary
//     let remote_file = RemoteFile {
//         url: download_url,
//         sha,
//         local_path: output_path.to_path_buf(),
//         date_created,
//         date_downloaded: Utc::now(),
//     };
//     debug!("Downloaded file: {remote_file:?}");

//     Ok(remote_file)
// }

// #[allow(dead_code)]
// pub fn ext_to_delim(ext: &str) -> Result<char, Report> {
//     let delim = match ext {
//         "tsv" => '\t',
//         "csv" => ',',
//         "txt" => {
//             warn!("File extension .txt is assumed to be tab-delimited.");
//             '\t'
//         }
//         _ => {
//             return Err(
//                 eyre!("Unknown file extension: {ext:?}").suggestion("Options are tsv or csv.")
//             )
//         }
//     };

//     Ok(delim)
// }

// // ----------------------------------------------------------------------------
// // Remote File
// // ----------------------------------------------------------------------------

// #[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructDoc)]
// pub struct RemoteFile {
//     pub url: String,
//     pub sha: String,
//     pub local_path: PathBuf,
//     #[structdoc(skip)]
//     pub date_created: DateTime<Utc>,
//     #[structdoc(skip)]
//     pub date_downloaded: DateTime<Utc>,
// }

// impl Default for RemoteFile {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl RemoteFile {
//     pub fn new() -> Self {
//         RemoteFile {
//             url: String::new(),
//             sha: String::new(),
//             local_path: PathBuf::new(),
//             date_created: DateTime::default(),
//             date_downloaded: DateTime::default(),
//         }
//     }
// }
