use std::path::{Path, PathBuf};

use rayon::ThreadPoolBuilder;

use util::scan_node;

mod util;

/// ディレクトリツリーを表す構造体
#[derive(Debug)]
pub struct DirNode {
    pub path: PathBuf,
    pub sub_dirs: Vec<DirNode>, // サブディレクトリ（再帰構造）
    pub files: Vec<PathBuf>,    // その階層にあるファイル
}

impl DirNode {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            sub_dirs: vec![],
            files: vec![],
        }
    }

    /// 指定したスレッド数でディレクトリツリーを並列構築する
    pub fn build_tree_parallel(&self, max_threads: usize) -> DirNode {
        // スレッドプールを作成
        let pool = ThreadPoolBuilder::new()
            .num_threads(max_threads)
            .build()
            .expect("failed to build rayon thread pool");

        // プール内でスキャンを実行
        pool.install(|| scan_node(self.path.as_path().as_ref()))
    }
}
