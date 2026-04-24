# Reference: feature flags

swdir has exactly one feature flag.

| Feature  | Default | What it gates                                                                     |
|----------|---------|-----------------------------------------------------------------------------------|
| `filter` | ✅ on   | `FilterRule`, `Decision`, `EntryKind`, `FilterContext`, and all `Swdir` filter methods. |

## `filter` (default)

With `filter` on — the default — `Swdir::new()` installs
`FilterRule::SkipHidden` out of the box, and you get `.filter()`,
`.filters()`, `.clear_filters()`, and `.filter_rules()` on the builder.
See the [filter-rules reference](filter-rules.md) for the full model.

## `filter` disabled

```toml
[dependencies]
swdir = { version = "0.11", default-features = false }
```

Turning `filter` off strips the filter model entirely. You get a
minimal walker:

- `Swdir::new()` has no default rules. **Hidden entries are no longer
  skipped.**
- `.filter*` / `.clear_filters()` / `.filter_rules()` are not compiled.
- `FilterRule`, `Decision`, `EntryKind`, `FilterContext` are not exported.
- `scan_dir`, `scan_dir_with_options`, `Recurse`, `SortOrder`,
  `WalkReport`, `WalkError`, `DirEntry`, and the `Swdir` walker
  itself all still work.

Use this when you really do want an unfiltered walker and don't want
any of the filter code in your binary.

## What there **isn't**

No `advanced-filter`, no `async`, no `watcher`, no `glob`, no `regex`.
See [design-notes.md](../design-notes.md) for the rationale.

If a compelling real-world use case shows up for one of them, it'll be
added behind its own feature flag in a minor release — never enabled
by default.
