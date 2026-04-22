//! Integration tests for the low-level [`swdir::scan_dir`] API.
//!
//! These tests spin up isolated temp directories under `$TMPDIR` with a PID
//! prefix so they can run in parallel without colliding, and clean up via a
//! `Drop` guard so a test panic still tidies up.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use swdir::{DirEntry, ScanError, scan_dir};

// ---------------------------------------------------------------------------
// Test fixture helpers
// ---------------------------------------------------------------------------

/// Self-cleaning temp directory. Not thread-global state — each test gets its
/// own unique path under $TMPDIR.
struct TmpDir(PathBuf);

impl TmpDir {
    fn new(tag: &str) -> Self {
        // Pid + nanos + tag gives us uniqueness even under parallel test
        // runs on the same machine.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "swdir-test-{}-{}-{}",
            std::process::id(),
            nanos,
            tag
        ));
        // In case of a leftover from a previous crashed run:
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create tmpdir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn touch(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        fs::write(&p, b"").expect("touch");
        p
    }

    fn mkdir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        fs::create_dir(&p).expect("mkdir");
        p
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        // Best-effort; avoid panicking in Drop.
        #[cfg(unix)]
        {
            // Re-open permissions in case a test set the dir to 000.
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Extract file names from scan results as a set, since order is not
/// guaranteed by the API contract.
fn name_set(entries: &[DirEntry]) -> BTreeSet<OsString> {
    entries.iter().map(|e| e.file_name()).collect()
}

// ---------------------------------------------------------------------------
// Happy-path tests
// ---------------------------------------------------------------------------

#[test]
fn scan_dir_empty_ok() {
    let tmp = TmpDir::new("empty");
    let entries = scan_dir(tmp.path()).expect("scan empty");
    assert!(entries.is_empty(), "expected empty vec, got {:?}", entries);
}

#[test]
fn scan_dir_files_only_ok() {
    let tmp = TmpDir::new("files-only");
    tmp.touch("a.txt");
    tmp.touch("b.md");
    tmp.touch("c");

    let entries = scan_dir(tmp.path()).expect("scan files-only");

    assert_eq!(entries.len(), 3);
    assert_eq!(
        name_set(&entries),
        ["a.txt", "b.md", "c"].iter().map(OsString::from).collect()
    );
    for e in &entries {
        assert!(e.is_file(), "{:?} should be file", e.path());
        assert!(!e.is_dir());
        assert!(!e.is_symlink());
    }
}

#[test]
fn scan_dir_dirs_only_ok() {
    let tmp = TmpDir::new("dirs-only");
    tmp.mkdir("alpha");
    tmp.mkdir("beta");

    let entries = scan_dir(tmp.path()).expect("scan dirs-only");

    assert_eq!(entries.len(), 2);
    assert_eq!(
        name_set(&entries),
        ["alpha", "beta"].iter().map(OsString::from).collect()
    );
    for e in &entries {
        assert!(e.is_dir(), "{:?} should be dir", e.path());
        assert!(!e.is_file());
    }
}

#[test]
fn scan_dir_mixed_ok() {
    let tmp = TmpDir::new("mixed");
    tmp.touch("file.txt");
    tmp.mkdir("subdir");
    tmp.touch("another");

    let entries = scan_dir(tmp.path()).expect("scan mixed");

    assert_eq!(entries.len(), 3);
    let dirs: Vec<_> = entries.iter().filter(|e| e.is_dir()).collect();
    let files: Vec<_> = entries.iter().filter(|e| e.is_file()).collect();
    assert_eq!(dirs.len(), 1);
    assert_eq!(files.len(), 2);
    assert_eq!(dirs[0].file_name(), OsString::from("subdir"));
}

#[test]
fn scan_dir_non_recursive() {
    // Must not descend into subdirectories.
    let tmp = TmpDir::new("non-recursive");
    tmp.touch("top.txt");
    let sub = tmp.mkdir("child");
    fs::write(sub.join("nested.txt"), b"").unwrap();

    let entries = scan_dir(tmp.path()).expect("scan top");

    assert_eq!(entries.len(), 2);
    assert_eq!(
        name_set(&entries),
        ["top.txt", "child"].iter().map(OsString::from).collect()
    );
    // Confirm nested.txt is reachable via a second scan — proves it exists
    // but was not included by the first one.
    let nested = scan_dir(&sub).expect("scan child");
    assert_eq!(nested.len(), 1);
    assert_eq!(nested[0].file_name(), OsString::from("nested.txt"));
}

#[test]
fn scan_dir_includes_hidden() {
    // scan_dir is a raw listing — no hidden filter (that's a higher-level
    // concern, handled by Swdir::walk).
    let tmp = TmpDir::new("hidden");
    tmp.touch(".hidden");
    tmp.touch("visible");

    let entries = scan_dir(tmp.path()).expect("scan hidden");
    assert_eq!(entries.len(), 2);
    assert_eq!(
        name_set(&entries),
        [".hidden", "visible"].iter().map(OsString::from).collect()
    );
}

#[test]
fn scan_dir_entry_metadata_present() {
    let tmp = TmpDir::new("metadata");
    tmp.touch("a.txt");

    let entries = scan_dir(tmp.path()).expect("scan");
    let e = &entries[0];
    let meta = e.metadata().expect("metadata should be available");
    assert!(meta.is_file());
}

#[test]
fn scan_dir_returns_absolute_ish_paths_rooted_at_input() {
    // Paths in DirEntry should have the scanned directory as a prefix,
    // matching std::fs::DirEntry::path() semantics.
    let tmp = TmpDir::new("paths");
    tmp.touch("f");
    let entries = scan_dir(tmp.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(
        entries[0].path().starts_with(tmp.path()),
        "{:?} should start with {:?}",
        entries[0].path(),
        tmp.path()
    );
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

#[test]
fn scan_dir_nonexistent_err() {
    let missing = std::env::temp_dir().join(format!(
        "swdir-test-{}-does-not-exist-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let err = scan_dir(&missing).expect_err("should fail on missing path");
    assert_eq!(err.io_kind(), io::ErrorKind::NotFound);
    assert_eq!(err.path(), missing);
}

#[test]
fn scan_dir_on_file_err() {
    let tmp = TmpDir::new("on-file");
    let f = tmp.touch("plain");
    let err = scan_dir(&f).expect_err("scanning a file should fail");
    // On Unix this is typically NotADirectory; we don't hard-code the kind
    // (Windows uses Other) but we do insist the error carries the path.
    assert_eq!(err.path(), f);
}

#[cfg(unix)]
#[test]
fn scan_dir_permission_denied_err() {
    use std::os::unix::fs::PermissionsExt;

    // Running as root renders chmod 000 ineffective — skip cleanly.
    if nix_running_as_root() {
        eprintln!("skipping permission test: running as root");
        return;
    }

    let tmp = TmpDir::new("perm-denied");
    // chmod 000 on the directory itself — read_dir should fail.
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o000))
        .expect("chmod 000");

    let result = scan_dir(tmp.path());

    // Restore permissions immediately so TmpDir::drop can clean up
    // regardless of assertion outcome.
    let _ = fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o755));

    let err = result.expect_err("chmod 000 should give permission denied");
    assert_eq!(err.io_kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(err.path(), tmp.path());
}

#[cfg(unix)]
fn nix_running_as_root() -> bool {
    // Avoid pulling nix/libc just for geteuid: parse /proc/self/status.
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1).and_then(|x| x.parse::<u32>().ok()))
        })
        .map(|uid| uid == 0)
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Contract checks: thread-safety & panic-freedom
// ---------------------------------------------------------------------------

