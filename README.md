# swdir

[![crates.io](https://img.shields.io/crates/v/swdir?label=rust)](https://crates.io/crates/swdir)
[![License](https://img.shields.io/github/license/nabbisen/swdir)](https://github.com/nabbisen/swdir/blob/main/LICENSE)
[![Rust Documentation](https://docs.rs/swdir/badge.svg?version=latest)](https://docs.rs/swdir)
[![Dependency Status](https://deps.rs/crate/swdir/latest/status.svg)](https://deps.rs/crate/swdir)

Swiftly traverse and scan directories recursively.
Sway 🪭, swing 🎷 or swim 🪼 in directories.

## Quick start

```sh
cargo add swdir
```

```rust
use swdir::Swdir;

fn run() {
    let report = Swdir::new().root_path("/some/path").walk();
    //             -> WalkReport { tree: DirNode, errors: Vec<WalkError> }
    //                 tree.flatten_paths() returns Vec<PathBuf>
}
```

### Recursion

`Recurse` is an enum with three meaningful states:

```rust
use swdir::{Recurse, Swdir};

fn run() {
    let report = Swdir::new()
        .root_path("/some/path")
        .recurse(Recurse::Depth(1)) // only the root's immediate subdirs
        .walk();

    let report = Swdir::new()
        .root_path("/some/path")
        .recurse(Recurse::Unlimited) // whole tree
        .walk();
}
```

### Filtering

Filtering is part of the default `filter` feature. The model is built
around two types:

* [`FilterRule`] — the condition (hidden, extension, path prefix, kind, depth)
* [`Decision`] — what to do about a given entry, split into two axes:
  * `include` — should it appear in the results?
  * `descend` — should the walker look inside it?

Rules compose with AND. `Swdir::new()` already installs one rule —
`FilterRule::SkipHidden` — so the common case needs no extra
configuration.

```rust
use swdir::{FilterRule, Recurse, Swdir, SwdirError};

fn run() -> Result<(), SwdirError> {
    let report = Swdir::new()
        .root_path("/some/path")
        .recurse(Recurse::Unlimited)
        .filter(FilterRule::extension_allowlist(["md", "rs"])?)
        .filter(FilterRule::max_depth(3))
        .walk();
    Ok(())
}
```

To see hidden entries, clear the default rules first (this is the
0.10 equivalent of 0.9's `include_hidden()`):

```rust
use swdir::Swdir;

fn run() {
    let report = Swdir::new()
        .root_path("/some/path")
        .clear_filters()
        .walk();
}
```

#### `include` vs `descend`

Separating the two axes means a rule can hide a directory from the
result tree while still descending through it. For instance,
`FilterRule::only_kind(EntryKind::File)` drops directories from the
output but keeps walking into them so nested files remain reachable —
which is what callers usually want when they say "give me just the
files under here".

#### What ships in 0.10

| Rule                              | Purpose                                       |
|-----------------------------------|-----------------------------------------------|
| `FilterRule::SkipHidden`          | Drop entries whose name starts with `.` (plus Windows hidden-bit). |
| `FilterRule::OnlyKinds(..)`       | Keep only files / dirs / symlinks. |
| `FilterRule::ExtensionAllowlist(..)` | Keep files with these extensions. |
| `FilterRule::ExtensionDenylist(..)`  | Drop files with these extensions. |
| `FilterRule::UnderPath(..)`       | Restrict to entries under a path prefix. |
| `FilterRule::NotUnderPath(..)`    | Exclude everything under a path prefix. |
| `FilterRule::MaxDepth(n)`         | Cap depth of entries and descent. |

The enum is `#[non_exhaustive]`, so future additions won't break
existing `match` statements. Advanced filter families (regex, glob,
metadata predicates, arbitrary closures) are deliberately out of scope
for 0.10 — bring those needs upstream if they come up in practice.

### Error handling

`walk()` returns a [`WalkReport`]. Unreadable directories go into
`report.errors` instead of being printed to stderr, so callers can
handle partial failures on purpose:

```rust
use swdir::Swdir;

fn run() {
    let report = Swdir::new().root_path(".").walk();

    if !report.is_ok() {
        for err in &report.errors {
            eprintln!("warn: {err}");
        }
    }

    let paths = report.tree.flatten_paths();
}
```

If you don't care about errors, `.walk_tree()` returns just the tree.

## GUI / lazy loading API — `scan_dir`

Alongside the high-level `Swdir::walk()` there is a low-level,
single-directory scan API aimed at GUI tree views (iced, egui, …) that
expand one node at a time. Use `scan_dir` when you want to stream a
tree into the UI as the user clicks, rather than pay the cost of a full
traversal up front.

### How it differs from `walk()`

| | `Swdir::walk()` | `scan_dir(path)` |
|---|---|---|
| Scope | recursive, whole tree | one directory, direct children only |
| Filtering / sorting | configurable via `FilterRule` | none (raw listing) |
| Parallelism | internal rayon pool | single-threaded |
| Runtime deps | none (sync) | none (sync) |
| Return type | `WalkReport` (tree + errors) | `Result<Vec<DirEntry>, ScanError>` |
| Error handling | collected into `WalkReport::errors` | atomic — first I/O failure is returned |
| Intended caller | CLI, batch tools | GUI lazy-loading, file pickers |

`scan_dir` is unchanged from 0.9.

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

Each `DirEntry` is owned (`Send + 'static`) and caches its `FileType`,
so `is_dir()` / `is_file()` / `is_symlink()` answer without additional
syscalls. Empty directories return `Ok(Vec::new())`; permission denied,
missing paths, and "not a directory" all return
`Err(ScanError::Io { path, source })` with both the offending path and
the original `std::io::Error` preserved.

### iced lazy tree example

`scan_dir` is deliberately synchronous. To keep the iced runtime
responsive, wrap the call in `std::thread::spawn` and drive it via
`Task::perform`:

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

No `tokio`, no `async-std`, no feature flag — the crate stays
runtime-agnostic.

## Feature flags

| Feature  | Default | Purpose                                             |
|----------|---------|-----------------------------------------------------|
| `filter` | ✅       | The `FilterRule` / `Decision` / `EntryKind` filter model. Leave this on unless you genuinely want a bare walker with no filtering. |

There is **no** `advanced-filter` feature. Keeping the surface small
was deliberate for 0.10 — see the design doc / CHANGELOG for why.
