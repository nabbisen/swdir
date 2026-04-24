# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.2] - 2026-04-24

Documentation restructure. No production-code or API changes.

### Changed
- **README.md slimmed down** from ~270 lines to a 7-section landing
  page (hero / overview / when-to-use / quick start / features /
  design notes / pointers). The long-form material moved into the
  new `docs/` tree so the README reads as an entry point rather than
  a manual.
- **`docs/` added** with a functional, topic-scoped hierarchy:
  - `getting-started.md` — fuller walkthrough after the README.
  - `guides/recursive-walk.md`, `guides/lazy-loading.md` — the two
    use cases, end-to-end.
  - `reference/sort-order.md`, `reference/filter-rules.md`,
    `reference/error-handling.md`, `reference/feature-flags.md` — one
    file per concept catalog.
  - `design-notes.md` — philosophy and non-goals.
  - `migration/from-0-10-to-0-11.md`, `migration/from-0-9-to-0-10.md`.
- **Crate-level docs (`src/lib.rs`) trimmed** to quick-start examples
  plus links into the `docs/` tree. Rust-API users still get a
  self-contained intro on docs.rs; repository users get the long
  form one click away.

### Not changed
- Public API: unchanged.
- Test suite: unchanged.
- `Cargo.toml` still references `README.md` as the crate's readme — the
  crate box on crates.io now displays the slimmer text.

## [0.11.1] - 2026-04-24

Test-suite reorganization. No production-code or API changes.

### Changed
- `tests/` restructured so the two integration-test binaries reflect
  the product surface instead of historical naming:
  - `tests/test.rs` → `tests/walk.rs` (the `Swdir::walk` story)
  - `tests/scan_dir.rs` kept, but split into topic-scoped submodules
    under `tests/scan_dir/`:
    - `listing.rs` — baseline happy-path listings
    - `errors.rs` — the atomic-failure contract
    - `ordering.rs` — `SortOrder` / `scan_dir_with_options`
    - `dir_entry_helpers.rs` — `display_name`, `relative_to`, cached FileType
    - `thread_safety.rs` — `Send + 'static`, panic-freedom
- Shared fixtures (`TmpDir`, `name_set`) extracted to
  `tests/common/mod.rs` and loaded via `#[path = ...] mod common;`
  from the binaries that need them.
- Redundant `walk_still_works_alongside_scan_dir` test dropped —
  `tests/walk.rs` already covers walk, and `tests/scan_dir.rs` already
  covers scan. The combined test added no unique signal.

### Removed
- None from the public API. Only test-file layout moved.

## [0.11.0] - 2026-04-24

0.11 is an **additive** release focused on making `swdir` a clean
supplier for a Directory Tree widget (iced, egui, …). No behavioral
breaks against 0.10; just three small, narrowly-scoped additions.

### Added
- `SortOrder` enum with two variants — `Filesystem` (OS `readdir` order)
  and `NameAscDirsFirst` (dirs first, then files, name-ascending). Used
  by both the high-level walk and the low-level scan.
- `ScanOptions` struct (currently holding only `sort_order`) —
  `#[non_exhaustive]` so future growth is additive. `ScanOptions::new(..)`
  shortcut and `Default` impl yielding `NameAscDirsFirst`.
- `Swdir::sort_order(SortOrder) -> Self` — builder method.
  `Swdir::sort_order_policy() -> SortOrder` — read-back getter.
- `scan_dir_with_options(&Path, &ScanOptions) -> Result<Vec<DirEntry>, ScanError>` —
  sorted sibling of `scan_dir`. Single directory only; sorting is an
  in-memory pass over the cached `FileType` field, so it adds **no
  extra syscalls** over `scan_dir`.
- `DirEntry::display_name(&self) -> &OsStr` — borrowed, no allocation;
  intended for GUI labels.
- `DirEntry::relative_to(&self, root: &Path) -> Option<PathBuf>` —
  pure path arithmetic; returns `None` when `root` isn't a prefix.

### Changed
- Crate top-level docs and `README.md` reframe `scan_dir` as the
  canonical **lazy-loading API** for GUI tree widgets, and spell out
  the split between recursive (`Swdir::walk`) and one-directory
  (`scan_dir*`) entry points.
- `Swdir::new()` now records a `SortOrder` (default:
  `NameAscDirsFirst`). Output of `walk()` is unchanged from 0.10 at
  this default — dirs and files were already sorted per level.
- Internal: the unconditional `.sort()` in `core::scan::scan_node` is
  now gated on `SortOrder`; `Filesystem` skips the sort entirely.
  Rayon's ordered `collect` preserves input order, so no extra work is
  needed to keep OS order through the parallel descent.

