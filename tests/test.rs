use std::path::PathBuf;

use swdir::{DirNodeCount, Recurse, Swdir, SwdirError};

#[test]
fn walk_current_directory_ok() {
    let result = Swdir::default().set_root_path(".").walk();
    assert_eq!(result.path, std::path::Path::new(".").to_path_buf());
}

#[test]
fn walk_not_recurse_ok() {
    let result = Swdir::default().set_root_path("tests/fixtures").walk();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
}

#[test]
fn walk_not_include_hidden_ok() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            enabled: true,
            depth_limit: Some(1),
        })
        .walk();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
    assert_eq!(
        result.sub_dirs[0].files.as_array().unwrap(),
        &[PathBuf::from("tests/fixtures/subdir/subdir.txt"),]
    );
}

#[test]
fn walk_include_hidden_ok() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            enabled: true,
            depth_limit: Some(1),
        })
        .include_hidden()
        .walk();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[
            PathBuf::from("tests/fixtures/.hidden-file"),
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
    assert_eq!(
        result.sub_dirs[0].files.as_array().unwrap(),
        &[PathBuf::from("tests/fixtures/.hidden-dir/dummy"),]
    );
    assert_eq!(
        result.sub_dirs[1].files.as_array().unwrap(),
        &[PathBuf::from("tests/fixtures/subdir/subdir.txt"),]
    );
}

#[test]
fn walk_recurse_depth_limit_0_ok() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            enabled: true,
            depth_limit: Some(0),
        })
        .walk();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
}

#[test]
fn walk_recurse_depth_limit_1_ok() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            enabled: true,
            depth_limit: Some(1),
        })
        .walk();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
    assert_eq!(
        result.sub_dirs[0].files.as_array().unwrap(),
        &[PathBuf::from("tests/fixtures/subdir/subdir.txt"),]
    );
}

#[test]
fn walk_with_allowlist_ok() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_extension_allowlist(&["md"])
        .unwrap()
        .walk();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[PathBuf::from("tests/fixtures/test.md"),]
    );
}

#[test]
fn walk_with_denylist_ok() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_extension_denylist(&["md"])
        .unwrap()
        .walk();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
}

#[test]
fn duplicate_extension_allowlist_err() {
    let result = Swdir::default()
        .set_extension_denylist(&["txt"])
        .unwrap()
        .set_extension_allowlist(&["txt"]);
    assert_eq!(
        result.is_err_and(|x| x == SwdirError::DuplicateExtensionList),
        true
    );
}

#[test]
fn duplicate_extension_denylist_err() {
    let result = Swdir::default()
        .set_extension_allowlist(&["txt"])
        .unwrap()
        .set_extension_denylist(&["txt"]);
    assert_eq!(
        result.is_err_and(|x| x == SwdirError::DuplicateExtensionList),
        true
    );
}

#[test]
fn allowlist_start_with_period_err() {
    let result = Swdir::default().set_extension_allowlist(&[".txt"]);
    assert_eq!(
        result.is_err_and(|x| x == SwdirError::InvalidExtensionListItem(".txt".to_owned())),
        true
    );
}

#[test]
fn denylist_start_with_period_err() {
    let result = Swdir::default().set_extension_denylist(&[".txt"]);
    assert_eq!(
        result.is_err_and(|x| x == SwdirError::InvalidExtensionListItem(".txt".to_owned())),
        true
    );
}

#[test]
fn dir_node_flatten_paths_not_recurse_ok() {
    let dir_node = Swdir::default().set_root_path("tests/fixtures").walk();
    let flatten_paths = dir_node.flatten_paths();
    assert_eq!(
        flatten_paths,
        vec![
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.md"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
}

#[test]
fn dir_node_flatten_paths_recurse_ok() {
    let dir_node = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            enabled: true,
            depth_limit: Some(1),
        })
        .walk();
    let flatten_paths = dir_node.flatten_paths();
    assert_eq!(
        flatten_paths,
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
    let dir_node = Swdir::default().set_root_path("tests/fixtures").walk();
    let count = dir_node.count();
    assert_eq!(count, DirNodeCount { files: 3, dirs: 1 });
}

#[test]
fn count_sub_dir_included_ok() {
    let dir_node = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            enabled: true,
            depth_limit: Some(1),
        })
        .walk();
    let count = dir_node.count();
    assert_eq!(count, DirNodeCount { files: 4, dirs: 2 });
}
