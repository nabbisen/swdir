# Migrating from 0.9 to 0.10

0.10 reshaped the `Swdir` API around a unified filter model and an
explicit error contract. This is the breaking release; 0.11 on top of
it is purely additive.

## Breaking changes at a glance

| 0.9                                                   | 0.10                                                                 |
|-------------------------------------------------------|----------------------------------------------------------------------|
| `Swdir::default().set_root_path(p)`                   | `Swdir::new().root_path(p)`                                          |
| `.set_recurse(Recurse { enabled, depth_limit })`      | `.recurse(Recurse::None \| Recurse::Unlimited \| Recurse::Depth(n))` |
| `.include_hidden()`                                   | `.clear_filters()` *(drops the default `FilterRule::SkipHidden`)*    |
| `.set_extension_allowlist(&["md"])?`                  | `.filter(FilterRule::extension_allowlist(["md"])?)`                  |
| `.set_extension_denylist(&["md"])?`                   | `.filter(FilterRule::extension_denylist(["md"])?)`                   |
| `walk() -> DirNode`                                   | `walk() -> WalkReport { tree, errors }` *(or `walk_tree()`)*         |
| Stderr warnings on I/O failure                        | `WalkReport::errors`                                                 |
| `SwdirError::DuplicateExtensionList`                  | *(removed — allow and deny stack freely)*                            |

`scan_dir` is unchanged from 0.9.

## Minimal upgrade example

```rust,ignore
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
    .clear_filters()    // drops default SkipHidden
    .filter(FilterRule::extension_allowlist(["md"])?)
    .walk();
let tree = report.tree;
```

## The new ideas

### Filter model

Filtering is now one enum, `FilterRule`, with validated constructors
and the two-axis `Decision { include, descend }` model. Rules compose
with AND, so allowlist + denylist now stack freely — no more
`DuplicateExtensionList`.

See the [filter-rules reference](../reference/filter-rules.md).

### `Recurse` is an enum

```rust
pub enum Recurse {
    None,
    Unlimited,
    Depth(usize),
}
```

Replaces 0.9's two-field struct. The impossible combination
`enabled = false, depth_limit = Some(_)` is no longer representable.

### `walk()` returns a report

```rust
pub struct WalkReport {
    pub tree: DirNode,
    pub errors: Vec<WalkError>,
}
```

Errors are collected, not printed. Call `report.is_ok()` to check,
iterate `report.errors` to react. Use `.walk_tree()` if you don't want
the errors at all.

See the [error-handling reference](../reference/error-handling.md).

### Default `filter` feature

Filtering is now a feature flag, default-on. A minimal configuration
(`default-features = false`) strips the filter model and gives you a
bare walker. See [feature-flags.md](../reference/feature-flags.md).

## What's next

Once you're on 0.10, the 0.11 additions (sort order, GUI helpers,
`scan_dir_with_options`) are purely additive — see
[from-0-10-to-0-11.md](from-0-10-to-0-11.md).
