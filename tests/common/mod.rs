//! Shared fixtures for the integration-test suite.
//!
//! Cargo compiles each `tests/*.rs` file as its own binary, but
//! `tests/common/mod.rs` is *not* directly under `tests/`, so cargo
//! leaves it alone — each test binary loads it explicitly with
//! `#[path = "common/mod.rs"] mod common;`. As a consequence every
//! binary re-compiles this file, and a helper that a given binary
//! doesn't use will light up `dead_code`. The blanket allow below is
//! the conventional fix for that layout.
//!
//! If you need something specific to just one test binary, keep it in
//! that binary's own module tree (e.g. `tests/scan_dir/`) rather than
//! adding it here.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use swdir::DirEntry;

/// Self-cleaning temp directory. Not thread-global state — each test gets
/// its own unique path under `$TMPDIR`, so tests run in parallel without
/// colliding.
pub(crate) struct TmpDir(PathBuf);

impl TmpDir {
    /// Create a fresh, empty temp directory tagged for this test. `tag`
    /// only affects the directory name — any test can pick any tag; the
    /// pid + nanos suffix keeps the path unique regardless.
    pub(crate) fn new(tag: &str) -> Self {
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
        // Defensive cleanup in case of a leftover from a previous
        // crashed run — remove_dir_all on a missing path is a no-op.
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create tmpdir");
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    /// Create an empty regular file inside the temp directory and return
    /// its full path. Panics on I/O failure — tests are allowed to
    /// assume their own temp directory is writable.
    pub(crate) fn touch(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        fs::write(&p, b"").expect("touch");
        p
    }

    /// Create a subdirectory inside the temp directory and return its
    /// full path.
    pub(crate) fn mkdir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        fs::create_dir(&p).expect("mkdir");
        p
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        // Best-effort cleanup; never panic in Drop.
        #[cfg(unix)]
        {
            // Re-open permissions in case a test set the dir to 000
            // (e.g. the permission-denied test).
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Extract file names from scan results as a `BTreeSet` — the standard
/// way to compare scan output whose order is not guaranteed by the API.
pub(crate) fn name_set(entries: &[DirEntry]) -> BTreeSet<OsString> {
    entries.iter().map(|e| e.file_name()).collect()
}
