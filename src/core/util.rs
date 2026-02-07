use std::fs;
use std::path::Path;

use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::Swdir;
use super::dir_node::DirNode;

impl Swdir {
    /// scan to build tree with concurrency of specified process number
    pub(super) fn scan_parallel(&self) -> DirNode {
        // thread pool
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.max_threads)
            .build()
            .expect("failed to build rayon thread pool");

        // scan in the pool
        pool.install(|| {
            self.scan_node(
                self.root_path.as_path().as_ref(),
                self.recurse.is_recurse,
                self.recurse.depth_limit,
            )
        })
    }

    /// scan node to build subtree
    fn scan_node(&self, dir_path: &Path, is_recurse: bool, depth_limit: Option<usize>) -> DirNode {
        let mut files = Vec::new();
        // scan sud directories only when resurse is true and depth_limit has capacity
        let mut sub_dir_paths = if is_recurse && depth_limit.is_some_and(|x| 1 <= x) {
            Some(Vec::new())
        } else {
            None
        };

        // 1. 現在のディレクトリを読み込む (I/O)
        // エラー時は空の結果として扱い、ログだけ出す実装にしています
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();

                if path.is_dir() {
                    if let Some(sub_dir_paths) = sub_dir_paths.as_mut() {
                        sub_dir_paths.push(path);
                    }
                    continue;
                }

                let should_push = if let Some(extension_allowlist) = &self.extension_allowlist {
                    if let Some(extension) = path.extension() {
                        let extension = extension.to_string_lossy();
                        extension_allowlist.iter().any(|x| **x == extension)
                    } else {
                        false
                    }
                } else if let Some(extension_denylist) = &self.extension_denylist {
                    if let Some(extension) = path.extension() {
                        let extension = extension.to_string_lossy();
                        extension_denylist.iter().all(|x| **x != extension)
                    } else {
                        true
                    }
                } else {
                    true
                };

                if should_push {
                    files.push(path);
                }
            }
        } else {
            // todo: error handling
            eprintln!("⚠️ Access denied or error: {}", dir_path.display());
        }

        // 2. サブディレクトリを並列処理で再帰的にスキャン
        // `par_iter` を使うことで、ブランチごとに別スレッドで処理が走ります
        let mut sub_dirs = if let Some(sub_dir_paths) = &sub_dir_paths {
            sub_dir_paths
                .into_par_iter()
                .map(|path| {
                    let depth_limit = if let Some(depth_limit) = depth_limit {
                        Some(depth_limit - 1)
                    } else {
                        None
                    };
                    // recursion
                    self.scan_node(&path, is_recurse, depth_limit)
                })
                .collect()
        } else {
            Vec::new()
        };

        sub_dirs.sort();
        files.sort();
        // return
        DirNode {
            path: dir_path.to_path_buf(),
            sub_dirs,
            files,
        }
    }
}
