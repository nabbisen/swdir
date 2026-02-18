/// ## Quick start
///
/// ```sh
/// cargo install swdir
/// ```
///
/// ```rust
/// use swdir::Swdir;
///
/// fn run() {
///     let dir_node = Swdir::default().set_root_path("/some/path").walk();
///     // -> DirNode (files and subdirectories)
///     //     -> flatten_paths() returns Vec<PathBuf>
/// }
/// ```
///
/// ### Recurse option
///
/// ```rust
/// use swdir::{Recurse, Swdir};
///
/// fn run() {
///     let recurse = Recurse {
///         enabled: true,
///         depth_limit: Some(1), // only first level subdirectory is scanned
///     };
///
///     let dir_node = Swdir::default()
///         .set_root_path("/some/path")
///         .set_recurse(recurse)
///         .include_hidden() // don't skip hidden files and directories
///         .walk();
/// }
/// ```
///
/// ### Allowlist and denylist
///
/// ```rust
/// use swdir::Swdir;
///
/// fn run() {
///     let dir_node_with_allowlist = Swdir::default()
///         .set_root_path("/some/path")
///         .set_extension_allowlist(&["md"])
///         .unwrap()
///         .walk();
///
///     let dir_node_with_denylist = Swdir::default()
///         .set_root_path("/some/path")
///         .set_extension_denylist(&["md"])
///         .unwrap()
///         .walk();
/// }
/// ```
mod core;
mod helpers;

pub use crate::{
    core::Swdir,
    helpers::{dir_node::DirNode, recurse::Recurse},
};
