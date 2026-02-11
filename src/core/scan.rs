use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};

use std::fs;
use std::path::Path;

use crate::helpers::file::is_hidden;

use super::DirNode;
use super::Swdir;

impl Swdir {
    /// scan to build tree with concurrency of specified process number
    pub(super) fn scan_parallel(&self) -> DirNode {
        // thread pool
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.max_threads)
            .build()
            .expect("failed to build rayon thread pool");

        // scan in the pool
        pool.install(|| self.scan_node(self.root_path.as_path().as_ref(), self.recurse.depth_limit))
    }

    /// scan node to build subtree
    fn scan_node(&self, dir_path: &Path, depth_limit: Option<usize>) -> DirNode {
        let mut files = Vec::new();
        // condition to scan sud directories: only when recurse.enabled is true and depth_limit has capacity
        let scan_sub_dir_enabled =
            self.recurse.enabled && (depth_limit.is_none() || depth_limit.is_some_and(|x| 1 <= x));
        let mut sub_dir_paths = if scan_sub_dir_enabled {
            Some(Vec::new())
        } else {
            None
        };

        // scan current directory
        let entries = match fs::read_dir(dir_path) {
            Ok(x) => x,
            Err(err) => {
                // todo: error handling
                eprintln!(
                    "⚠️ Access denied or error: {} ({})",
                    dir_path.display(),
                    err,
                );

                return DirNode {
                    path: dir_path.to_path_buf(),
                    sub_dirs: vec![],
                    files: vec![],
                };
            }
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            if self.recurse.skip_hidden && is_hidden(&entry) {
                continue;
            }

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
                    !(self.recurse.skip_hidden && is_hidden(&entry))
                }
            } else {
                true
            };

            if should_push {
                files.push(path);
            }
        }

        // scan subdirectories recursively
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
                    self.scan_node(&path, depth_limit)
                })
                .collect()
        } else {
            Vec::new()
        };

        sub_dirs.sort();
        files.sort();

        DirNode {
            path: dir_path.to_path_buf(),
            sub_dirs,
            files,
        }
    }
}
