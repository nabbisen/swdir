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
///     let dir_node = Swdir::default().set_root_path("/some/path").scan();
///     // -> DirNode (files and subdirectories)
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
///         .disable_skip_hidden() // disable skip hidden files and directories
///         .scan();
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
///         .scan();
///
///     let dir_node_with_denylist = Swdir::default()
///         .set_root_path("/some/path")
///         .set_extension_denylist(&["md"])
///         .unwrap()
///         .scan();
/// }
/// ```
mod core;
mod helpers;

pub use crate::{
    core::Swdir,
    helpers::{dir_node::DirNode, recurse::Recurse},
};
