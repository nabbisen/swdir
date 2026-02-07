use std::{fs, path::Path};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::DirNode;

/// 再帰ロジック: 現在のディレクトリを読み、サブディレクトリがあれば並列で再帰する
pub fn scan_node(dir_path: &Path) -> DirNode {
    let mut files = Vec::new();
    let mut sub_dir_paths = Vec::new();

    // 1. 現在のディレクトリを読み込む (I/O)
    // エラー時は空の結果として扱い、ログだけ出す実装にしています
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                sub_dir_paths.push(path);
            } else {
                files.push(path);
            }
        }
    } else {
        eprintln!("⚠️ Access denied or error: {}", dir_path.display());
    }

    // 2. サブディレクトリを並列処理で再帰的にスキャン
    // `par_iter` を使うことで、ブランチごとに別スレッドで処理が走ります
    let sub_dirs = sub_dir_paths
        .into_par_iter()
        .map(|path| scan_node(&path)) // ここで再帰呼び出し
        .collect();

    // 3. 結果を構造体にまとめる
    DirNode {
        path: dir_path.to_path_buf(),
        sub_dirs,
        files,
    }
}
