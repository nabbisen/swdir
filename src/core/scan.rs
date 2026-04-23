//! Parallel scan engine powering [`crate::Swdir::walk`].
//!
//! Responsibilities:
//!
//! * Walk directories in parallel via rayon.
//! * Track depth from the scan root so depth-aware filters (and
//!   [`crate::Recurse::Depth`]) work.
//! * Apply the configured [`crate::FilterRule`] stack, obeying both axes
//!   of the resulting [`crate::Decision`] (`include` and `descend`).
//! * Collect I/O errors into the walker's [`crate::WalkReport`] instead
//!   of printing them to stderr.

use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use crate::helpers::recurse::Recurse;
use crate::helpers::walk_error::WalkError;

#[cfg(feature = "filter")]
use crate::helpers::file::is_hidden;
#[cfg(feature = "filter")]
use crate::helpers::filter::{EntryKind, FilterContext, evaluate_all};

use super::DirNode;
use super::Swdir;

impl Swdir {
    /// Build the rayon pool and delegate into the recursive worker.
    pub(super) fn walk_parallel(&self, errors: &mut Vec<WalkError>) -> DirNode {
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.max_threads)
            .build()
            .expect("failed to build rayon thread pool");

        let errors_mx: Mutex<Vec<WalkError>> = Mutex::new(Vec::new());
        let root = self.root_path.clone();

        // `entry_depth` is the depth of *entries inside* the scanned
        // directory. Root-level entries are at depth 0, matching how
        // `FilterRule::MaxDepth(0)` reads: "include root-level entries,
        // don't descend further".
        let tree = pool.install(|| self.scan_node(root.as_path(), 0, &errors_mx));

        let collected = errors_mx
            .into_inner()
            .unwrap_or_else(|poison| poison.into_inner());
        errors.extend(collected);

        tree
    }

    fn scan_node(
        &self,
        dir_path: &Path,
        entry_depth: usize,
        errors: &Mutex<Vec<WalkError>>,
    ) -> DirNode {
        let read = match fs::read_dir(dir_path) {
            Ok(it) => it,
            Err(err) => {
                push_error(errors, WalkError::io(dir_path, err));
                return DirNode {
                    path: dir_path.to_path_buf(),
                    sub_dirs: vec![],
                    files: vec![],
                };
            }
        };

        let may_enter_sub_dirs = subdir_descent_allowed(self.recurse, entry_depth);

        let mut files = Vec::new();
        let mut sub_dir_paths: Vec<std::path::PathBuf> = Vec::new();

        for item in read {
            let entry = match item {
                Ok(e) => e,
                Err(err) => {
                    push_error(errors, WalkError::io(dir_path, err));
                    continue;
                }
            };

            let entry_path = entry.path();
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(err) => {
                    push_error(errors, WalkError::io(&entry_path, err));
                    continue;
                }
            };

            let is_dir = file_type.is_dir();

            // --- filter evaluation --------------------------------------
            #[cfg(feature = "filter")]
            let decision = {
                let hidden_flag = is_hidden(&entry);
                let kind = match EntryKind::from_file_type(file_type) {
                    Some(k) => k,
                    // Unknown entry kind (socket, fifo, …): skip quietly.
                    None => continue,
                };
                let ctx = FilterContext::new(&entry_path, kind, entry_depth, hidden_flag);
                evaluate_all(&self.filters, &ctx)
            };

            #[cfg(not(feature = "filter"))]
            let decision = PlainDecision {
                include: true,
                descend: true,
            };

            // --- placement ----------------------------------------------
            if is_dir {
                // Directories: respect both axes.
                //
                // We queue the child for descent iff the recurse policy
                // allows it AND the filter says `descend`. The child's
                // contribution to the tree is kept even when the filter
                // said `include = false` for this dir, because otherwise
                // matching files inside a "hidden-passthrough" dir would
                // be unreachable. Callers who want a flat result list
                // should call `DirNode::flatten_paths()`; callers who
                // care about the tree shape can inspect sub_dirs.
                if may_enter_sub_dirs && decision.descend {
                    sub_dir_paths.push(entry_path);
                }
                continue;
            }

            // Non-directories: only `include` matters.
            if decision.include {
                files.push(entry_path);
            }
        }

        // Recurse in parallel.
        let mut sub_dirs: Vec<DirNode> = sub_dir_paths
            .into_par_iter()
            .map(|path| self.scan_node(&path, entry_depth + 1, errors))
            .collect();

        sub_dirs.sort();
        files.sort();

        DirNode {
            path: dir_path.to_path_buf(),
            sub_dirs,
            files,
        }
    }
}

#[cfg(not(feature = "filter"))]
struct PlainDecision {
    include: bool,
    descend: bool,
}

fn push_error(errors: &Mutex<Vec<WalkError>>, err: WalkError) {
    match errors.lock() {
        Ok(mut guard) => guard.push(err),
        Err(poison) => {
            // A worker thread panicked earlier; the mutex is poisoned
            // but we can still record this error.
            poison.into_inner().push(err);
        }
    }
}

fn subdir_descent_allowed(recurse: Recurse, entry_depth: usize) -> bool {
    match recurse {
        Recurse::None => false,
        Recurse::Unlimited => true,
        // `Recurse::Depth(n)` admits entries at depth <= n and descent
        // through entries at depth < n. A child directory would be
        // scanned with `entry_depth + 1`, so we may enter it iff
        // `entry_depth < n` (keeping its entries at depth <= n).
        Recurse::Depth(n) => entry_depth < n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdir_descent_depth_gate() {
        assert!(!subdir_descent_allowed(Recurse::None, 0));
        assert!(subdir_descent_allowed(Recurse::Unlimited, 0));
        assert!(subdir_descent_allowed(Recurse::Unlimited, 100));

        // Depth(1): enter the root's direct children (entry_depth==0 -> yes)
        // but not their children (entry_depth==1 -> no).
        assert!(subdir_descent_allowed(Recurse::Depth(1), 0));
        assert!(!subdir_descent_allowed(Recurse::Depth(1), 1));

        // Depth(0) behaves like None.
        assert!(!subdir_descent_allowed(Recurse::Depth(0), 0));
    }
}
