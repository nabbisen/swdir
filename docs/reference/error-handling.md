# Reference: error handling

swdir has two error shapes, one per entry point:

| Entry point                             | Error contract                                   |
|-----------------------------------------|--------------------------------------------------|
| `Swdir::walk`                           | Partial — errors go into `WalkReport::errors`.   |
| `scan_dir` / `scan_dir_with_options`    | Atomic — the first I/O error is returned.        |

The two shapes match the use cases. A batch walk over a large tree
benefits from "return what you found, report what you couldn't read";
a GUI single-folder scan wants all-or-nothing.

## `WalkReport` — partial failures

```rust
pub struct WalkReport {
    pub tree: DirNode,
    pub errors: Vec<WalkError>,
}
```

Every unreadable directory, every `file_type()` failure, every entry
iteration error is appended to `errors` with its path attached. The
walk does **not** unwind: an on-error directory appears in the tree
with empty `files` / `sub_dirs`, and the walk continues elsewhere.

```rust
use swdir::Swdir;

let report = Swdir::new().root_path("/").walk();

if !report.is_ok() {
    for err in &report.errors {
        eprintln!("{} ({:?})", err, err.io_kind());
    }
}
// still usable:
let paths = report.tree.flatten_paths();
```

Convenience methods:

- `report.is_ok()` — `true` iff `errors.is_empty()`.
- `report.into_tree()` — consume and return the `DirNode`.
- `Swdir::walk_tree()` — shortcut that discards errors entirely.

### Why not panic or stderr?

Older versions of swdir wrote warnings to stderr on I/O failure. That
silently broke batch tools (stderr wasn't captured), hid
permission-denied subtrees in production, and couldn't be tested. 0.10
moved the contract into the return type, and 0.11 keeps it there.

## `WalkError`

```rust
#[derive(Error, Debug)]
pub enum WalkError {
    #[error("I/O error at {}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source] source: io::Error,
    },
}
```

Helpers:

- `err.path() -> &Path`
- `err.io_kind() -> io::ErrorKind`

The enum is kept small intentionally — it covers every class of
failure the walker can encounter today. If a future variant becomes
necessary it will be added in a minor release behind
`#[non_exhaustive]`.

## `ScanError` — atomic

```rust
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("I/O error at {}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source] source: io::Error,
    },
}
```

Same shape as `WalkError`, different use site. Any failure from
`read_dir`, from iterating entries, or from resolving a child's
`FileType` converts to a `ScanError::Io` and aborts the whole scan.
Partial results are never produced.

Same helpers:

- `err.path() -> &Path`
- `err.io_kind() -> io::ErrorKind`

Typical use in a GUI tree-view callback:

```rust
use std::path::Path;
use swdir::{ScanError, scan_dir};

let folder = Path::new(".");

match scan_dir(folder) {
    Ok(entries) => { /* render children */ }
    Err(ScanError::Io { path, source }) => {
        // badge the node as inaccessible, log once, move on
        eprintln!("{}: {}", path.display(), source);
    }
}
```

## `SwdirError` — construction-time validation

A separate, tiny type for problems caught when *building* a filter:

```rust
pub enum SwdirError {
    InvalidExtensionListItem(String),
}
```

Returned only by the extension-list constructors:

```rust
use swdir::FilterRule;

FilterRule::extension_allowlist([".md"])  // => Err(SwdirError::InvalidExtensionListItem(".md"))
    .unwrap_err();
```

Not mixed with runtime `WalkError` / `ScanError` on purpose —
construction errors surface before you ever touch the filesystem.

## Common patterns

**"Fail on any error"** — use `scan_dir` for the single-folder case;
for walks, check `report.is_ok()` and bail:

```rust
use swdir::Swdir;

let report = Swdir::new().root_path(".").walk();
if !report.is_ok() {
    // ...map to your own error
}
```

**"Log and continue"** — iterate `errors`:

```rust
use swdir::Swdir;

let report = Swdir::new().root_path(".").walk();
for err in &report.errors {
    eprintln!("swdir: {err}");
}
```

**"I'm feeling lucky"** — `walk_tree()` or `.unwrap()` on `scan_dir`:

```rust
use std::path::Path;
use swdir::{Swdir, scan_dir};

let p = Path::new(".");

let tree    = Swdir::new().root_path(".").walk_tree();       // drops errors
let entries = scan_dir(p).unwrap();                          // panics on I/O failure
```
