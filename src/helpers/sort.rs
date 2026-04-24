//! Result-ordering policy for [`crate::Swdir::walk`] and
//! [`crate::scan_dir_with_options`].
//!
//! 0.11 introduces a small, explicit ordering model. The rationale is
//! GUI predictability: a directory-tree widget needs the crate to return
//! the same entries in the same positions each time the user opens the
//! same folder, or the UI jitters. The crate offers exactly two orderings
//! — no more — to keep the surface narrow. Callers who need something
//! else can always re-sort the returned `Vec`s themselves.
//!
//! * [`SortOrder::Filesystem`] — preserve the OS's `readdir` order.
//!   Cheapest, but not stable across filesystems or remounts.
//! * [`SortOrder::NameAscDirsFirst`] — directories first, then files,
//!   each group sorted by name ascending. The go-to for GUI trees.
//!
//! The same enum is used by the high-level [`crate::Swdir::walk`] (via
//! [`crate::Swdir::sort_order`]) and the low-level
//! [`crate::scan_dir_with_options`] (via [`ScanOptions`]).

/// How entries are ordered in walk / scan results.
///
/// Only two variants, on purpose. If you need a third ordering, you
/// probably want to sort the `Vec` yourself at the call site — adding
/// more variants here would bloat the crate's surface without
/// proportionate benefit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SortOrder {
    /// Preserve the OS's `readdir` iteration order. Cheapest, but not
    /// necessarily reproducible across runs or filesystems.
    Filesystem,

    /// Directories first, then files, each group sorted by name
    /// ascending. The default — and usually what a GUI wants.
    #[default]
    NameAscDirsFirst,
}

/// Options for [`crate::scan_dir_with_options`].
///
/// Kept deliberately small. The struct exists mainly so that future
/// additions (should the need ever arise and the crate's scope expand)
/// do not break the low-level API. For 0.11 it holds a single field.
///
/// Note the contract: `ScanOptions::default()` and [`SortOrder::default`]
/// agree — a freshly-defaulted `ScanOptions` yields
/// [`SortOrder::NameAscDirsFirst`], which is the expected shape for GUI
/// callers. If you want raw OS order, call [`crate::scan_dir`] (no
/// options) or set `sort_order` explicitly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ScanOptions {
    /// How to order the returned entries.
    pub sort_order: SortOrder,
}

impl ScanOptions {
    /// Build a [`ScanOptions`] with the given sort order.
    ///
    /// Equivalent to `ScanOptions { sort_order, .. ScanOptions::default() }`
    /// but avoids having to name the struct's update syntax at the call site.
    /// The `#[non_exhaustive]` attribute on the struct means direct struct
    /// literals from outside the crate need `..Default::default()`; this
    /// constructor is the ergonomic shortcut.
    pub fn new(sort_order: SortOrder) -> Self {
        Self { sort_order }
    }
}
