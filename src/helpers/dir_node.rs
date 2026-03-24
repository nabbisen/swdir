use std::{cmp::Ordering, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// directory tree
pub struct DirNode {
    pub path: PathBuf,
    pub sub_dirs: Vec<DirNode>, // subdirectories (recursion)
    pub files: Vec<PathBuf>,    // files
}

impl DirNode {
    /// new() with specific path
    pub fn with_path<T: Into<PathBuf>>(path: T) -> Self {
        Self {
            path: path.into(),
            sub_dirs: vec![],
            files: vec![],
        }
    }

    /// get flatten path list
    pub fn flatten_paths(&self) -> Vec<PathBuf> {
        let mut ret = self.files.clone();
        ret.extend(
            self.sub_dirs
                .iter()
                .flat_map(|dir_node| dir_node.flatten_paths()),
        );
        // todo: sort alg: lower dir first ?
        ret.sort_by(flatten_paths_sort);
        ret
    }

    /// count (files, directories) of root and all sub_dirs
    pub fn count(&self) -> (usize, usize) {
        count((0, 0), self)
    }
}

/// note: use extension() instead of is_dir() because file system i/o is heavier
fn flatten_paths_sort(a: &PathBuf, b: &PathBuf) -> Ordering {
    // 第一条件: ディレクトリ階層数
    let depth_a = a.components().count();
    let depth_b = b.components().count();

    depth_a
        .cmp(&depth_b)
        .then_with(|| {
            // 第二条件: パスが '/' で終わるかどうかでディレクトリ判定
            // または拡張子がないものをディレクトリとみなす
            let is_likely_dir_a = a.extension().is_none();
            let is_likely_dir_b = b.extension().is_none();

            // ディレクトリを先に（false < true なので逆にする）
            is_likely_dir_b.cmp(&is_likely_dir_a)
        })
        .then_with(|| {
            // 第三条件: ファイル名でソート
            let name_a = a.file_name().unwrap_or_default();
            let name_b = b.file_name().unwrap_or_default();
            name_a.cmp(name_b)
        })
}

/// count (files, dirs) recursively
fn count(current: (usize, usize), dir_node: &DirNode) -> (usize, usize) {
    let mut ret = current;

    ret.0 += dir_node.files.len();
    ret.1 += 1;

    let (sub_dirs_files_count, sub_dirs_dirs_count) = dir_node.sub_dirs.iter().fold((0, 0), count);

    ret.0 += sub_dirs_files_count;
    ret.1 += sub_dirs_dirs_count;

    ret
}
