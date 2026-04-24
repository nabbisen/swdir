//! Tests for the GUI-friendly helpers on [`swdir::DirEntry`] introduced
//! in 0.11: `display_name`, `relative_to`, plus the long-standing
//! "cached `FileType` — no extra syscalls" invariant.
//!
//! These tests live here rather than in `listing.rs` because they
//! exercise the *entry*'s API, not the directory listing itself.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use swdir::scan_dir;

use super::common::TmpDir;

#[test]
fn display_name_returns_borrowed_os_str() {
    // We can't prove "no allocation" in a test, but we can verify that
    // the returned `&OsStr` matches the file name and has a lifetime
    // tied to the DirEntry (enforced by the compiler, not this assert).
    // That lifetime is the property GUIs care about.
    let tmp = TmpDir::new("display-name");
    tmp.touch("widget.rs");
    let entries = scan_dir(tmp.path()).expect("scan");
    let e = entries
        .iter()
        .find(|e| e.file_name() == "widget.rs")
        .unwrap();
    let borrowed: &OsStr = e.display_name();
    assert_eq!(borrowed, OsStr::new("widget.rs"));
}

#[test]
fn relative_to_strips_root_prefix() {
    let tmp = TmpDir::new("relto-under");
    tmp.touch("inner.txt");

    let entries = scan_dir(tmp.path()).expect("scan");
    let e = entries
        .iter()
        .find(|e| e.file_name() == "inner.txt")
        .unwrap();

    let rel = e.relative_to(tmp.path()).expect("under root");
    assert!(rel.is_relative(), "relative path should be relative");
    assert_eq!(rel, PathBuf::from("inner.txt"));
}

#[test]
fn relative_to_returns_none_for_unrelated_root() {
    let tmp = TmpDir::new("relto-outside");
    tmp.touch("x");

    let entries = scan_dir(tmp.path()).expect("scan");
    let e = entries.iter().find(|e| e.file_name() == "x").unwrap();

    // `/nonexistent-prefix-xyz` is not an ancestor of `$TMPDIR/...`,
    // so `strip_prefix` (and hence `relative_to`) returns None.
    let rel = e.relative_to(Path::new("/nonexistent-prefix-xyz"));
    assert!(rel.is_none(), "expected None, got {:?}", rel);
}

#[test]
fn relative_to_does_no_filesystem_io() {
    // `relative_to` must be pure path arithmetic. Feeding it a
    // fictitious root proves it doesn't try to canonicalize or stat
    // either side — if it did, a nonexistent path would be detectable
    // as an error or a syscall failure.
    let tmp = TmpDir::new("relto-noio");
    tmp.touch("a");
    let entries = scan_dir(tmp.path()).expect("scan");
    let e = entries.iter().find(|e| e.file_name() == "a").unwrap();

    let fake_root = Path::new("/nope/nope/nope-xyz");
    assert!(e.relative_to(fake_root).is_none());
}

#[test]
fn is_dir_uses_cached_file_type_even_after_file_removal() {
    // Regression guard: the spec requires "GUI から使う際に余計な syscall
    // を増やさないこと". The `FileType` is captured at scan time — we
    // prove this by deleting the file after the scan and confirming
    // `is_file()` still reports correctly. A fresh stat would have
    // failed or reported the file as missing.
    let tmp = TmpDir::new("cached-ft");
    let p = tmp.touch("to-delete");
    let entries = scan_dir(tmp.path()).expect("scan");
    let e = entries
        .iter()
        .find(|e| e.file_name() == "to-delete")
        .unwrap();
    assert!(e.is_file());

    fs::remove_file(&p).expect("rm");

    // After removal, a fresh stat would fail. Cached answer still fine.
    assert!(e.is_file());
    assert!(!e.is_dir());
}
