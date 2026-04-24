# swdir

[![crates.io](https://img.shields.io/crates/v/swdir?label=rust)](https://crates.io/crates/swdir)
[![License](https://img.shields.io/github/license/nabbisen/swdir)](https://github.com/nabbisen/swdir/blob/main/LICENSE)
[![Rust Documentation](https://docs.rs/swdir/badge.svg?version=latest)](https://docs.rs/swdir)
[![Dependency Status](https://deps.rs/crate/swdir/latest/status.svg)](https://deps.rs/crate/swdir)

Swiftly traverse and scan directories recursively.
Sway 🪭, swing 🎷 or swim 🪼 in directories.

`swdir` is a small crate that supplies the **raw material** for a
Directory Tree widget — path listings, recursive walks, typed entries.
It does **not** draw the tree, watch for file-change events, cache
results, or interpret file contents. Those responsibilities belong to
the GUI layer on top.

## Two entry points

| Use case                                   | API                                                |
|--------------------------------------------|----------------------------------------------------|
| **Recursive** walk (batch tools, CLIs)     | `Swdir::walk`                                      |
| **Lazy-loading** one-folder scan (GUIs)    | `scan_dir` / `scan_dir_with_options`               |

Both share the same `SortOrder` concept when a reproducible display
order matters.

## Quick start — recursive walk

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

## Quick start — lazy loading (Directory Tree widgets)

`scan_dir_with_options` is the intended entry point for a GUI tree: call
it every time the user expands a folder. One directory per call, no
recursion, no hidden syscalls.

```rust
use std::path::Path;
use swdir::{ScanOptions, SortOrder, scan_dir_with_options};

fn load(folder: &Path) -> Result<(), swdir::ScanError> {
    let opts = ScanOptions::new(SortOrder::NameAscDirsFirst);
    let entries = scan_dir_with_options(folder, &opts)?;
    for entry in &entries {
        // entry.display_name() -> &OsStr (no allocation)
        // entry.is_dir()       -> cached FileType (no syscall)
        // entry.relative_to(root) -> pure path arithmetic (no I/O)
    }
    Ok(())
}
```

Raw OS order? Use bare `scan_dir(path)` — no options, no sort, cheapest.

## Ordering

Both `Swdir::walk` and `scan_dir_with_options` accept a `SortOrder`:

| Variant                         | Meaning                                                         |
|---------------------------------|-----------------------------------------------------------------|
| `SortOrder::Filesystem`         | OS `readdir` order; cheapest, but not stable across runs / filesystems. |
| `SortOrder::NameAscDirsFirst`   | Directories first, then files, each group sorted by name ascending. Default. |

```rust
use swdir::{SortOrder, Swdir};

fn run() {
    // Raw OS order for walk() — skips the in-memory sort pass.
    let report = Swdir::new()
        .root_path("/some/path")
        .sort_order(SortOrder::Filesystem)
        .walk();
}
```

Only two orderings, on purpose. If you need something else (size,
mtime, extension…), sort the returned `Vec` at the call site.

## Recursion

`Recurse` is an enum with three meaningful states:

```rust
use swdir::{Recurse, Swdir};

fn run() {
    let report = Swdir::new()
        .root_path("/some/path")
        .recurse(Recurse::Depth(1))  // only the root's immediate subdirs
        .walk();

    let report = Swdir::new()
        .root_path("/some/path")
        .recurse(Recurse::Unlimited) // whole tree
        .walk();
}
```

## Filtering

Filtering is part of the default `filter` feature. The model is built
around two types:

- `FilterRule` — the condition (hidden, extension, path prefix, kind, depth)
- `Decision` — what to do about a given entry, split into two axes:
  - `include` — should it appear in the results?
  - `descend` — should the walker look inside it?

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

To see hidden entries, clear the default rules first:

```rust
use swdir::Swdir;

fn run() {
    let report = Swdir::new()
        .root_path("/some/path")
        .clear_filters()
        .walk();
}
```

### `include` vs `descend`

Separating the two axes means a rule can hide a directory from the
result tree while still descending through it. For instance,
`FilterRule::only_kind(EntryKind::File)` drops directories from the
output but keeps walking into them so nested files remain reachable —
which is what callers usually want when they say "give me just the
files under here".

### What ships in 0.11

| Rule                                 | Purpose                                              |
|--------------------------------------|------------------------------------------------------|
| `FilterRule::SkipHidden`             | Drop entries whose name starts with `.` (plus Windows hidden-bit). |
| `FilterRule::OnlyKinds(..)`          | Keep only files / dirs / symlinks.                   |
| `FilterRule::ExtensionAllowlist(..)` | Keep files with these extensions.                    |
| `FilterRule::ExtensionDenylist(..)`  | Drop files with these extensions.                    |
| `FilterRule::UnderPath(..)`          | Restrict to entries under a path prefix.             |
| `FilterRule::NotUnderPath(..)`       | Exclude everything under a path prefix.              |
| `FilterRule::MaxDepth(n)`            | Cap depth of entries and descent.                    |

The enum is `#[non_exhaustive]`, so future additions won't break
existing `match` statements. Advanced filter families (regex, glob,
metadata predicates, arbitrary closures) are deliberately out of scope
— bring those needs upstream if they come up in practice.

## Error handling

`Swdir::walk()` returns a `WalkReport`. Unreadable directories go into
`report.errors` instead of being printed to stderr:

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

`scan_dir` / `scan_dir_with_options` are **atomic** instead — on the
first I/O failure they return `Err(ScanError::Io { .. })`. The trade-off
matches the use case: batch walks want partial results, a single
GUI-node expansion wants all-or-nothing.

## DirEntry — GUI-friendly helpers

`DirEntry` (the type returned by the scan functions) caches the
entry's `FileType`, so `is_dir()` / `is_file()` / `is_symlink()` never
re-syscall — even after the underlying file has been removed. 0.11 adds
two thin conveniences for tree widgets:

| Method                             | Returns          | Notes                           |
|------------------------------------|------------------|---------------------------------|
| `DirEntry::display_name()`         | `&OsStr`         | Borrowed, no allocation.        |
| `DirEntry::relative_to(&self, root)` | `Option<PathBuf>` | Pure path arithmetic, no I/O. |

That's the full list. The crate deliberately stops here — GUI tree
data structures, routing, and state management are the widget's job.

## iced lazy tree example

`scan_dir_with_options` is deliberately synchronous. To keep the iced
runtime responsive, wrap it in `std::thread::spawn` and drive it via
`Task::perform`:

```rust,ignore
use std::path::PathBuf;
use iced::Task;
use swdir::{DirEntry, ScanError, ScanOptions, SortOrder, scan_dir_with_options};

# enum Message { Loaded(Result<Vec<DirEntry>, ScanError>) }
fn load(path: PathBuf) -> Task<Message> {
    let opts = ScanOptions::new(SortOrder::NameAscDirsFirst);
    Task::perform(
        async move {
            std::thread::spawn(move || scan_dir_with_options(&path, &opts))
                .join()
                .expect("scan must not panic")
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
| `filter` | ✅      | The `FilterRule` / `Decision` / `EntryKind` filter model. |

No `advanced-filter`. No `async`. No `watcher`. Scope creep is the
enemy.

## Migrating from 0.10

0.11 is additive on top of 0.10. Existing 0.10 code keeps working
unchanged. Optional adjustments to take advantage of the new surface:

| 0.10                                         | 0.11 (optional)                                            |
|----------------------------------------------|------------------------------------------------------------|
| Bespoke `walk()` + sort in your own code     | `.sort_order(SortOrder::Filesystem)` to skip the sort      |
| `entry.path().strip_prefix(root).ok()`       | `entry.relative_to(root)`                                  |
| `entry.file_name()` for GUI labels           | `entry.display_name()` (borrowed `&OsStr`, no alloc)       |
| `scan_dir(path)` (raw order)                 | `scan_dir_with_options(path, &ScanOptions::default())`     |

See `CHANGELOG.md` for the full list. The `Swdir`, `WalkReport`,
`FilterRule`, `Decision`, `Recurse`, `scan_dir`, and `DirEntry` APIs
from 0.10 are unchanged.
