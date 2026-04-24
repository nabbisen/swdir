//! Owned directory entry produced by [`crate::scan_dir`].
//!
//! This type is intentionally distinct from [`std::fs::DirEntry`]:
//!
//! * It owns its data (`PathBuf`, `FileType`, `Option<Metadata>`) so instances
//!   are `'static` + `Send` and can cross thread boundaries — required for the
//!   iced / GUI lazy-loading use case.
//! * "Is this a directory?" is decidable in-memory without another syscall
//!   via [`DirEntry::is_dir`] — the whole point of returning `Vec<DirEntry>`
//!   instead of `Vec<PathBuf>` for a lazy tree.

use std::ffi::{OsStr, OsString};
use std::fs::{FileType, Metadata};
use std::path::{Path, PathBuf};

/// An owned directory entry returned by [`crate::scan_dir`].
///
/// Unlike [`std::fs::DirEntry`], this type is fully owned: it holds its own
/// [`PathBuf`], a cached [`FileType`], and an optional cached [`Metadata`].
/// That makes it cheap to move across threads and safe to keep around after
/// the underlying directory handle has been dropped.
///
/// `file_type` reflects the entry's own type *without* following symlinks
/// (matching the semantics of [`std::fs::DirEntry::file_type`]). Use
/// [`DirEntry::is_symlink`] to detect symlinks explicitly.
#[derive(Debug, Clone)]
pub struct DirEntry {
    path: PathBuf,
    file_type: FileType,
    metadata: Option<Metadata>,
}

impl DirEntry {
    /// Construct a `DirEntry` from already-extracted parts.
    ///
    /// Crate-internal; callers obtain `DirEntry` values through [`crate::scan_dir`].
    pub(crate) fn new(path: PathBuf, file_type: FileType, metadata: Option<Metadata>) -> Self {
        Self {
            path,
            file_type,
            metadata,
        }
    }

    /// Full path of the entry (including the scanned directory as prefix).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Consume `self` and return the owned path.
    pub fn into_path(self) -> PathBuf {
        self.path
    }

    /// Final path component as an [`OsString`] (e.g. `"foo.txt"`).
    ///
    /// Returns an empty `OsString` if the path has no file name, which in
    /// practice does not happen for entries produced by [`crate::scan_dir`].
    pub fn file_name(&self) -> OsString {
        self.path
            .file_name()
            .map(|s| s.to_os_string())
            .unwrap_or_default()
    }

    /// Cached [`FileType`] — never triggers an additional syscall.
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// `true` if the entry is a directory (does **not** follow symlinks).
    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    /// `true` if the entry is a regular file (does **not** follow symlinks).
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    /// `true` if the entry is itself a symlink.
    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }

    /// Cached [`Metadata`] when it was available at scan time.
    ///
    /// May be `None` if the metadata call failed for a single entry (rare —
    /// e.g. a dangling symlink, or a race with the filesystem). The scan as
    /// a whole still succeeds; only the per-entry metadata is missing.
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Final path component as a **borrowed** [`OsStr`] — intended for
    /// direct use as a GUI label (tree row, tab title, …).
    ///
    /// Unlike [`DirEntry::file_name`], this does *not* allocate; it hands
    /// back the slice stored inside the cached path. No syscall is made.
    ///
    /// For entries produced by [`crate::scan_dir`] the underlying path
    /// always has a file name, but to stay total the method falls back
    /// to an empty `&OsStr` instead of panicking if the path somehow
    /// lacks one (e.g. the path is the filesystem root).
    pub fn display_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or(OsStr::new(""))
    }

    /// This entry's path, relativized to `root`.
    ///
    /// Returns `Some(relative)` if `root` is a prefix of `self.path()`,
    /// else `None`. Pure path arithmetic — no filesystem I/O.
    ///
    /// Typical use: a GUI that caches `root` once and wants
    /// `"sub/thing.txt"` instead of `"/abs/root/sub/thing.txt"` for the
    /// tree label or a routing key.
    ///
    /// ```
    /// # use std::path::{Path, PathBuf};
    /// # // The doctest fabricates a DirEntry via the crate-private
    /// # // constructor, so it runs in-crate only; external callers
    /// # // get `DirEntry` values from `scan_dir`.
    /// # fn _shape() {
    /// # let entry: swdir::DirEntry = unimplemented!();
    /// let rel = entry.relative_to(Path::new("/workspace"));
    /// assert!(rel.as_deref().map(Path::is_relative).unwrap_or(true));
    /// # }
    /// ```
    pub fn relative_to(&self, root: &Path) -> Option<PathBuf> {
        self.path.strip_prefix(root).ok().map(Path::to_path_buf)
    }
}
