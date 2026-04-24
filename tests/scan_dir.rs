//! Integration-test binary for the low-level, single-directory
//! [`swdir::scan_dir`] / [`swdir::scan_dir_with_options`] API.
//!
//! Cargo compiles this file — and only this file — as the test binary;
//! the `tests/scan_dir/` subdirectory isn't auto-detected. Everything
//! is re-exposed via `mod` declarations below so every submodule
//! appears as a logical grouping inside one binary.
//!
//! The grouping follows the product contract, not implementation files:
//!
//! * [`listing`] — baseline happy-path listings
//! * [`errors`] — the atomic-failure contract
//! * [`ordering`] — `SortOrder` and `scan_dir_with_options`
//! * [`dir_entry_helpers`] — GUI-oriented [`swdir::DirEntry`] helpers
//! * [`thread_safety`] — `Send + 'static` and panic-freedom
//!
//! Shared fixtures live in [`common`]. Each test binary needs its own
//! `#[path = ...]` module declaration for `common`, because
//! `tests/common/mod.rs` is itself not compiled as a binary — see the
//! comment at the top of that file.

#[path = "common/mod.rs"]
mod common;

#[path = "scan_dir/dir_entry_helpers.rs"]
mod dir_entry_helpers;
#[path = "scan_dir/errors.rs"]
mod errors;
#[path = "scan_dir/listing.rs"]
mod listing;
#[path = "scan_dir/ordering.rs"]
mod ordering;
#[path = "scan_dir/thread_safety.rs"]
mod thread_safety;
