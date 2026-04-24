//! # swdir
//!
//! Swiftly traverse and scan directories recursively.
//!
//! `swdir` is a small crate that supplies the raw material for a
//! Directory Tree widget — path listings, recursive walks, typed
//! entries. It does **not** draw the tree, track file-watch events,
//! cache results, or interpret file contents; those belong to the GUI
//! layer that calls into swdir.
//!
//! Two entry points cover the common cases:
//!
//! | Use case                                    | API                                              |
//! |---------------------------------------------|--------------------------------------------------|
//! | **Recursive** walk (batch tools, CLIs)      | [`Swdir::walk`]                                  |
//! | **Lazy-loading** one-folder scan (GUIs)     | [`scan_dir`] / [`scan_dir_with_options`]         |
//!
//! Both share the same [`SortOrder`] concept when reproducible ordering
//! matters.
//!
//! ## Quick start — recursive walk
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
//! ## Quick start — lazy loading (for Directory Tree widgets)
//!
//! The intended pattern for GUIs: call [`scan_dir_with_options`] each
//! time the user expands a folder.
//!
//! ```no_run
//! use std::path::Path;
//! use swdir::{ScanOptions, SortOrder, scan_dir_with_options};
//!
//! # fn demo() -> Result<(), swdir::ScanError> {
//! let opts = ScanOptions::new(SortOrder::NameAscDirsFirst);
//! let entries = scan_dir_with_options(Path::new("/some/folder"), &opts)?;
//! for entry in &entries {
//!     // entry.display_name() — &OsStr, no allocation
//!     // entry.is_dir() — uses the cached FileType, no syscall
//!     // entry.relative_to(root) — pure path arithmetic
//! }
//! # Ok(()) }
//! ```
//!
//! [`scan_dir`] (no options) stays as-is: OS `readdir` order, no
//! sorting. Prefer [`scan_dir_with_options`] when the GUI needs a
//! deterministic display order.
//!
//! ## Ordering
//!
//! Both walk and scan accept a [`SortOrder`]:
//!
//! * [`SortOrder::Filesystem`] — OS `readdir` order; cheapest.
//! * [`SortOrder::NameAscDirsFirst`] — directories first, then files,
//!   each group sorted by name ascending. The default, because it's
//!   what a tree widget usually wants.
//!
//! ```no_run
//! use swdir::{SortOrder, Swdir};
//!
//! let report = Swdir::new()
//!     .root_path("/some/path")
//!     .sort_order(SortOrder::Filesystem) // skip the in-memory sort
//!     .walk();
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
//! [`scan_dir`] / [`scan_dir_with_options`] are **atomic** instead — on
//! the first I/O failure they return `Err(ScanError::Io { .. })`
//! carrying the offending path. The trade-off matches the use case:
//! batch walks want partial results, a GUI node expand wants
//! all-or-nothing.
//!
//! ## GUI-friendly [`DirEntry`] helpers
//!
//! [`DirEntry`] (the type returned by the scan functions) caches the
//! entry's [`std::fs::FileType`], so `is_dir()` / `is_file()` /
//! `is_symlink()` never re-syscall. 0.11 adds two thin convenience
//! methods for tree widgets:
//!
//! * [`DirEntry::display_name`] — borrowed `&OsStr` for labels
//!   (no allocation).
//! * [`DirEntry::relative_to`] — relativize against a cached root
//!   (no I/O).
//!
//! ## Migration from 0.10
//!
//! 0.11 is additive on top of 0.10. Existing 0.10 code keeps working.
//! Optional adjustments:
//!
//! * Pass [`SortOrder::Filesystem`] to [`Swdir::sort_order`] if you
//!   want to skip the default in-memory sort.
//! * Replace bespoke `strip_prefix(root).ok().map(..)` glue with
//!   [`DirEntry::relative_to`].
//! * Swap `scan_dir(path)` for `scan_dir_with_options(path,
//!   &ScanOptions::default())` if you want sorted output instead of
//!   raw OS order.
//!
//! See [CHANGELOG.md](https://github.com/nabbisen/swdir/blob/main/CHANGELOG.md)
//! for the full list.

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
        sort::{ScanOptions, SortOrder},
        walk_error::WalkError,
        walk_report::WalkReport,
    },
    scan::{scan_dir, scan_dir_with_options},
};

#[cfg(feature = "filter")]
pub use crate::helpers::filter::{Decision, EntryKind, FilterContext, FilterRule};
