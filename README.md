# swdir

[![crates.io](https://img.shields.io/crates/v/swdir?label=rust)](https://crates.io/crates/swdir)
[![License](https://img.shields.io/github/license/nabbisen/swdir)](https://github.com/nabbisen/swdir/blob/main/LICENSE)
[![Rust Documentation](https://docs.rs/swdir/badge.svg?version=latest)](https://docs.rs/swdir)
[![Dependency Status](https://deps.rs/crate/swdir/latest/status.svg)](https://deps.rs/crate/swdir)

Swiftly traverse and scan directories recursively.
Sway 🪭, swing 🎷 or swim 🪼 in directories.

## Quick start

```sh
cargo install swdir
```

```rust
use swdir::Swdir;

fn run() {
    let dir_node = Swdir::default().set_root_path("/some/path").walk();
    // -> DirNode (files and subdirectories)
    //     -> flatten_paths() returns Vec<PathBuf>
}
```

### Recurse option

```rust
use swdir::{Recurse, Swdir};

fn run() {
    let recurse = Recurse {
        enabled: true,
        depth_limit: Some(1), // only first level subdirectory is scanned
    };

    let dir_node = Swdir::default()
        .set_root_path("/some/path")
        .set_recurse(recurse)
        .include_hidden() // don't skip hidden files and directories
        .walk();
}
```

### Allowlist and denylist

```rust
use swdir::{Swdir, SwdirError};

fn run() -> Result<(), SwdirError> {
    let dir_node_with_allowlist = Swdir::default()
        .set_root_path("/some/path")
        .set_extension_allowlist(&["md"])?
        .walk();

    let dir_node_with_denylist = Swdir::default()
        .set_root_path("/some/path")
        .set_extension_denylist(&["md"])?
        .walk();

    Ok(())
}
```

## GUI / lazy loading API — `scan_dir`

Alongside the high-level `Swdir::walk()` there is a low-level, single-directory
scan API aimed at GUI tree views (iced, egui, …) that expand one node at a
time. Use `scan_dir` when you want to stream a tree into the UI as the user
clicks, rather than pay the cost of a full traversal up front.

### How it differs from `walk()`

| | `Swdir::walk()` | `scan_dir(path)` |
|---|---|---|
| Scope | recursive, whole tree | one directory, direct children only |
| Filtering / sorting | hidden filter, extension allow/deny-list | none (raw listing) |
| Parallelism | internal rayon pool | single-threaded |
| Runtime deps | none (sync) | none (sync) |
| Return type | `DirNode` (tree) | `Result<Vec<DirEntry>, ScanError>` |
| Error handling | best-effort, warnings to stderr | atomic — first I/O failure is returned |
| Intended caller | CLI, batch tools | GUI lazy-loading, file pickers |

`walk()` remains unchanged and is still the right call for full or filtered
traversal. `scan_dir` is purely additive.

### Basic usage

```rust
use std::path::Path;
use swdir::{DirEntry, ScanError, scan_dir};

fn list(path: &Path) -> Result<(), ScanError> {
    let entries: Vec<DirEntry> = scan_dir(path)?;
    for e in entries {
        println!("{}{}", e.path().display(), if e.is_dir() { "/" } else { "" });
    }
    Ok(())
}
```

Each `DirEntry` is owned (`Send + 'static`) and caches its `FileType`, so
`is_dir()` / `is_file()` / `is_symlink()` answer without additional syscalls.
Empty directories return `Ok(Vec::new())`; permission denied, missing paths,
and "not a directory" all return `Err(ScanError::Io { path, source })` with
both the offending path and the original `std::io::Error` preserved.

### iced lazy tree example

`scan_dir` is deliberately synchronous. To keep the iced runtime responsive,
wrap the call in `std::thread::spawn` and drive it via `Task::perform`:

```rust,ignore
use std::path::PathBuf;
use iced::Task;
use swdir::{DirEntry, ScanError, scan_dir};

# enum Message { Loaded(Result<Vec<DirEntry>, ScanError>) }
fn load(path: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            std::thread::spawn(move || scan_dir(&path))
                .join()
                .expect("scan_dir must not panic")
        },
        Message::Loaded,
    )
}
```

No `tokio`, no `async-std`, no feature flag — the crate stays runtime-agnostic.
