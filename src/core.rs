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
    root_path: PathBuf,
    max_threads: usize,
    recurse: Recurse,
    extension_allowlist: Option<Vec<String>>,
    extension_denylist: Option<Vec<String>>,
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

    pub fn set_extension_allowlist<T: Into<String> + Clone>(
        &mut self,
        list: &[T],
    ) -> Result<Self, String> {
        let list: Vec<String> = list.to_vec().into_iter().map(|x| x.into()).collect();
        if let Err(err) = validate_list_extensions(&list, self.extension_denylist.as_ref()) {
            return Err(err);
        }
        self.extension_allowlist = Some(list);
        Ok(self.to_owned())
    }

    pub fn set_extension_denylist<T: Into<String> + Clone>(
        &mut self,
        list: &[T],
    ) -> Result<Self, String> {
        let list: Vec<String> = list.to_vec().into_iter().map(|x| x.into()).collect();
        if let Err(err) = validate_list_extensions(&list, self.extension_allowlist.as_ref()) {
            return Err(err);
        }
        self.extension_denylist = Some(list);
        Ok(self.to_owned())
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
            extension_allowlist: None,
            extension_denylist: None,
        }
    }
}

fn validate_list_extensions(
    list: &Vec<String>,
    reference: Option<&Vec<String>>,
) -> Result<(), String> {
    for x in list {
        if x.starts_with(".") {
            return Err(format!("Should not start with \".\": {}", x));
        }
    }
    if reference.is_some() {
        return Err("Cannot specify both allowlist and denylist. Please choose one".to_owned());
    }
    Ok(())
}