/// Compile-time assertion: `DirEntry` and `ScanError` are `Send + 'static`,
/// i.e. usable with `std::thread::spawn` and iced's `Task::perform`.
#[test]
fn compile_time_send_static() {
    fn assert_send_static<T: Send + 'static>() {}
    assert_send_static::<DirEntry>();
    assert_send_static::<Vec<DirEntry>>();
    assert_send_static::<ScanError>();
    assert_send_static::<Result<Vec<DirEntry>, ScanError>>();
}

#[test]
fn scan_dir_usable_across_threads() {
    // Mirrors the iced-style off-thread pattern from the docs: we spawn a
    // worker thread and .join() it. If scan_dir panicked, .join() would
    // return Err; if the result weren't Send, this wouldn't compile.
    let tmp = TmpDir::new("threaded");
    tmp.touch("x");
    tmp.mkdir("y");
    let path: PathBuf = tmp.path().to_path_buf();

    let handle = std::thread::spawn(move || scan_dir(&path));
    let result = handle.join().expect("scan_dir must not panic");
    let entries = result.expect("scan ok");
    assert_eq!(entries.len(), 2);
}

#[test]
fn scan_dir_does_not_panic_on_bad_input() {
    // Deliberately feed a clearly-invalid path. Must return Err, not panic.
    let _ = scan_dir(Path::new("")); // empty string path
    let _ = scan_dir(Path::new("/definitely/does/not/exist/xyz-123"));
    // Reached here without panic.
}

// ---------------------------------------------------------------------------
// walk() regression — make sure the existing API still works alongside.
// ---------------------------------------------------------------------------

#[test]
fn walk_still_works_alongside_scan_dir() {
    use swdir::Swdir;
    // Old API against the checked-in fixture directory.
    let result = Swdir::default().set_root_path("tests/fixtures").walk();
    assert_eq!(result.path, Path::new("tests/fixtures").to_path_buf());

    // New API on the same directory.
    let entries = scan_dir(Path::new("tests/fixtures")).unwrap();
    assert!(!entries.is_empty());
}
