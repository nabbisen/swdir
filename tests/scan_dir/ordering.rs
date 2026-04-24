//! Ordering tests for [`swdir::scan_dir_with_options`] and the
//! [`swdir::SortOrder`] enum.
//!
//! The contract under test, per the 0.11 spec:
//!
//! * `SortOrder::Filesystem` — same result *set* as bare `scan_dir`,
//!   order unspecified.
//! * `SortOrder::NameAscDirsFirst` — directories alphabetically,
//!   then files alphabetically; no interleaving.
//! * Default `ScanOptions::default()` yields `NameAscDirsFirst` (the
//!   GUI-friendly shape).
//! * Repeat calls under `NameAscDirsFirst` must be byte-identical —
//!   the cornerstone of flicker-free tree rendering.

use std::ffi::OsString;

use swdir::{ScanOptions, SortOrder, scan_dir, scan_dir_with_options};

use super::common::{TmpDir, name_set};

#[test]
fn filesystem_order_has_same_set_as_bare_scan_dir() {
    // Order is unspecified for `Filesystem`, so we can only assert the
    // set matches — which is enough to rule out "sort dropped entries"
    // or "sort added duplicates" bugs.
    let tmp = TmpDir::new("scan-opts-fs");
    tmp.touch("alpha");
    tmp.touch("bravo");
    tmp.mkdir("charlie");

    let plain = scan_dir(tmp.path()).expect("scan_dir");
    let with_opts = scan_dir_with_options(tmp.path(), &ScanOptions::new(SortOrder::Filesystem))
        .expect("scan_dir_with_options");

    assert_eq!(name_set(&plain), name_set(&with_opts));
}

#[test]
fn name_asc_dirs_first_groups_dirs_before_files() {
    // Names are chosen so plain alphabetical order (f_* < z_*) would
    // interleave dirs and files — meaning this test fails unless the
    // sort actually groups by kind before sorting by name.
    let tmp = TmpDir::new("scan-opts-nadf");
    tmp.touch("f_apple");
    tmp.touch("f_banana");
    tmp.mkdir("z_dir");
    tmp.mkdir("a_dir");

    let entries = scan_dir_with_options(tmp.path(), &ScanOptions::new(SortOrder::NameAscDirsFirst))
        .expect("scan_dir_with_options");

    // Expect: [a_dir, z_dir, f_apple, f_banana]  — dirs first (A..Z),
    // then files (A..Z). Neither group is interleaved.
    let names: Vec<OsString> = entries.iter().map(|e| e.file_name()).collect();
    assert_eq!(
        names,
        vec![
            OsString::from("a_dir"),
            OsString::from("z_dir"),
            OsString::from("f_apple"),
            OsString::from("f_banana"),
        ]
    );
}

#[test]
fn scan_options_default_is_name_asc_dirs_first() {
    // The default value of ScanOptions must yield a deterministic,
    // GUI-friendly order. A separate test so a future accidental
    // default swap (say, to `Filesystem`) fails loudly with an obvious
    // message instead of hiding behind a reproducibility test.
    assert_eq!(
        ScanOptions::default().sort_order,
        SortOrder::NameAscDirsFirst
    );
}

#[test]
fn name_asc_dirs_first_is_reproducible() {
    // Two consecutive calls under NameAscDirsFirst must return the
    // same name sequence. Cornerstone guarantee for GUI rendering
    // stability: without this, re-expanding a folder could reshuffle
    // children on screen.
    let tmp = TmpDir::new("scan-opts-repro");
    for n in ["one", "two", "three", "four"] {
        tmp.touch(n);
    }
    for n in ["DirA", "DirB"] {
        tmp.mkdir(n);
    }

    let names = |_| {
        scan_dir_with_options(tmp.path(), &ScanOptions::default())
            .unwrap()
            .iter()
            .map(|e| e.file_name())
            .collect::<Vec<OsString>>()
    };
    assert_eq!(names(()), names(()));
}
