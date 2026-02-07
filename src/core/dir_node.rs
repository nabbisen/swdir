use std::path::PathBuf;

/// directory tree
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DirNode {
    pub path: PathBuf,
    pub sub_dirs: Vec<DirNode>, // subdirectories (recursion)
    pub files: Vec<PathBuf>,    // files
}
