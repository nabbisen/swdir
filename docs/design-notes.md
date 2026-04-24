# Design notes

swdir tries to be the smallest crate that does its job well. This page
makes the shape of "small" explicit.

## Philosophy

**One job: give the caller typed, traversable directory data.**
Anything that isn't traversal, filtering, or ordering lives in the
caller — whether that's a GUI widget, a CLI, or a test harness. The
crate doesn't draw trees, watch filesystems, cache results, interpret
file contents, or decide what "open" means in a given UI.

**Two entry points, matched to two use shapes.**

- `Swdir::walk` is recursive, parallel, error-collecting. For batch
  tools that consume the whole tree.
- `scan_dir*` is single-folder, single-threaded, atomic. For GUI
  trees that expand one node at a time.

They share vocabulary — `SortOrder`, `DirEntry` — but their error
shapes differ on purpose. Partial failure is useful when you're
scanning everything; it's noise when you're showing one folder.

**Defaults that match the common case.**
`Swdir::new()` installs `FilterRule::SkipHidden` and sorts
directories-first-then-files-alphabetically. If you want something
else, say so explicitly. If the defaults annoy you once, the fix is
short (`clear_filters`, `sort_order`); if they helped you a hundred
times, the savings compound.

**Errors are first-class, not stderr fire-and-forget.**
`walk` returns them in a struct; `scan_dir` returns them as `Err`. The
caller decides. The library never panics on filesystem input and
never writes to stderr.

**Behavior that's reproducible by construction.**
`SortOrder::NameAscDirsFirst` is a pure sort over cached data, so
repeating a walk on an unchanged tree returns a byte-identical result.
GUI trees need this to not flicker on re-expand.

## Non-goals

These are deliberately out of scope. Each has been considered and
rejected; each has a principled reason for staying out.

**Async / await.** The sync API is composable with any runtime via
`std::thread::spawn`, `spawn_blocking`, or iced's `Task::perform`.
Adding an `async` feature would either lock users into a specific
runtime or duplicate the API with `#[cfg]` walls. The cost isn't
worth it for a crate whose hot path is a handful of `readdir`s.

**File watching.** `notify`, `inotify`, and friends already do this
well. swdir doesn't try to be a watch-on-change library.

**Caching.** Caches belong next to the data they protect — in the
widget, in the tool, in the application layer. Putting one inside
swdir would expose cache-invalidation semantics the crate can't
reasonably know.

**Symlink following.** A single policy can't satisfy everybody, and
the cycle-detection required to be safe is a surprising amount of
code. `DirEntry::is_symlink()` is the escape hatch; walk callers
implement their own policy with `FilterRule::OnlyKinds` and their own
cycle logic if they really need it.

**Advanced filters (regex, glob, closures, metadata).** The
`FilterRule` enum stays `#[non_exhaustive]` so they can be added
later if real demand appears, but the 0.11 surface is deliberately
small. A glob-backed filter would pull `globset`, a regex filter
would pull `regex`, a closure filter would leak lifetime concerns
into the public API. None of these are paid for by the majority of
callers.

**A GUI-tree data model.** `DirNode` and `DirEntry` stop at "here's
what's in the folder." `Arc<Node>` trees, lazy child placeholders,
selection state, expansion state, keyboard navigation — all widget
concerns.

**Thread-pool knobs beyond `max_threads`.** Rayon's own configuration
is available to callers who need it (install a global pool before
calling `walk`).

## Conventions that hold the line

- **Enums over booleans.** `Recurse`, `SortOrder`, `EntryKind`,
  `FilterRule` are enums rather than flags. Adding a third state later
  is a pattern match, not a breaking change to a `bool` pair.
- **Struct fields get `#[non_exhaustive]` when forward compatibility
  is non-negotiable.** `ScanOptions`, `FilterRule`.
- **Validation up front.** Extension lists reject `".md"` at
  construction time, not silently at match time.
- **No hidden allocations on hot paths.** `DirEntry::display_name()`
  borrows; `DirEntry::is_dir()` uses a cached `FileType`. Tree
  widgets re-render a lot; every allocation avoided is one the GC
  (there isn't one, but Rust equivalent) doesn't have to reclaim.

## Versioning stance

swdir follows SemVer strictly. Within a 0.x line, breaking changes
raise the middle number (0.10 → 0.11). Test-layout or documentation
changes raise the patch number. Once 1.0 ships, this policy moves to
the major.minor.patch triplet.

The [migration notes](migration/) explain every breaking change
along with an upgrade path.
