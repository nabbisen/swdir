use std::path::PathBuf;

use swdir::{Recurse, Swdir};

#[test]
fn it_works() {
    let result = Swdir::default().set_root_path(".").scan();
    assert_eq!(result.path, std::path::Path::new(".").to_path_buf());
}

#[test]
fn scan_not_recurse() {
    let result = Swdir::default().set_root_path("tests/fixtures").scan();
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
fn scan_recurse_depth_limit_0() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            is_recurse: true,
            depth_limit: Some(0),
        })
        .scan();
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
fn scan_recurse_depth_limit_1() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_recurse(Recurse {
            is_recurse: true,
            depth_limit: Some(1),
        })
        .scan();
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
fn scan_with_allowlist() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_extension_allowlist(&["md"])
        .unwrap()
        .scan();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[PathBuf::from("tests/fixtures/test.md"),]
    );
}

#[test]
fn scan_with_denylist() {
    let result = Swdir::default()
        .set_root_path("tests/fixtures")
        .set_extension_denylist(&["md"])
        .unwrap()
        .scan();
    assert_eq!(
        result.files.as_array().unwrap(),
        &[
            PathBuf::from("tests/fixtures/test"),
            PathBuf::from("tests/fixtures/test.txt"),
        ]
    );
}

#[test]
fn err_duplicate_extension_allowlist() {
    let result = Swdir::default()
        .set_extension_denylist(&["txt"])
        .unwrap()
        .set_extension_allowlist(&["txt"]);
    assert!(result.is_err());
}

#[test]
fn err_duplicate_extension_denylist() {
    let result = Swdir::default()
        .set_extension_allowlist(&["txt"])
        .unwrap()
        .set_extension_denylist(&["txt"]);
    assert!(result.is_err());
}

#[test]
fn err_allowlist_start_with_period() {
    let result = Swdir::default().set_extension_allowlist(&[".txt"]);
    assert!(result.is_err());
}

#[test]
fn err_denylist_start_with_period() {
    let result = Swdir::default().set_extension_denylist(&[".txt"]);
    assert!(result.is_err());
}
