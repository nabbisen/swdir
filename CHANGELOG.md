# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Migration

See [MIGRATION.md#migrating-from-09](./MIGRATION.md#migrating-from-09).

### Not added (explicit non-goals for 0.10)
- No `advanced-filter` feature. Regex, glob, metadata predicates, and
  arbitrary closure rules are out of scope for this release. The internal
  design leaves room to add them later.
- No implicit defaults beyond `FilterRule::SkipHidden`. Every other filter
  is opt-in.

## [0.9.0] - 2026-04-23
### Added
- Support for gui / lazy loading api - `scan_dir()`.
