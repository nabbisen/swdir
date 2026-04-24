# Getting started

This page walks you from zero to a working swdir integration in five
minutes. For reference material on individual types, see
[`../reference/`](reference/). For the two end-to-end use cases, see
the [`../guides/`](guides/) pages.

## Install

```sh
cargo add swdir
```

`swdir` compiles on stable Rust (edition 2024) and has two runtime
dependencies: `rayon` and `thiserror`. No async runtime is required
or assumed.

## Two entry points

swdir gives you exactly two ways to look at a directory tree:

| Use case                                     | API                                       |
|----------------------------------------------|-------------------------------------------|
| **Recursive walk** (batch tools, CLIs)       | [`Swdir::walk`][walk]                     |
| **Lazy one-folder scan** (GUI tree widgets)  | [`scan_dir`][sd] / [`scan_dir_with_options`][sdo] |

Pick the one that matches how your caller consumes the results.

[walk]: guides/recursive-walk.md
[sd]:   guides/lazy-loading.md
[sdo]:  guides/lazy-loading.md

## Your first walk

```rust
use swdir::Swdir;

fn main() {
    let report = Swdir::new().root_path(".").walk();

    println!("{} paths, {} errors",
        report.tree.flatten_paths().len(),
        report.errors.len(),
    );

    for err in &report.errors {
        eprintln!("warn: {err}");
    }
}
```

What just happened:

- `Swdir::new()` builds a walker with sensible defaults: skip hidden
  entries, sort directories first and then files by name, no recursion.
- `.root_path(".")` tells it where to start.
- `.walk()` runs the scan and returns a `WalkReport { tree, errors }`.
  Any unreadable directory lands in `errors` — `walk` never panics or
  writes to stderr.

To recurse, add `.recurse(...)`:

```rust
use swdir::{Recurse, Swdir};

let report = Swdir::new()
    .root_path(".")
    .recurse(Recurse::Unlimited)
    .walk();
```

See the [recursive-walk guide](guides/recursive-walk.md) for the rest
of the builder surface.

## Your first lazy scan

A GUI tree widget expands folders on demand — each click calls
swdir once for that folder, no recursion:

```rust
use std::path::Path;
use swdir::{ScanOptions, SortOrder, scan_dir_with_options};

fn on_expand(folder: &Path) -> Result<(), swdir::ScanError> {
    let opts    = ScanOptions::new(SortOrder::NameAscDirsFirst);
    let entries = scan_dir_with_options(folder, &opts)?;
    for entry in &entries {
        // entry.display_name() -> &OsStr  (no allocation)
        // entry.is_dir()       -> cached  (no syscall)
    }
    Ok(())
}
```

- `scan_dir_with_options` is **atomic**: the first I/O failure
  returns `Err(ScanError::Io { path, source })`, no partial results.
- The returned `Vec<DirEntry>` is `Send + 'static`, safe to move to a
  worker thread.

See the [lazy-loading guide](guides/lazy-loading.md) for the iced
integration pattern and the `DirEntry` helpers.

## Adding a filter

Filtering is on by default (the `filter` feature). One rule is already
installed: `FilterRule::SkipHidden`. Stack more with `.filter(...)`:

```rust
use swdir::{FilterRule, Recurse, Swdir, SwdirError};

fn run() -> Result<(), SwdirError> {
    let report = Swdir::new()
        .root_path(".")
        .recurse(Recurse::Unlimited)
        .filter(FilterRule::extension_allowlist(["md", "rs"])?)
        .filter(FilterRule::max_depth(3))
        .walk();
    Ok(())
}
```

The full rule catalog, including the `include` / `descend` model, is in
the [filter-rules reference](reference/filter-rules.md).

## Choosing a sort order

`Swdir::walk` and `scan_dir_with_options` both accept a
[`SortOrder`](reference/sort-order.md):

- `SortOrder::NameAscDirsFirst` (default) — reproducible, GUI-friendly.
- `SortOrder::Filesystem` — raw `readdir` order, cheapest.

If you need a different order (size, mtime, extension), sort the
returned `Vec` at the call site. swdir stops here on purpose.

## Where to go next

- **Recursive walks in depth** → [guides/recursive-walk.md](guides/recursive-walk.md)
- **GUI lazy loading** → [guides/lazy-loading.md](guides/lazy-loading.md)
- **Why is the API so small?** → [design-notes.md](design-notes.md)
- **Upgrading from 0.10 or 0.9** → [migration/](migration/)
