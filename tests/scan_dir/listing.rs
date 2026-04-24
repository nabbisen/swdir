//! Happy-path listing tests for [`swdir::scan_dir`].
//!
//! These lock down the baseline contract: given some mix of files and
//! directories in one folder, what do we get back? No ordering
//! assertions live here — those belong in [`super::ordering`].

use std::ffi::OsString;
use std::fs;

use swdir::scan_dir;

use super::common::{TmpDir, name_set};

#[test]
fn empty_ok() {
    let tmp = TmpDir::new("empty");
    let entries = scan_dir(tmp.path()).expect("scan empty");
    assert!(entries.is_empty(), "expected empty vec, got {:?}", entries);
}

#[test]
fn files_only_ok() {
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
fn dirs_only_ok() {
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
fn mixed_files_and_dirs_ok() {
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
fn non_recursive() {
    // `scan_dir` must not descend into subdirectories — that's the whole
    // point of pairing it with the recursive `Swdir::walk`.
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
fn only_sees_one_directory() {
    // Near-duplicate of `non_recursive`, but phrased as the 0.11 spec
    // phrased it: the "1 ディレクトリのみ" guarantee. Kept separate
    // because the two test names document two different product
    // commitments even if the mechanics coincide.
    let tmp = TmpDir::new("one-level");
    tmp.touch("top.txt");
    let child = tmp.mkdir("child");
    fs::write(child.join("nested.txt"), b"").expect("nested");

    let entries = scan_dir(tmp.path()).expect("scan one-level");
    let names = name_set(&entries);

    assert!(names.contains(&OsString::from("top.txt")));
    assert!(names.contains(&OsString::from("child")));
    assert!(
        !names.contains(&OsString::from("nested.txt")),
        "scan_dir leaked nested content: {:?}",
        names
    );
}

#[test]
fn includes_hidden() {
    // `scan_dir` is a raw listing — no hidden filter. Hiding dotfiles is
    // a higher-level concern handled by `Swdir::walk` via FilterRule.
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
fn entry_metadata_present() {
    let tmp = TmpDir::new("metadata");
    tmp.touch("a.txt");

    let entries = scan_dir(tmp.path()).expect("scan");
    let e = &entries[0];
    let meta = e.metadata().expect("metadata should be available");
    assert!(meta.is_file());
}

#[test]
fn returns_paths_rooted_at_input() {
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
