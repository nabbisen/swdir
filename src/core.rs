/// ```rust
/// use swdir::Swdir;
///
/// let dir_node = Swdir::default().set_root_path("/some/path").scan(); // -> DirNode
/// ```
use std::path::PathBuf;

pub mod dir_node;
mod util;

use dir_node::DirNode;

const MAX_THREADS: usize = 8;

#[derive(Clone)]
pub struct Swdir {
    pub root_path: PathBuf,
    pub max_threads: usize,
    pub recurse: Recurse,
    pub whitelist_exts: Option<Vec<String>>,
    pub blacklist_exts: Option<Vec<String>>,
}

#[derive(Clone, Default)]
pub struct Recurse {
    pub is_recurse: bool,
    pub depth_limit: Option<usize>,
}

impl Swdir {
    pub fn set_root_path<T: Into<PathBuf>>(&mut self, path: T) -> Self {
        self.root_path = path.into();
        self.to_owned()
    }

    pub fn set_recurse(&mut self, recurse: Recurse) -> Self {
        self.recurse = recurse;
        self.to_owned()
    }

    pub fn set_whitelist_exts(&mut self, list: Vec<String>) -> Self {
        self.whitelist_exts = Some(list);
        self.to_owned()
    }

    pub fn set_blacklist_exts(&mut self, list: Vec<String>) -> Self {
        self.blacklist_exts = Some(list);
        self.to_owned()
    }

    pub fn scan(&self) -> DirNode {
        self.scan_parallel()
    }
}

impl Swdir {
    pub fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            max_threads: MAX_THREADS,
            recurse: Recurse::default(),
            whitelist_exts: None,
            blacklist_exts: None,
        }
    }
}
