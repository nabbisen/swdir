# Reference: `FilterRule`

Filtering in swdir is one enum: `FilterRule`. Rules compose with AND
via `Swdir::filter(...)`, so stacking them is always safe — a rule
can only be more restrictive, never more permissive.

Availability: filtering is behind the default `filter` feature. If you
disable default features, `FilterRule` and its friends go away and
`Swdir::walk` returns every entry it can read.

## The rule catalog

| Rule                                 | What it does                                                           |
|--------------------------------------|------------------------------------------------------------------------|
| `FilterRule::SkipHidden`             | Drop entries whose name starts with `.` (plus Windows hidden bit).     |
| `FilterRule::OnlyKinds(..)`          | Keep only the listed [`EntryKind`](#entrykind)s.                        |
| `FilterRule::ExtensionAllowlist(..)` | Keep files with these extensions. Non-files pass unchanged.            |
| `FilterRule::ExtensionDenylist(..)`  | Drop files with these extensions. Non-files pass unchanged.            |
| `FilterRule::UnderPath(..)`          | Restrict to entries under a given path prefix.                          |
| `FilterRule::NotUnderPath(..)`       | Exclude everything under a given path prefix.                           |
| `FilterRule::MaxDepth(n)`            | Cap the depth of included entries and descent.                          |

The enum is `#[non_exhaustive]`, so future additions won't break
existing `match` statements.

## Default rule: `SkipHidden`

`Swdir::new()` installs one rule by default: `FilterRule::SkipHidden`.
That means the common case — "walk this tree, don't bother with
dotfiles" — is a one-liner. To include hidden entries, clear the
filters:

```rust
use swdir::Swdir;

let report = Swdir::new()
    .root_path(".")
    .clear_filters()    // drops the default SkipHidden
    .walk();
```

## Validated constructors

Extension lists must not start with a `.`:

```rust
use swdir::FilterRule;

fn demo() -> Result<(), swdir::SwdirError> {
    FilterRule::extension_allowlist(["md", "rs"])?;   // OK
    FilterRule::extension_allowlist([".md"]);         // Err(InvalidExtensionListItem)
    Ok(())
}
```

`std::path::Path::extension()` never includes the leading dot, so
`".md"` would silently match nothing — the constructor catches that
up front. The validation applies to both allowlist and denylist.

## `include` vs `descend` — the two-axis model

Each rule returns a `Decision` with two independent axes:

```rust
pub struct Decision {
    pub include: bool,   // appears in the result tree?
    pub descend: bool,   // walker enters this directory's children?
}
```

Rules are combined by ANDing both axes separately. That separation is
load-bearing: a rule can hide a directory from the output but still
descend through it, which is what you want for something like
"give me all .rs files under here":

```rust
use swdir::{EntryKind, FilterRule, Recurse, Swdir};

let report = Swdir::new()
    .root_path(".")
    .recurse(Recurse::Unlimited)
    .filter(FilterRule::only_kind(EntryKind::File))
    .walk();
```

Under the hood, `OnlyKinds(File)` returns `include = false, descend =
true` for directories — the `HIDDEN_PASSTHROUGH` decision. Without the
split, the walk would never reach the files inside.

Three useful constants model the common shapes:

| Constant                      | `include` | `descend` | Meaning                                   |
|-------------------------------|-----------|-----------|-------------------------------------------|
| `Decision::PASS`              | `true`    | `true`    | Include entry; keep walking.              |
| `Decision::DROP`              | `false`   | `false`   | Exclude entry; stop here.                 |
| `Decision::HIDDEN_PASSTHROUGH`| `false`   | `true`    | Hide entry; keep walking through it.      |

## `EntryKind`

```rust
pub enum EntryKind {
    File,
    Dir,
    Symlink,
}
```

Mirrors `std::fs::FileType` as a pattern-matchable enum. Symlinks are
reported as `Symlink` and are **not** followed — swdir never chases a
symlink into another subtree.

## What's **not** a rule

- No regex rules.
- No glob rules.
- No metadata rules (size, mtime, mode…).
- No user-supplied closure rules.

These are deliberately out of scope — see [design-notes.md](../design-notes.md).
If you need them, filter the returned `Vec<PathBuf>` / walked
`DirNode` in your own code. The `#[non_exhaustive]` attribute reserves
the option of adding them later if real usage demands it.

## See also

- [Guide: recursive walks](../guides/recursive-walk.md) — how filters
  fit into `Swdir::walk`.
- [Reference: sort order](sort-order.md) — filtering happens before
  sorting, so filtered results are still sorted.
