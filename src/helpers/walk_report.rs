//! Output of [`crate::Swdir::walk`] — the tree *and* any errors encountered.
//!
//! 0.9's `walk()` returned just [`crate::DirNode`] and printed I/O failures
//! to stderr. 0.10 returns this explicit report so callers can decide
//! programmatically how to handle partial failures.

use crate::helpers::dir_node::DirNode;
use crate::helpers::walk_error::WalkError;

/// Result of a [`crate::Swdir::walk`] call.
///
/// `tree` always represents the successfully-scanned portion of the
/// directory tree (on-error dirs appear in it with empty `files`/`sub_dirs`
/// — the walk does not unwind). `errors` lists every I/O problem hit
/// along the way, in no particular order.
///
/// For most callers, the two-step `let report = ...; let tree = report.tree;`
/// pattern is noise-free; when you care, match on `report.errors.is_empty()`.
#[derive(Debug)]
pub struct WalkReport {
    /// The scanned directory tree.
    pub tree: DirNode,
    /// I/O errors encountered during the walk, in no guaranteed order.
    /// An empty `Vec` means the walk finished cleanly.
    pub errors: Vec<WalkError>,
}

impl WalkReport {
    /// `true` if no errors were recorded during the walk.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Return the tree, discarding any errors. Useful in tests / scripts
    /// where partial results are fine.
    pub fn into_tree(self) -> DirNode {
        self.tree
    }
}
