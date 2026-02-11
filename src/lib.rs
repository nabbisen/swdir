/// ## Quick start
///
/// ```sh
/// cargo install swdir
/// ```
///
/// ```rust
/// use swdir::Swdir;
///
/// let dir_node = Swdir::default().set_root_path("/some/path").scan();
/// // -> DirNode (files and subdirectories)
/// ```
///
/// ### Recurse option
///
/// ```rust
/// use swdir::{Recurse, Swdir};
///
/// let recurse = Recurse {
///     enabled: true,
///     skip_hidden: true,    // skip hidden files and directories
///     depth_limit: Some(1), // only first level subdirectory is scanned
/// };
///
/// let dir_node = Swdir::default()
///     .set_root_path("/some/path")
///     .set_recurse(recurse)
///     .scan();
/// ```
///
/// ### Allowlist and denylist
///
/// ```rust
/// use swdir::Swdir;
///
/// let dir_node_with_allowlist = Swdir::default()
///     .set_root_path("/some/path")
///     .set_extension_allowlist(&["md"])
///     .unwrap()
///     .scan();
///
/// let dir_node_with_denylist = Swdir::default()
///     .set_root_path("/some/path")
///     .set_extension_denylist(&["md"])
///     .unwrap()
///     .scan();
/// ```
mod core;
mod helpers;

pub use crate::{
    core::Swdir,
    helpers::{dir_node::DirNode, recurse::Recurse},
};
