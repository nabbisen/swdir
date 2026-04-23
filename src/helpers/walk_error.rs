//! Runtime error reported from [`crate::Swdir::walk`].
//!
//! This is a *collected* error type — walk is best-effort, so a single
//! unreadable directory does not abort the whole traversal. Each problem
//! the walker hits is recorded here and handed back via
//! [`crate::WalkReport::errors`]. Replaces the silent `eprintln!`-to-stderr
//! behavior of 0.9, which made it easy to miss permission-denied subtrees
//! in production usage.
//!
//! For the *atomic* error type used by the single-directory
//! [`crate::scan_dir`] API, see [`crate::ScanError`].

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// An error that occurred while walking the tree.
///
/// Always carries the offending path so callers don't have to re-thread
/// context themselves.
#[derive(Error, Debug)]
pub enum WalkError {
    /// An underlying I/O error. Covers `read_dir` failing outright
    /// (e.g. permission denied, not found) as well as errors encountered
    /// while iterating entries or reading their file types.
    #[error("I/O error at {}: {source}", path.display())]
    Io {
        /// Path the error refers to. For `read_dir` failures this is the
        /// directory being listed; for per-entry failures it is the child.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
}

impl WalkError {
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
