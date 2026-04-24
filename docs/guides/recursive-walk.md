# Guide: recursive walks with `Swdir::walk`

`Swdir::walk` is the batch API: point it at a root directory, get back
a [`WalkReport`](../reference/error-handling.md) containing the
recursive tree and any I/O errors encountered along the way. It's the
right tool for CLIs, static analysis passes, backup tools, and any
flow where "give me everything under here" is the whole job.

For single-folder / GUI lazy loading, see the
[lazy-loading guide](lazy-loading.md) instead.

## The builder

```rust
use swdir::{FilterRule, Recurse, SortOrder, Swdir, SwdirError};

let report = Swdir::new()
    .root_path("/workspace")
    .recurse(Recurse::Unlimited)
    .filter(FilterRule::extension_allowlist(["md", "rs"])?)
    .sort_order(SortOrder::NameAscDirsFirst)
    .walk();
```

Every configuration method takes `self` and returns `Self`, so chains
read top-to-bottom. Call `.walk()` at the end to run the scan.

| Method                        | What it sets                                          | Default                      |
|-------------------------------|-------------------------------------------------------|------------------------------|
| `root_path(path)`             | Where to start scanning.                              | `"."`                        |
| `recurse(Recurse)`            | How deep to descend.                                  | `Recurse::None`              |
| `filter(FilterRule)`          | Appends one filter rule. AND-composed.                | one rule: `SkipHidden`       |
| `filters(iter)`               | Appends several rules at once.                        | —                            |
| `clear_filters()`             | Empties the rule list.                                | —                            |
| `sort_order(SortOrder)`       | Result ordering.                                      | `NameAscDirsFirst`           |
| `max_threads(n)`              | Caps the internal rayon pool.                         | `8`                          |

## Recursion policy

`Recurse` is an enum with three variants:

```rust
use swdir::{Recurse, Swdir};

let root = ".";

Swdir::new().root_path(root).recurse(Recurse::None);           // root only
Swdir::new().root_path(root).recurse(Recurse::Depth(2));       // at most 2 levels deep
Swdir::new().root_path(root).recurse(Recurse::Unlimited);      // whole tree
```

`Recurse::Depth(n)` and [`FilterRule::MaxDepth(n)`](../reference/filter-rules.md)
overlap. Either works; if both are set, the stricter one wins (they
AND together).

## Filtering

Every entry the walker sees is evaluated against each installed filter
rule. Rules AND together: an entry is kept only if every rule agrees.

```rust
let report = Swdir::new()
    .root_path("/workspace")
    .filter(FilterRule::extension_allowlist(["md"])?)
    .filter(FilterRule::not_under_path("/workspace/target"))
    .walk();
```

`Swdir::new()` already installs `FilterRule::SkipHidden`. To keep
dotfiles, either clear everything with `.clear_filters()` or install
your own rule set from scratch.

For the full rule catalog and the `include` / `descend` split, see
the [filter-rules reference](../reference/filter-rules.md).

## The result: `WalkReport`

```rust
pub struct WalkReport {
    pub tree: DirNode,
    pub errors: Vec<WalkError>,
}
```

- `tree` is the recursive directory structure — `sub_dirs` and `files`
  at each level, sorted according to the configured [`SortOrder`](../reference/sort-order.md).
- `errors` collects every I/O failure encountered. An empty vec means
  the walk finished cleanly; `report.is_ok()` is the convenience check.

Useful helpers on `DirNode`:

- `flatten_paths() -> Vec<PathBuf>` — every file path in the tree,
  depth-first.
- `count() -> DirNodeCount { files, dirs }` — totals.

If you don't care about errors at all, `walk_tree()` gives you the
`DirNode` directly:

```rust
use swdir::Swdir;

let tree = Swdir::new().root_path(".").walk_tree();
```

## Error handling

`walk` never panics and never writes to stderr. Every problem becomes
a [`WalkError`](../reference/error-handling.md) with the path attached:

```rust
use swdir::Swdir;

let report = Swdir::new().root_path("/").walk();
for err in &report.errors {
    eprintln!("{} ({:?})", err, err.io_kind());
}
```

See the [error-handling reference](../reference/error-handling.md) for
the contract details.

## Ordering

By default, entries at each directory level are sorted dirs-first then
files-alphabetical — a reproducible, GUI-friendly order. If you don't
need that stability and want the cheapest possible walk, opt into
`SortOrder::Filesystem`:

```rust
use swdir::{SortOrder, Swdir};

let report = Swdir::new()
    .root_path(".")
    .sort_order(SortOrder::Filesystem)
    .walk();
```

Details in the [sort-order reference](../reference/sort-order.md).

## Thread-pool tuning

The walker uses rayon internally. The default pool size is 8. Raise it
for deep trees on fast storage; lower it when you have other CPU work
sharing the process.

```rust
use swdir::Swdir;

Swdir::new().root_path(".").max_threads(2);
```

`scan_dir` does **not** use rayon — it's intentionally single-threaded
I/O. See the [lazy-loading guide](lazy-loading.md).
