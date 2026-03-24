use super::DirNode;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DirNodeCount {
    pub files: usize,
    pub dirs: usize,
}

impl DirNodeCount {
    pub fn new(dir_node: &DirNode) -> Self {
        let mut ret = Self::default();
        count(&mut ret, dir_node);
        ret
    }
}

/// count (files, dirs) recursively
fn count(acc: &mut DirNodeCount, dir_node: &DirNode) {
    acc.files += dir_node.files.len();
    acc.dirs += 1;

    // 子ディレクトリに対して再帰
    for sub_dir in &dir_node.sub_dirs {
        count(acc, sub_dir);
    }
}
