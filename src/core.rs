//! The [`Swdir`] builder — the main entry point for recursive scans.
//!
//! 0.10 changes from 0.9:
//!
//! * Builder methods take `mut self` and return `Self`, dropping the `set_`
//!   prefix. Chains read top-to-bottom: `Swdir::new().root_path(..).filter(..)`.
//! * `include_hidden()`, `set_extension_allowlist()`, `set_extension_denylist()`
//!   are gone. Their jobs are done by [`FilterRule`] (feature `filter`).
//! * `walk()` returns [`WalkReport`] instead of a bare [`DirNode`], making
//!   partial failures observable. The old `eprintln!`-to-stderr behavior
//!   is gone.

use std::path::PathBuf;

mod scan;

use crate::helpers::dir_node::DirNode;
use crate::helpers::recurse::Recurse;
use crate::helpers::sort::SortOrder;
use crate::helpers::walk_error::WalkError;
use crate::helpers::walk_report::WalkReport;

#[cfg(feature = "filter")]
use crate::helpers::filter::FilterRule;

const MAX_THREADS: usize = 8;

/// Configures and runs a recursive directory scan.
///
/// See the crate-level docs for worked examples.
#[derive(Clone, Debug)]
pub struct Swdir {
    root_path: PathBuf,
    recurse: Recurse,
    max_threads: usize,
    sort_order: SortOrder,
    #[cfg(feature = "filter")]
    filters: Vec<FilterRule>,
}

impl Swdir {
    /// Create a new [`Swdir`] with the default configuration.
    ///
    /// The default installs a single [`FilterRule::SkipHidden`] so the
    /// common case ("scan this tree, skip dotfiles") is a one-liner. To
    /// see hidden entries, call [`Swdir::clear_filters`] before — or
    /// instead of — adding your own.
    ///
    /// The default [`SortOrder`] is [`SortOrder::NameAscDirsFirst`],
    /// matching the most common GUI expectation. Override with
    /// [`Swdir::sort_order`].
    pub fn new() -> Self {
        #[cfg(feature = "filter")]
        {
            Self {
                root_path: PathBuf::from("."),
                max_threads: MAX_THREADS,
                recurse: Recurse::default(),
                sort_order: SortOrder::default(),
                filters: vec![FilterRule::SkipHidden],
            }
        }
        #[cfg(not(feature = "filter"))]
        {
            Self {
                root_path: PathBuf::from("."),
                max_threads: MAX_THREADS,
                recurse: Recurse::default(),
                sort_order: SortOrder::default(),
            }
        }
    }

    /// Set the scan root.
    pub fn root_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.root_path = path.into();
        self
    }

    /// Set the recursion policy.
    pub fn recurse(mut self, recurse: Recurse) -> Self {
        self.recurse = recurse;
        self
    }

    /// Cap the size of the internal rayon thread pool.
    ///
    /// Defaults to 8. Lower this if you have other CPU work sharing the
    /// process; raise it if you're scanning huge, deep trees on fast
    /// storage and profiling shows it helps.
    pub fn max_threads(mut self, n: usize) -> Self {
        self.max_threads = n.max(1);
        self
    }

    /// Choose the order of entries in the resulting [`WalkReport`].
    ///
    /// Two options, on purpose — see [`SortOrder`]. Defaults to
    /// [`SortOrder::NameAscDirsFirst`]; pass [`SortOrder::Filesystem`]
    /// when you want OS `readdir` order preserved (cheaper but
    /// non-deterministic across runs / filesystems).
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }

    /// Append one filter rule. Rules compose with AND.
    ///
    /// Available only with the default `filter` feature.
    #[cfg(feature = "filter")]
    pub fn filter(mut self, rule: FilterRule) -> Self {
        self.filters.push(rule);
        self
    }

    /// Append several filter rules at once.
    #[cfg(feature = "filter")]
    pub fn filters<I: IntoIterator<Item = FilterRule>>(mut self, rules: I) -> Self {
        self.filters.extend(rules);
        self
    }

    /// Remove every filter rule, *including* the default
    /// [`FilterRule::SkipHidden`]. Use this when you want to start from a
    /// blank slate (or simply want to see hidden entries).
    #[cfg(feature = "filter")]
    pub fn clear_filters(mut self) -> Self {
        self.filters.clear();
        self
    }

    /// Borrow the currently-installed filter rules.
    #[cfg(feature = "filter")]
    pub fn filter_rules(&self) -> &[FilterRule] {
        &self.filters
    }

    /// Borrow the current recursion policy.
    pub fn recurse_policy(&self) -> Recurse {
        self.recurse
    }

    /// Borrow the current sort-order policy.
    pub fn sort_order_policy(&self) -> SortOrder {
        self.sort_order
    }

    /// Run the scan and return a [`WalkReport`] carrying both the tree
    /// and any errors encountered along the way.
    pub fn walk(&self) -> WalkReport {
        let mut errors: Vec<WalkError> = Vec::new();
        let tree = self.walk_parallel(&mut errors);
        WalkReport { tree, errors }
    }

    /// Convenience: run the scan and return just the tree. Errors, if
    /// any, are discarded. Prefer [`Swdir::walk`] in real code.
    pub fn walk_tree(&self) -> DirNode {
        self.walk().tree
    }
}

impl Default for Swdir {
    fn default() -> Self {
        Self::new()
    }
}
