//! Error-path tests for [`swdir::scan_dir`].
//!
//! `scan_dir` is **atomic** — on any I/O failure it returns
//! `Err(ScanError::Io { path, source })` and does not produce partial
//! results. These tests pin that contract.

use std::fs;
use std::io;

use swdir::scan_dir;

use super::common::TmpDir;

#[test]
fn nonexistent_path_returns_not_found() {
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
fn scanning_a_file_is_an_error() {
    let tmp = TmpDir::new("on-file");
    let f = tmp.touch("plain");
    let err = scan_dir(&f).expect_err("scanning a file should fail");
    // On Unix this is typically `NotADirectory`; Windows uses `Other`.
    // We don't hard-code the kind — only insist the error carries the
    // offending path.
    assert_eq!(err.path(), f);
}

#[cfg(unix)]
#[test]
fn permission_denied_reading_directory() {
    use std::os::unix::fs::PermissionsExt;

    // Running as root renders chmod 000 ineffective — skip cleanly
    // rather than falsely failing in container / CI environments.
    if running_as_root() {
        eprintln!("skipping permission test: running as root");
        return;
    }

    let tmp = TmpDir::new("perm-denied");
    // chmod 000 on the directory itself — read_dir should fail.
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o000)).expect("chmod 000");

    let result = scan_dir(tmp.path());

    // Restore permissions immediately so TmpDir::drop can clean up
    // regardless of assertion outcome.
    let _ = fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o755));

    let err = result.expect_err("chmod 000 should give permission denied");
    assert_eq!(err.io_kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(err.path(), tmp.path());
}

#[cfg(unix)]
fn running_as_root() -> bool {
    // Avoid pulling `nix` or `libc` just for `geteuid`: parse
    // `/proc/self/status`. If that's unavailable we conservatively
    // assume non-root so the permission test runs.
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines().find(|l| l.starts_with("Uid:")).and_then(|l| {
                l.split_whitespace()
                    .nth(1)
                    .and_then(|x| x.parse::<u32>().ok())
            })
        })
        .map(|uid| uid == 0)
        .unwrap_or(false)
}
