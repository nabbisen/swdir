use std::path::PathBuf;

/// directory tree
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DirNode {
    pub path: PathBuf,
    pub sub_dirs: Vec<DirNode>, // subdirectories (recursion)
    pub files: Vec<PathBuf>,    // files
}

impl DirNode {
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path,
            sub_dirs: vec![],
            files: vec![],
        }
    }
}
