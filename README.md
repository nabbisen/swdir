# swdir

[![crates.io](https://img.shields.io/crates/v/swdir?label=rust)](https://crates.io/crates/swdir)
[![License](https://img.shields.io/github/license/nabbisen/swdir)](https://github.com/nabbisen/swdir/blob/main/LICENSE)
[![Rust Documentation](https://docs.rs/swdir/badge.svg?version=latest)](https://docs.rs/swdir)
[![Dependency Status](https://deps.rs/crate/swdir/latest/status.svg)](https://deps.rs/crate/swdir)

Swiftly traverse and scan directories.
Sway 🪭, swing 🎷 or swim 🪼 in directories.

## Overview

`swdir` is a small, focused crate for walking and listing directories.
It provides two entry points — one for recursive traversal, one for
single-directory lazy loading — with predictable ordering, a unified
filter model, and explicit error reporting.

## When to use it

- You're writing a **CLI or batch tool** that needs a fast recursive
  walk with filtering.
- You're building a **GUI Directory Tree widget** (iced, egui, Tauri, …)
  that expands one folder at a time as the user clicks.
- You want a walker that stays small and predictable — no async,
  no file watching, no caching, no surprises.

## Quick start

```sh
cargo add swdir
```

Recursive walk:

```rust
use swdir::Swdir;

let report = Swdir::new().root_path("/some/path").walk();
let paths  = report.tree.flatten_paths();
```

Single-folder lazy load (what a GUI tree widget calls on each expand):

```rust
use std::path::Path;
use swdir::{ScanOptions, SortOrder, scan_dir_with_options};

fn load(folder: &Path) -> Result<(), swdir::ScanError> {
    let opts    = ScanOptions::new(SortOrder::NameAscDirsFirst);
    let entries = scan_dir_with_options(folder, &opts)?;
    // entries: Vec<DirEntry>
    Ok(())
}
```

More in [docs/getting-started.md](https://github.com/nabbisen/swdir/blob/main/docs/getting-started.md).

## Features

**`Swdir::walk` — recursive traversal**

- Builder-style configuration: `root_path`, `recurse`, `filter`, `sort_order`.
- Returns a `WalkReport { tree, errors }` so partial I/O failures are
  observable, never silently swallowed.
- Parallel internally (rayon); synchronous at the call site.

**`scan_dir` / `scan_dir_with_options` — lazy single-folder listing**

- One directory per call; atomic `Result` — perfect for tree-expand
  callbacks.
- Entries are owned (`Send + 'static`) and cache their `FileType`, so
  `is_dir()` / `is_file()` never re-syscall.
- `DirEntry::display_name()` and `relative_to()` give GUIs labels and
  relative paths without extra allocations or I/O.

## Design notes

- **Small surface, on purpose.** Two entry points, one filter model,
  two sort orders. No regex filters, no async variant, no watcher, no
  built-in cache. The sharp edges stay inside the widget or runtime
  that consumes swdir.
- **Predictable over clever.** `SortOrder::NameAscDirsFirst` is the
  default because GUI trees need reproducible order; `Filesystem` is
  there when you want raw `readdir` speed.
- **Errors belong in the API, not stderr.** `walk` collects errors,
  `scan_dir` returns them. The caller decides.

## Documentation

For everything beyond the quick start, see **[docs/](https://github.com/nabbisen/swdir/tree/main/docs)**.
A few commonly-wanted entry points:

- [Getting started](https://github.com/nabbisen/swdir/blob/main/docs/getting-started.md) — a complete walk-through.
- [Guide: recursive walks](https://github.com/nabbisen/swdir/blob/main/docs/guides/recursive-walk.md) — `Swdir::walk`
  in depth.
- [Guide: lazy loading](https://github.com/nabbisen/swdir/blob/main/docs/guides/lazy-loading.md) — single-folder
  scan for Directory Tree widgets, with an iced example.
- [Reference: filter rules](https://github.com/nabbisen/swdir/blob/main/docs/reference/filter-rules.md) — the full
  `FilterRule` catalog and the `include` vs `descend` model.
- [Reference: sort order](https://github.com/nabbisen/swdir/blob/main/docs/reference/sort-order.md) — the two
  orderings and when to pick which.
- [Reference: error handling](https://github.com/nabbisen/swdir/blob/main/docs/reference/error-handling.md) —
  `WalkReport` vs `ScanError`.
- [Design notes](https://github.com/nabbisen/swdir/blob/main/docs/design-notes.md) — the "why" behind what's in
  and what's out.
- [Migration notes](https://github.com/nabbisen/swdir/tree/main/docs/migration) — upgrading from older versions.

Happy walking — wherever your directories take you.
