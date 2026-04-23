//! # swdir
//!
//! Swiftly traverse and scan directories recursively.
//!
//! ## Quick start
//!
//! ```sh
//! cargo add swdir
//! ```
//!
//! ```no_run
//! use swdir::Swdir;
//!
//! let report = Swdir::new().root_path("/some/path").walk();
//! // -> WalkReport { tree: DirNode, errors: Vec<WalkError> }
//! let paths = report.tree.flatten_paths();
//! ```
//!
//! ## Recursion
//!
//! ```no_run
//! use swdir::{Recurse, Swdir};
//!
//! let report = Swdir::new()
//!     .root_path("/some/path")
//!     .recurse(Recurse::Depth(1)) // enter first-level subdirs only
//!     .walk();
//! ```
//!
//! ## Filtering
//!
//! Filtering is part of the default `filter` feature. Every filter is a
//! [`FilterRule`]; rules compose with AND.
//!
//! ```no_run
//! # #[cfg(feature = "filter")] {
//! use swdir::{FilterRule, Recurse, Swdir, SwdirError};
//!
//! # fn demo() -> Result<(), SwdirError> {
//! let report = Swdir::new()
//!     .root_path("/some/path")
//!     .recurse(Recurse::Unlimited)
//!     .filter(FilterRule::extension_allowlist(["md", "rs"])?)
//!     .filter(FilterRule::max_depth(3))
//!     .walk();
//! # Ok(()) }
//! # }
//! ```
//!
//! To see hidden entries, clear the default rules first:
//!
//! ```no_run
//! # #[cfg(feature = "filter")] {
//! use swdir::Swdir;
//!
//! let report = Swdir::new()
//!     .root_path("/some/path")
//!     .clear_filters()       // drops the default `SkipHidden`
//!     .walk();
//! # }
//! ```
//!
//! ### `include` vs `descend`
//!
//! Each [`FilterRule`] returns a [`Decision`] with two independent axes:
//! `include` (appears in results) and `descend` (the walker keeps going
//! through its children). For example, `FilterRule::only_kind(File)`
//! hides directories from the output but still looks inside them so the
//! files they contain remain reachable.
//!
//! ## Error handling
//!
//! [`Swdir::walk`] returns a [`WalkReport`]. Unreadable directories are
//! recorded in `report.errors` rather than printed to stderr, so callers
//! can handle partial failures programmatically:
//!
//! ```no_run
//! use swdir::Swdir;
//!
//! let report = Swdir::new().root_path(".").walk();
//! for err in &report.errors {
//!     eprintln!("warn: {err}");
//! }
//! ```
//!
//! ## Single-directory scan (GUI / lazy loading)
//!
//! For GUIs that expand one directory at a time, use the low-level
//! [`scan_dir`] function. It lists one level, does no filtering or
//! sorting, and uses neither rayon nor async.
//!
//! ```no_run
//! use std::path::Path;
//! use swdir::scan_dir;
//!
//! # fn demo() -> Result<(), swdir::ScanError> {
//! let entries = scan_dir(Path::new("."))?;
//! for entry in entries {
//!     if entry.is_dir() {
//!         // expand on user click...
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! ## Migration from 0.9
//!
//! 0.10 introduces a few deliberate breaks to unify filtering and
//! surface partial failures. Rough map of the renames:
//!
//! | 0.9                                        | 0.10                                                    |
//! |--------------------------------------------|---------------------------------------------------------|
//! | `Swdir::default().set_root_path(p)`        | `Swdir::new().root_path(p)`                             |
//! | `.set_recurse(Recurse { enabled, depth_limit })` | `.recurse(Recurse::None \| Recurse::Unlimited \| Recurse::Depth(n))` |
//! | `.include_hidden()`                        | `.clear_filters()`  *(or drop `FilterRule::SkipHidden`)* |
//! | `.set_extension_allowlist(&["md"])`        | `.filter(FilterRule::extension_allowlist(["md"])?)`     |
//! | `.set_extension_denylist(&["md"])`         | `.filter(FilterRule::extension_denylist(["md"])?)`      |
//! | `walk() -> DirNode`                        | `walk() -> WalkReport { tree, errors }` *(or `walk_tree()`)* |
//! | stderr warnings on I/O failure             | `WalkReport::errors`                                    |
//! | `SwdirError::DuplicateExtensionList`       | *(removed — allow/deny stack freely)*                   |

mod core;
mod helpers;
mod scan;

pub use crate::{
    core::Swdir,
    helpers::{
        dir_entry::DirEntry,
        dir_node::{DirNode, dir_node_count::DirNodeCount},
        error::SwdirError,
        recurse::Recurse,
        scan_error::ScanError,
        walk_error::WalkError,
        walk_report::WalkReport,
    },
    scan::scan_dir,
};

#[cfg(feature = "filter")]
pub use crate::helpers::filter::{Decision, EntryKind, FilterContext, FilterRule};
