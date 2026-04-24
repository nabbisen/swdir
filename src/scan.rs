//! Low-level single-directory scan API.
//!
//! This module is the crate's **lazy-loading entry point**. Use it when
//! a GUI's directory tree widget needs one folder's direct children at a
//! time, rather than a full recursive walk — see [`scan_dir`] and
//! [`scan_dir_with_options`].
//!
//! For recursive traversals, use [`crate::Swdir::walk`] instead.

use std::fs;
use std::path::Path;

use crate::helpers::dir_entry::DirEntry;
use crate::helpers::scan_error::ScanError;
use crate::helpers::sort::{ScanOptions, SortOrder};

/// Scan a single directory non-recursively and return its direct entries,
/// in the order the OS's `readdir` yields them.
///
/// This is the low-level counterpart to [`crate::Swdir::walk`]: it scans
/// exactly one level of `path` and hands back a `Vec<DirEntry>`. It
/// exists for GUI / lazy-loading use cases — for example, an iced tree
/// view that expands one node at a time, fetching only what the user
/// actually opens.
///
/// This function intentionally does *no* ordering. If the GUI wants a
/// predictable display order (dirs first, then files, alphabetically),
/// call [`scan_dir_with_options`] instead.
///
/// # Properties
///
/// * **Non-recursive.** Subdirectories are listed but not descended into.
/// * **No filtering, no sorting.** Callers decide how to order/filter;
///   order of the returned `Vec` is whatever the OS's `readdir` returns.
/// * **No parallelism.** A single directory listing is I/O-bound; the
///   rayon overhead is not justified here.
/// * **No async.** Callers wanting off-thread execution should wrap this
///   with `std::thread::spawn` (or a runtime's blocking pool). Keeping
///   this sync means the crate stays runtime-independent.
/// * **Atomic.** Returns `Err` on the first I/O failure — either from
///   `read_dir` itself or from reading an entry's file type. Partial
///   results are not produced.
///
/// # Errors
///
/// Returns [`ScanError::Io`] wrapping the original [`std::io::Error`]
/// together with the offending [`std::path::PathBuf`]. Notable cases:
///
/// * `path` does not exist → `NotFound`
/// * `path` is a file, not a directory → `NotADirectory` (on Unix)
/// * permission denied reading the directory → `PermissionDenied`
/// * permission denied on a child's metadata → `PermissionDenied`
///
/// Empty directories are *not* errors; they return `Ok(Vec::new())`, so
/// callers can cleanly distinguish "nothing here" from "couldn't look".
///
/// # Thread-safety
///
/// The returned `Vec<DirEntry>` is `'static + Send`, safe to move to
/// another thread.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use swdir::scan_dir;
///
/// let entries = scan_dir(Path::new("."))?;
/// for entry in &entries {
///     println!("{} (dir = {})", entry.path().display(), entry.is_dir());
/// }
/// # Ok::<(), swdir::ScanError>(())
/// ```
///
/// # Example — iced lazy loading
///
/// ```ignore
/// use std::path::PathBuf;
/// use iced::Task;
/// use swdir::{DirEntry, ScanError, scan_dir};
///
/// # enum Message { Loaded(Result<Vec<DirEntry>, ScanError>) }
/// fn load(path: PathBuf) -> Task<Message> {
///     Task::perform(
///         async move {
///             // Delegate the blocking call to a worker thread so the
///             // iced runtime keeps ticking.
///             std::thread::spawn(move || scan_dir(&path))
///                 .join()
///                 .expect("scan_dir panicked (should not happen)")
///         },
///         Message::Loaded,
///     )
/// }
/// ```
pub fn scan_dir(path: &Path) -> Result<Vec<DirEntry>, ScanError> {
    read_entries(path)
}

/// Scan a single directory non-recursively and return its direct entries,
/// ordered per [`ScanOptions::sort_order`].
///
/// Same contract as [`scan_dir`] for everything except ordering. GUI
/// callers should prefer this entry point: the default
/// [`ScanOptions::default()`] yields [`SortOrder::NameAscDirsFirst`],
/// the layout most tree widgets expect.
///
/// Sorting happens after the directory has been fully read, so it
/// introduces **no extra syscalls** on top of `scan_dir` — only a
/// single in-memory `Vec::sort_by` pass.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use swdir::{ScanOptions, SortOrder, scan_dir_with_options};
///
/// let opts = ScanOptions::new(SortOrder::NameAscDirsFirst);
/// let entries = scan_dir_with_options(Path::new("."), &opts)?;
/// # Ok::<(), swdir::ScanError>(())
/// ```
pub fn scan_dir_with_options(
    path: &Path,
    options: &ScanOptions,
) -> Result<Vec<DirEntry>, ScanError> {
    let mut entries = read_entries(path)?;
    match options.sort_order {
        SortOrder::Filesystem => {
            // Keep OS order as-is.
        }
        SortOrder::NameAscDirsFirst => {
            // Stable two-key sort: (is_file, file_name). Dirs (and
            // symlinks, which are neither dir nor regular-file in the
            // classification sense used here — we treat them as
            // "non-directories") come first, then files, each group
            // sorted by name ascending.
            //
            // Using `sort_by` with cached `FileType` so we don't touch
            // the filesystem again — the whole point of the cached
            // `file_type` field on `DirEntry`.
            entries.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                // `false < true`, so "is not a dir" sorts after "is a
                // dir" only if we invert. Easier to compare
                // `b_is_dir.cmp(&a_is_dir)` so dir=true wins.
                b_is_dir
                    .cmp(&a_is_dir)
                    .then_with(|| a.display_name().cmp(b.display_name()))
            });
        }
    }
    Ok(entries)
}

/// The shared read-entries body. Pulled out so the two public entry
/// points don't drift in their I/O handling.
fn read_entries(path: &Path) -> Result<Vec<DirEntry>, ScanError> {
    let read = fs::read_dir(path).map_err(|e| ScanError::io(path, e))?;

    let mut entries = Vec::new();
    for item in read {
        // Fail atomically on per-entry iterator errors (rare, but can happen
        // if the directory is mutated concurrently or a device goes away).
        let entry = item.map_err(|e| ScanError::io(path, e))?;
        let entry_path = entry.path();

        // file_type() can fail on some platforms (e.g. needs a stat). We do
        // NOT fall back silently here — the whole scan fails, per the atomic
        // contract above.
        let file_type = entry
            .file_type()
            .map_err(|e| ScanError::io(&entry_path, e))?;

        // metadata() is allowed to be absent per-entry (e.g. dangling
        // symlink) — callers see `DirEntry::metadata()` return `None`.
        let metadata = entry.metadata().ok();

        entries.push(DirEntry::new(entry_path, file_type, metadata));
    }
    Ok(entries)
}
