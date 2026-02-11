# swdir

![Rust](https://img.shields.io/badge/Rust-%23CE412B?style=flat&logo=rust&logoColor=white)
[![crates.io](https://img.shields.io/crates/v/swdir?label=latest)](https://crates.io/crates/swdir)
[![Rust Documentation](https://docs.rs/swdir/badge.svg?version=latest)](https://docs.rs/swdir)
[![Dependency Status](https://deps.rs/crate/swdir/latest/status.svg)](https://deps.rs/crate/swdir)
[![License](https://img.shields.io/github/license/nabbisen/swdir)](https://github.com/nabbisen/swdir/blob/main/LICENSE)

Swiftly traverse and scan directories recursively.
Sway 💃, swing 🎷 or swim 🪼 in directories.

## Quick start

```sh
cargo install swdir
```

```rust
use swdir::Swdir;

let dir_node = Swdir::default().set_root_path("/some/path").scan();
// -> DirNode (files and subdirectories)
```

### Recurse option

```rust
use swdir::{Recurse, Swdir};

let recurse = Recurse {
    enabled: true,
    skip_hidden: true,    // skip hidden files and directories
    depth_limit: Some(1), // only first level subdirectory is scanned
};

let dir_node = Swdir::default()
    .set_root_path("/some/path")
    .set_recurse(recurse)
    .scan();
```

### Allowlist and denylist

```rust
use swdir::Swdir;

let dir_node_with_allowlist = Swdir::default()
    .set_root_path("/some/path")
    .set_extension_allowlist(&["md"])
    .unwrap()
    .scan();

let dir_node_with_denylist = Swdir::default()
    .set_root_path("/some/path")
    .set_extension_denylist(&["md"])
    .unwrap()
    .scan();
```
