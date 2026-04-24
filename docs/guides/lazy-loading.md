# Guide: lazy loading for Directory Tree widgets

`scan_dir` and `scan_dir_with_options` are the lightweight, one-folder
scan functions. They exist for GUI tree widgets (iced, egui, Tauri,
Dioxus, …) that expand one directory at a time as the user clicks.

For batch / recursive scanning, see the
[recursive-walk guide](recursive-walk.md).

## The contract

- **One directory, non-recursive.** Subdirectories show up in the
  result but are not opened.
- **Sync, single-threaded.** No async runtime, no rayon. Wrap with
  `std::thread::spawn` if you need off-thread execution.
- **Atomic.** On the first I/O failure, the function returns
  `Err(ScanError::Io { path, source })` — no partial results. The
  widget shows an error badge on that node and moves on.
- **Cheap to keep around.** Returned `DirEntry` values are owned
  (`'static + Send`) and cache their `FileType`, so they survive
  thread hops and answer `is_dir()` / `is_file()` without re-syscalls.

## Two functions, same shape

```rust
pub fn scan_dir(path: &Path) -> Result<Vec<DirEntry>, ScanError>;
pub fn scan_dir_with_options(path: &Path, options: &ScanOptions)
    -> Result<Vec<DirEntry>, ScanError>;
```

- `scan_dir(path)` returns entries in raw OS `readdir` order — cheapest.
- `scan_dir_with_options(path, &opts)` lets you pick a [`SortOrder`](../reference/sort-order.md).

Default `ScanOptions::default()` yields `SortOrder::NameAscDirsFirst`,
the shape GUI trees want. Prefer this for widgets:

```rust
use std::path::Path;
use swdir::{ScanOptions, SortOrder, scan_dir_with_options};

let opts    = ScanOptions::new(SortOrder::NameAscDirsFirst);
let entries = scan_dir_with_options(Path::new("/some/folder"), &opts)?;
```

## `DirEntry` — what the widget works with

Each entry carries just what a tree view needs:

| Method                         | Returns          | Cost                     |
|--------------------------------|------------------|--------------------------|
| `path()`                       | `&Path`          | free                     |
| `display_name()`               | `&OsStr`         | free, no allocation      |
| `file_name()`                  | `OsString`       | one allocation           |
| `is_dir()` / `is_file()` / `is_symlink()` | `bool` | cached, no syscall      |
| `file_type()`                  | `FileType`       | cached                   |
| `metadata()`                   | `Option<&Metadata>` | cached at scan time   |
| `relative_to(root)`            | `Option<PathBuf>` | pure path arithmetic   |

Typical GUI flow:

```rust
use std::path::Path;
use swdir::{ScanOptions, scan_dir_with_options};

let folder = Path::new(".");
let root   = Path::new(".");

let entries = scan_dir_with_options(folder, &ScanOptions::default())?;
for e in &entries {
    let label = e.display_name();       // &OsStr — stick directly in a text widget
    let key   = e.relative_to(root);    // Option<PathBuf> — routing key
    let open  = e.is_dir();             // show the expand chevron
    // ...
}
```

`display_name()` borrows from the entry's cached path, so it's cheaper
than `file_name()` (which allocates an owned `OsString`). Use it when
you just want to render the name.

## iced integration

iced schedules work via `Task`. Because `scan_dir_with_options` is
blocking, wrap it in a worker thread so the runtime keeps ticking:

```rust,ignore
use std::path::PathBuf;
use iced::Task;
use swdir::{DirEntry, ScanError, ScanOptions, SortOrder, scan_dir_with_options};

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

The same pattern works with any runtime: spawn a blocking worker, send
the `Result` back on completion. `DirEntry` is `Send + 'static`, so
crossing threads is a non-event.

## A note on ordering and stability

A GUI tree that reorders on every expand is jarring. swdir guarantees
that repeated calls with the same `SortOrder` produce the same
sequence (it's a pure sort over cached data), so rendering is stable
out of the box.

If you want raw `readdir` speed and don't care about ordering — an
iterator-first file picker, say — use `scan_dir(path)` (no options).

## Error handling in a tree view

`ScanError::Io` carries both the path and the original `io::Error`:

```rust
use swdir::{ScanError, scan_dir};
use std::path::Path;

let path = Path::new(".");

match scan_dir(path) {
    Ok(entries) => { /* render */ }
    Err(ScanError::Io { path, source }) => {
        // e.g. show a lock icon on the folder node and log
        eprintln!("couldn't open {}: {source}", path.display());
    }
}
```

See the [error-handling reference](../reference/error-handling.md) for
the `ScanError` vs `WalkError` comparison.
