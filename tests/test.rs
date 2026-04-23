//! Integration tests for the 0.10 [`swdir::Swdir`] API.
//!
//! These mirror the 0.9 tests as far as the behavior is still meaningful,
//! and add coverage for the pieces that are actually new: [`FilterRule`]
//! composition, the two-axis [`Decision`] (include vs descend), and the
//! [`WalkReport`] / [`WalkError`] error-collection contract.

#![cfg(feature = "filter")]

use std::path::PathBuf;

use swdir::{
    Decision, DirNodeCount, EntryKind, FilterRule, Recurse, Swdir, SwdirError, WalkReport,
};

// ---------------------------------------------------------------------------
// Defaults & the "SkipHidden is on by default" contract
// ---------------------------------------------------------------------------

#[test]
fn walk_current_directory_ok() {
    let report = Swdir::new().root_path(".").walk();
    assert_eq!(report.tree.path, std::path::Path::new(".").to_path_buf());
}

#[test]
fn default_skip_hidden_is_installed() {
    // `Swdir::new()` must ship a single SkipHidden rule so the common
    // case is one line.
    let s = Swdir::new();
    let rules = s.filter_rules();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0], FilterRule::SkipHidden);
}

#[test]
fn walk_not_recurse_ok() {
    let report = Swdir::new().root_path("tests/fixtures").walk();
    assert_eq!(
        report.tree.files.as_slice(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
    // No recursion → no sub_dirs.
    assert!(report.tree.sub_dirs.is_empty());
}

#[test]
fn walk_default_skips_hidden() {
    // Fixture has .hidden-file and .hidden-dir at root; default must drop them.
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Depth(1))
        .walk();
    assert_eq!(
        report.tree.files.as_slice(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
    // Only the visible subdir should be present.
    assert_eq!(report.tree.sub_dirs.len(), 1);
    assert_eq!(
        report.tree.sub_dirs[0].files.as_slice(),
        &[PathBuf::from("tests/fixtures/subdir/subdir.txt")]
    );
}

#[test]
fn clear_filters_exposes_hidden() {
    // 0.9 had `include_hidden()`; the 0.10 equivalent is "drop the default rule".
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Depth(1))
        .clear_filters()
        .walk();
    assert_eq!(
        report.tree.files.as_slice(),
        &[
            PathBuf::from("tests/fixtures/.hidden-file"),
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
    // Both .hidden-dir and subdir are present, sorted by path.
    assert_eq!(report.tree.sub_dirs.len(), 2);
    assert_eq!(
        report.tree.sub_dirs[0].files.as_slice(),
        &[PathBuf::from("tests/fixtures/.hidden-dir/dummy")]
    );
    assert_eq!(
        report.tree.sub_dirs[1].files.as_slice(),
        &[PathBuf::from("tests/fixtures/subdir/subdir.txt")]
    );
}

// ---------------------------------------------------------------------------
// Recurse enum replaces the 0.9 struct — cover every variant.
// ---------------------------------------------------------------------------

#[test]
fn recurse_none_leaves_sub_dirs_empty() {
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::None)
        .walk();
    assert!(report.tree.sub_dirs.is_empty());
    assert_eq!(report.tree.files.len(), 3);
}

#[test]
fn recurse_depth_0_is_root_only() {
    // Depth(0) matches 0.9's depth_limit = Some(0): no descent at all.
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Depth(0))
        .walk();
    assert_eq!(
        report.tree.files.as_slice(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
    assert!(report.tree.sub_dirs.is_empty());
}

#[test]
fn recurse_depth_1_reaches_first_level() {
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Depth(1))
        .walk();
    assert_eq!(
        report.tree.sub_dirs[0].files.as_slice(),
        &[PathBuf::from("tests/fixtures/subdir/subdir.txt")]
    );
}

#[test]
fn recurse_unlimited_descends_whole_tree() {
    // fixtures is shallow so Depth(1) and Unlimited agree here; we just
    // confirm Unlimited doesn't bail early.
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Unlimited)
        .walk();
    assert_eq!(report.tree.sub_dirs.len(), 1);
    assert_eq!(
        report.tree.sub_dirs[0].files.as_slice(),
        &[PathBuf::from("tests/fixtures/subdir/subdir.txt")]
    );
}

// ---------------------------------------------------------------------------
// FilterRule composition — replaces 0.9 set_extension_* setters.
// ---------------------------------------------------------------------------

#[test]
fn extension_allowlist_rule_ok() {
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .filter(FilterRule::extension_allowlist(["md"]).unwrap())
        .walk();
    assert_eq!(
        report.tree.files.as_slice(),
        &[PathBuf::from("tests/fixtures/test.md")]
    );
}

#[test]
fn extension_denylist_rule_ok() {
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .filter(FilterRule::extension_denylist(["md"]).unwrap())
        .walk();
    // `test` has no extension → passes denylist.
    assert_eq!(
        report.tree.files.as_slice(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
}

#[test]
fn extension_allowlist_and_denylist_stack_freely() {
    // 0.9's `DuplicateExtensionList` error is gone — rules AND, so giving
    // both an allow and a deny for different extensions is sensible and
    // must not error.
    let s = Swdir::new()
        .root_path("tests/fixtures")
        .filter(FilterRule::extension_allowlist(["md", "txt"]).unwrap())
        .filter(FilterRule::extension_denylist(["md"]).unwrap());
    let report = s.walk();
    assert_eq!(
        report.tree.files.as_slice(),
        &[PathBuf::from("tests/fixtures/test.txt")]
    );
}

#[test]
fn allowlist_rejects_leading_period() {
    let err = FilterRule::extension_allowlist([".md"]).unwrap_err();
    assert_eq!(err, SwdirError::InvalidExtensionListItem(".md".to_owned()));
}

#[test]
fn denylist_rejects_leading_period() {
    let err = FilterRule::extension_denylist([".md"]).unwrap_err();
    assert_eq!(err, SwdirError::InvalidExtensionListItem(".md".to_owned()));
}

// ---------------------------------------------------------------------------
// Decision: the include / descend split.
// ---------------------------------------------------------------------------

#[test]
fn decision_constants_are_sensible() {
    assert_eq!(Decision::PASS, Decision::new(true, true));
    assert_eq!(Decision::DROP, Decision::new(false, false));
    assert_eq!(Decision::HIDDEN_PASSTHROUGH, Decision::new(false, true));
}

#[test]
fn only_kinds_file_still_reaches_nested_files() {
    // Proof-of-contract for the "include and descend are separate" rule:
    // OnlyKinds(File) hides directories from output but still descends
    // into them so nested files remain reachable.
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Unlimited)
        .filter(FilterRule::only_kind(EntryKind::File))
        .walk();

    let flat = report.tree.flatten_paths();
    assert!(
        flat.contains(&PathBuf::from("tests/fixtures/subdir/subdir.txt")),
        "nested file should still be reached: got {:?}",
        flat
    );
}

// ---------------------------------------------------------------------------
// MaxDepth filter: the depth-aware variant of Recurse::Depth.
// ---------------------------------------------------------------------------

#[test]
fn max_depth_filter_zero_drops_nested() {
    // MaxDepth(0) should match Recurse::None behavior: only root-level entries.
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Unlimited)
        .filter(FilterRule::max_depth(0))
        .walk();

    let flat = report.tree.flatten_paths();
    // No entries under subdir/ should appear.
    for p in &flat {
        assert!(
            !p.to_string_lossy().contains("subdir/"),
            "unexpected nested path: {:?}",
            p
        );
    }
}

// ---------------------------------------------------------------------------
// WalkReport: errors are collected, not printed.
// ---------------------------------------------------------------------------

#[test]
fn walk_missing_root_records_error_not_panic() {
    let report = Swdir::new()
        .root_path("/definitely/does/not/exist/swdir-xyz-123")
        .walk();
    assert!(!report.is_ok(), "missing root must populate errors");
    assert_eq!(report.errors.len(), 1);
    assert_eq!(
        report.errors[0].io_kind(),
        std::io::ErrorKind::NotFound,
        "kind = {:?}",
        report.errors[0].io_kind()
    );
}

#[test]
fn walk_ok_fixture_has_no_errors() {
    let report: WalkReport = Swdir::new().root_path("tests/fixtures").walk();
    assert!(report.is_ok(), "unexpected errors: {:?}", report.errors);
}

#[test]
fn walk_tree_discards_errors() {
    // `walk_tree()` is the "I don't care about errors" shortcut.
    let tree = Swdir::new().root_path("tests/fixtures").walk_tree();
    assert_eq!(tree.path, PathBuf::from("tests/fixtures"));
}

// ---------------------------------------------------------------------------
// Flatten / count — unchanged behavior, just routed via WalkReport.
// ---------------------------------------------------------------------------

#[test]
fn flatten_paths_not_recurse_ok() {
    let report = Swdir::new().root_path("tests/fixtures").walk();
    let flat = report.tree.flatten_paths();
    assert_eq!(
        flat,
        vec![
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
}

#[test]
fn flatten_paths_recurse_ok() {
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Depth(1))
        .walk();
    let flat = report.tree.flatten_paths();
    assert_eq!(
        flat,
        vec![
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
            PathBuf::from("tests/fixtures/subdir/subdir.txt"),
        ]
    );
}

#[test]
fn count_root_only_ok() {
    let report = Swdir::new().root_path("tests/fixtures").walk();
    let count = report.tree.count();
    assert_eq!(count, DirNodeCount { files: 3, dirs: 1 });
}

#[test]
fn count_sub_dir_included_ok() {
    let report = Swdir::new()
        .root_path("tests/fixtures")
        .recurse(Recurse::Depth(1))
        .walk();
    let count = report.tree.count();
    assert_eq!(count, DirNodeCount { files: 4, dirs: 2 });
}
