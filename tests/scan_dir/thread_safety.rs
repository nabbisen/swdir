//! Thread-safety and panic-freedom contracts for [`swdir::scan_dir`].
//!
//! The iced / Tokio pattern shown in the crate docs — spawn a worker
//! thread, `.join()` it — requires that the returned types be
//! `Send + 'static` and that `scan_dir` itself never panics on bad
//! input (it must return `Err` instead). These tests pin both.

use std::path::{Path, PathBuf};

use swdir::{DirEntry, ScanError, scan_dir};

use super::common::TmpDir;

/// Compile-time assertion: `DirEntry` and `ScanError` are
/// `Send + 'static`, i.e. usable with `std::thread::spawn`, iced's
/// `Task::perform`, or any other async runtime's blocking-task helper.
#[test]
fn dir_entry_and_scan_error_are_send_static() {
    fn assert_send_static<T: Send + 'static>() {}
    assert_send_static::<DirEntry>();
    assert_send_static::<Vec<DirEntry>>();
    assert_send_static::<ScanError>();
    assert_send_static::<Result<Vec<DirEntry>, ScanError>>();
}

#[test]
fn usable_across_threads() {
    // Mirrors the iced-style off-thread pattern from the docs: spawn a
    // worker thread, `.join()` it. If scan_dir panicked, `.join()` would
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
fn does_not_panic_on_bad_input() {
    // Deliberately feed clearly-invalid paths. Must return Err, not
    // panic. If this test *panics*, the whole integration-test binary
    // aborts — which is exactly the failure mode we want to catch.
    let _ = scan_dir(Path::new("")); // empty string path
    let _ = scan_dir(Path::new("/definitely/does/not/exist/xyz-123"));
    // Reached here without panic.
}
