# Reference: `SortOrder`

`SortOrder` controls the order of entries in both recursive and
single-folder scans.

## The two orderings

```rust
pub enum SortOrder {
    Filesystem,
    NameAscDirsFirst,
}
```

| Variant                        | Order                                                                 | Cost                                 | When to use                            |
|--------------------------------|-----------------------------------------------------------------------|--------------------------------------|----------------------------------------|
| `SortOrder::Filesystem`        | Whatever the OS's `readdir` returns.                                  | No sort step.                        | CLIs that stream output; speed-first.  |
| `SortOrder::NameAscDirsFirst`  | Directories A–Z first, then files A–Z. Each group sorted by name.    | One in-memory `sort_by` per level.   | Everything user-facing. **Default.**  |

"Name" sorts by the final path component, case-sensitive, using the
byte-wise ordering of `OsStr`.

## Using it with `Swdir::walk`

```rust
use swdir::{SortOrder, Swdir};

let report = Swdir::new()
    .root_path(".")
    .sort_order(SortOrder::Filesystem) // raw readdir order
    .walk();
```

Default is `NameAscDirsFirst`, so `Swdir::new().walk()` already
produces reproducible, GUI-ready output.

## Using it with `scan_dir_with_options`

```rust
use std::path::Path;
use swdir::{ScanOptions, SortOrder, scan_dir_with_options};

let opts    = ScanOptions::new(SortOrder::NameAscDirsFirst);
let entries = scan_dir_with_options(Path::new("."), &opts)?;
```

The bare `scan_dir(path)` function is equivalent to
`scan_dir_with_options(path, &ScanOptions::new(SortOrder::Filesystem))`
— use whichever name reads better at your call site.

## `ScanOptions`

```rust
#[non_exhaustive]
pub struct ScanOptions {
    pub sort_order: SortOrder,
}
```

`#[non_exhaustive]` leaves room for future fields without breaking
existing code. Build via `ScanOptions::new(SortOrder)` or
`ScanOptions::default()` (which yields `NameAscDirsFirst`).

## Why only two?

The crate doesn't ship size-based, mtime-based, or extension-based
orderings — sort the returned `Vec` at the call site when you need
those. Keeping the enum small keeps the API readable and the sort
policy obvious. See [design-notes.md](../design-notes.md) for more.

## Reproducibility

Both orderings are deterministic *given identical filesystem contents*.
Two consecutive `walk()` or `scan_dir_with_options()` calls on an
unchanged directory return byte-identical sequences — the guarantee a
Directory Tree widget depends on to avoid flicker on re-expand.

`Filesystem` order is deterministic *per OS / filesystem / session*
but not portable: an `ext4` volume, an `APFS` volume, and a Windows
NTFS volume may each iterate the same folder in different orders.
Don't rely on it for tests or cross-platform behavior.
