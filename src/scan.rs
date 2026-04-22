//! Low-level single-directory scan API.
//!
//! See [`scan_dir`].

use std::fs;
use std::path::Path;

use crate::helpers::dir_entry::DirEntry;
use crate::helpers::scan_error::ScanError;

/// Scan a single directory non-recursively and return its direct entries.
///
/// This is the low-level counterpart to [`crate::Swdir::walk`]: it scans
/// exactly one level of `path` and hands back a `Vec<DirEntry>`. It exists
/// for GUI / lazy-loading use cases (e.g. iced tree views that expand one
/// node at a time).
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
