use std::path::Path;

use crate::dir_node::DirNode;

pub mod dir_node;

pub fn scan() -> DirNode {
    let max_threads: usize = 8; // 環境に合わせて調整
    let it = dir_node::DirNode::new(Path::new(".").as_ref()).build_tree_parallel(max_threads);
    it

    // // 並列に処理したいときは `for_each` などの ParallelIterator メソッドを使う
    // it.clone().for_each(|p| {
    //     // ここは Rayon のスレッドプール上で実行されます
    //     println!("{}", p.display());
    // });

    // // 同期的にベクタが欲しいときは `collect` すれば OK
    // let all: Vec<_> = it.collect();
    // println!("found {} entries", all.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = scan();
        assert_eq!(result.path, std::path::Path::new(".").to_path_buf());
    }
}
