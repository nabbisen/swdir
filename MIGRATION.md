# Migration

## Migrating from 0.9

0.10 breaks the `Swdir` API on purpose to unify filtering and make
partial failures observable. The crate's top-level docs carry a full
mapping; the short version:

| 0.9                                              | 0.10                                                         |
|--------------------------------------------------|--------------------------------------------------------------|
| `Swdir::default().set_root_path(p)`              | `Swdir::new().root_path(p)`                                  |
| `.set_recurse(Recurse { enabled, depth_limit })` | `.recurse(Recurse::None \| ::Unlimited \| ::Depth(n))`       |
| `.include_hidden()`                              | `.clear_filters()`                                           |
| `.set_extension_allowlist(&["md"])?`             | `.filter(FilterRule::extension_allowlist(["md"])?)`          |
| `.set_extension_denylist(&["md"])?`              | `.filter(FilterRule::extension_denylist(["md"])?)`           |
| `walk() -> DirNode`                              | `walk() -> WalkReport { tree, errors }` *(or `walk_tree()`)* |
| Stderr warnings on I/O failure                   | `WalkReport::errors`                                         |
| `SwdirError::DuplicateExtensionList`             | *(removed — allow and deny stack freely)*                    |

`scan_dir` is unchanged.

### Minimal upgrade example

The crate docs (`src/lib.rs`) and `README.md` both carry a 0.9→0.10 mapping
table.

```rust
// 0.9
let tree = Swdir::default()
    .set_root_path("/some/path")
    .set_recurse(Recurse { enabled: true, depth_limit: Some(1) })
    .include_hidden()
    .set_extension_allowlist(&["md"])?
    .walk();

// 0.10
let report = Swdir::new()
    .root_path("/some/path")
    .recurse(Recurse::Depth(1))
    .clear_filters() // drops default SkipHidden
    .filter(FilterRule::extension_allowlist(["md"])?)
    .walk();
let tree = report.tree;
```