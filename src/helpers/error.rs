//! Construction-time errors for [`crate::Swdir`] / [`crate::FilterRule`].
//!
//! Kept small and focused on *building* the walker. Runtime I/O problems
//! encountered during [`crate::Swdir::walk`] are surfaced through
//! [`crate::WalkReport::errors`], not through this type.

use thiserror::Error;

/// Errors returned by builder-style constructors and validators.
///
/// `DuplicateExtensionList` from 0.9 is intentionally gone: in 0.10
/// allowlist / denylist are plain [`crate::FilterRule`] variants that
/// callers can stack freely — the old "can't have both at once" rule
/// no longer applies, so the error it produced is obsolete.
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum SwdirError {
    /// An extension string in an allow/deny list started with `.`
    /// (e.g. `".md"` instead of `"md"`). `std::path::Path::extension`
    /// never includes the leading dot, so dotted entries would
    /// silently never match — we reject them up front instead.
    #[error("Invalid extension list item (must not start with '.'): {0}")]
    InvalidExtensionListItem(String),
}
