//! Error type returned by [`crate::scan_dir`].
//!
//! Kept separate from [`crate::SwdirError`] on purpose: `SwdirError` derives
//! `PartialEq`, but [`std::io::Error`] does not implement `PartialEq`, so we
//! cannot add an I/O variant to it without breaking the existing public
//! derive. Using a dedicated error type preserves backwards compatibility.

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Error returned by [`crate::scan_dir`].
///
/// Always carries the [`PathBuf`] the error occurred on, so callers don't
/// have to re-thread context themselves.
#[derive(Error, Debug)]
pub enum ScanError {
    /// An underlying I/O error. Covers `read_dir` failing outright
    /// (e.g. permission denied, not found, not a directory) as well as
    /// errors encountered while iterating entries or reading their types.
    #[error("I/O error at {}: {source}", path.display())]
    Io {
        /// Path the error refers to. This is the directory for `read_dir`
        /// failures, or the specific child entry for per-entry failures.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
}

impl ScanError {
    /// Construct an [`ScanError::Io`] variant.
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Path associated with the error.
    pub fn path(&self) -> &Path {
        match self {
            Self::Io { path, .. } => path,
        }
    }

    /// [`io::ErrorKind`] of the underlying I/O error.
    pub fn io_kind(&self) -> io::ErrorKind {
        match self {
            Self::Io { source, .. } => source.kind(),
        }
    }
}
