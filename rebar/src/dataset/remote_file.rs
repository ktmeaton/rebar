#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::default::Default;

// ----------------------------------------------------------------------------
// Remote File
// ----------------------------------------------------------------------------

/// A file downloaded from a remote URL.
///
/// ## Generics
///
/// - `D` - Date, recommended [`chrono::NaiveDate`].
/// - `P` - File path.
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFile<D, P> {
    /// File URL
    pub url: String,
    // Github commit SHA hash
    pub sha: String,
    // Local path of the file.
    pub local_path: P,
    // Date the file was created.
    pub date_created: D,
    // Date the file was downloaded.
    pub date_downloaded: D,
}

impl<D, P> Default for RemoteFile<D, P>
where
    D: Default,
    P: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<D, P> RemoteFile<D, P>
where
    D: Default,
    P: Default,
{
    pub fn new() -> Self {
        RemoteFile {
            url: String::new(),
            sha: String::new(),
            local_path: P::default(),
            date_created: D::default(),
            date_downloaded: D::default(),
        }
    }
}