### Not added (explicit non-goals)
- No regex / glob / arbitrary-closure filters.
- No async / await API — `scan_dir*` stays sync; wrap with
  `std::thread::spawn` if off-thread execution is needed.
- No file-content analysis, URL-routing semantics, symlink-resolution
  policies.
- No GUI-tree data model (that's the widget's responsibility).
- No file watcher, no result cache, no parallelism knobs beyond
  `max_threads`.

### Migration
0.10 code keeps working. Optional adjustments:

```rust
// Take advantage of the new helpers:
let entries = scan_dir_with_options(
    path,
    &ScanOptions::new(SortOrder::NameAscDirsFirst),
)?;
for e in &entries {
    let label: &OsStr = e.display_name();        // was: e.file_name() (owned)
    let rel   = e.relative_to(root);             // was: e.path().strip_prefix(root).ok()
}

// Opt out of the default sort in walk():
let report = Swdir::new()
    .root_path("/x")
    .sort_order(SortOrder::Filesystem)
    .walk();
```

## [0.10.0] - 2026-04-23

0.10 is a cleanup release. The API of `Swdir` is reshaped around a unified
filter model and an explicit error-reporting contract. `scan_dir` is
unchanged.

### Added
- `FilterRule` enum unifying every filter condition in one type: `SkipHidden`,
  `OnlyKinds`, `ExtensionAllowlist`, `ExtensionDenylist`, `UnderPath`,
  `NotUnderPath`, `MaxDepth`. Marked `#[non_exhaustive]` so future rules
  are additive.
- `Decision { include, descend }` makes the two axes of a filter decision
  explicit. Rules compose with AND via `Decision::and`. Constants
  `Decision::PASS`, `Decision::DROP`, `Decision::HIDDEN_PASSTHROUGH` cover
  the common shapes.
- `EntryKind` (File / Dir / Symlink) as a pattern-matchable classifier.
- `FilterContext` carrying path, kind, depth and hidden flag into rule evaluation.
- `WalkReport { tree, errors }` — the new return type of `Swdir::walk`.
- `WalkError` — structured, path-carrying error type for runtime I/O
  failures during `walk`.
- `Swdir::walk_tree()` — convenience for callers who don't care about errors.
- `Swdir::filter(..)`, `.filters(..)`, `.clear_filters()`, `.filter_rules()` —
  the new filter-management surface.
- `Recurse::none()` / `::unlimited()` / `::depth(n)` constructors and
  `Recurse::is_enabled()` / `::depth_limit()` inspectors.
- Default `feature = "filter"` on by default. Turning it off strips the
  filter types and gives a bare walker.

### Changed (breaking)
- `Swdir::default().set_*` chain replaced with a by-value builder:
  `Swdir::new().root_path(p).recurse(..).filter(..).walk()`. The `set_`
  prefix is gone on every method.
- `Swdir::walk` now returns `WalkReport` instead of `DirNode`. Use
  `report.tree` for the old shape, or `.walk_tree()` to discard errors.
- `Recurse` is now an enum (`None` / `Unlimited` / `Depth(n)`) instead of a
  two-field struct. This makes the impossible combination
  `enabled = false, depth_limit = Some(..)` unrepresentable.
- `include_hidden()`, `set_extension_allowlist()`, `set_extension_denylist()`
  are removed. Express the same intents with `FilterRule`:
  - `include_hidden()`                         → `.clear_filters()`
  - `set_extension_allowlist(&["md"])?`        → `.filter(FilterRule::extension_allowlist(["md"])?)`
  - `set_extension_denylist(&["md"])?`         → `.filter(FilterRule::extension_denylist(["md"])?)`
- `SwdirError::DuplicateExtensionList` removed. Allowlist and denylist are
  just stackable rules now; asking for both is a supported composition.
- `Swdir::walk` no longer writes to stderr on I/O failure. Permission-denied
  and similar errors surface through `WalkReport::errors`. Callers who want
  the old behavior can `for err in &report.errors { eprintln!("{err}"); }`.

### Not added (explicit non-goals for 0.10)
- No `advanced-filter` feature. Regex, glob, metadata predicates, and
  arbitrary closure rules are out of scope for this release. The internal
  design leaves room to add them later.
- No implicit defaults beyond `FilterRule::SkipHidden`. Every other filter
  is opt-in.

### Migration
The crate docs (`src/lib.rs`) and `README.md` both carry a 0.9→0.10 mapping
table. Minimal upgrade example:

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

## [0.9.0] - 2026-04-23
### Added
- Support for gui / lazy loading api - `scan_dir()`.
