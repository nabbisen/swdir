//! # swdir
//!
//! Swiftly traverse and scan directories in Rust.
//!
//! `swdir` is a small, focused crate for walking and listing
//! directories. It supplies the raw material for CLIs and GUI
//! Directory Tree widgets — path listings, recursive walks, typed
//! entries — and deliberately stops there.
//!
//! ## Two entry points
//!
//! | Use case                                    | API                                              |
//! |---------------------------------------------|--------------------------------------------------|
//! | **Recursive** walk (batch tools, CLIs)      | [`Swdir::walk`]                                  |
//! | **Lazy-loading** one-folder scan (GUIs)     | [`scan_dir`] / [`scan_dir_with_options`]         |
//!
//! Both share the same [`SortOrder`] when reproducible ordering matters.
//!
//! ## Quick start — recursive walk
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
//! ```no_run
//! use std::path::Path;
//! use swdir::{ScanOptions, SortOrder, scan_dir_with_options};
//!
//! # fn demo() -> Result<(), swdir::ScanError> {
//! let opts = ScanOptions::new(SortOrder::NameAscDirsFirst);
//! let entries = scan_dir_with_options(Path::new("/some/folder"), &opts)?;
//! for entry in &entries {
//!     // entry.display_name() — &OsStr, no allocation
//!     // entry.is_dir()       — uses the cached FileType, no syscall
//! }
//! # Ok(()) }
//! ```
//!
//! ## With a filter (default feature)
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
//!     .walk();
//! # Ok(()) }
//! # }
//! ```
//!
//! `Swdir::new()` installs one rule by default ([`FilterRule::SkipHidden`])
//! so dotfiles don't appear in results. Call [`Swdir::clear_filters`] to
//! see them.
//!
//! ## Error handling
//!
//! [`Swdir::walk`] returns a [`WalkReport`] with errors collected —
//! unreadable directories never cause panics or stderr noise:
//!
//! ```no_run
//! # use swdir::Swdir;
//! let report = Swdir::new().root_path(".").walk();
//! for err in &report.errors {
//!     eprintln!("warn: {err}");
//! }
//! ```
//!
//! [`scan_dir`] / [`scan_dir_with_options`] are atomic instead: the
//! first I/O failure returns [`Err(ScanError::Io)`][ScanError] without
//! partial results.
//!
//! ## Further reading
//!
//! The crate ships a full set of guides and reference pages in
//! [`docs/`][docs] in the repository:
//!
//! * **Getting started** — [`docs/getting-started.md`][gs]
//! * **Recursive walks** — [`docs/guides/recursive-walk.md`][gw]
//! * **Lazy loading for GUIs** — [`docs/guides/lazy-loading.md`][gl]
//! * **Filter rules** (catalog + `include`/`descend` model) — [`docs/reference/filter-rules.md`][rf]
//! * **Sort order** — [`docs/reference/sort-order.md`][rs]
//! * **Error handling** — [`docs/reference/error-handling.md`][re]
//! * **Design notes** — [`docs/design-notes.md`][dn]
//! * **Migration notes** — [`docs/migration/`][mg]
//!
//! [docs]: https://github.com/nabbisen/swdir/tree/main/docs
//! [gs]:   https://github.com/nabbisen/swdir/blob/main/docs/getting-started.md
//! [gw]:   https://github.com/nabbisen/swdir/blob/main/docs/guides/recursive-walk.md
//! [gl]:   https://github.com/nabbisen/swdir/blob/main/docs/guides/lazy-loading.md
//! [rf]:   https://github.com/nabbisen/swdir/blob/main/docs/reference/filter-rules.md
//! [rs]:   https://github.com/nabbisen/swdir/blob/main/docs/reference/sort-order.md
//! [re]:   https://github.com/nabbisen/swdir/blob/main/docs/reference/error-handling.md
//! [dn]:   https://github.com/nabbisen/swdir/blob/main/docs/design-notes.md
//! [mg]:   https://github.com/nabbisen/swdir/tree/main/docs/migration

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
