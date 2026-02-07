mod core;

pub use crate::core::{Recurse, Swdir, dir_node::DirNode};

#[cfg(test)]
mod tests {
    use crate::{Recurse, Swdir};

    #[test]
    fn it_works() {
        let result = Swdir::default().set_root_path(".").scan();
        assert_eq!(result.path, std::path::Path::new(".").to_path_buf());
    }

    #[test]
    fn print_not_recurse() {
        let result = Swdir::default().scan();
        println!("{:?}", result);
        ()
    }

    #[test]
    fn print_recurse_depth_limit_0() {
        let result = Swdir::default()
            .set_recurse(Recurse {
                is_recurse: true,
                depth_limit: Some(0),
            })
            .scan();
        println!("{:?}", result);
        ()
    }

    #[test]
    fn print_with_allowlist() {
        let result = Swdir::default()
            .set_extension_allowlist(&["md"])
            .unwrap()
            .scan();
        println!("{:?}", result);
        ()
    }

    #[test]
    fn print_with_denylist() {
        let result = Swdir::default()
            .set_extension_denylist(&["md"])
            .unwrap()
            .scan();
        println!("{:?}", result);
        ()
    }

    #[test]
    fn print_recurse_depth_limit_1() {
        let result = Swdir::default()
            .set_recurse(Recurse {
                is_recurse: true,
                depth_limit: Some(1),
            })
            .scan();
        println!("{:?}", result);
        ()
    }

    #[test]
    fn err_duplicate_extension_allowlist() {
        let result = Swdir::default()
            .set_extension_denylist(&["txt"])
            .unwrap()
            .set_extension_allowlist(&["txt"]);
        assert!(result.is_err());
    }

    #[test]
    fn err_duplicate_extension_denylist() {
        let result = Swdir::default()
            .set_extension_allowlist(&["txt"])
            .unwrap()
            .set_extension_denylist(&["txt"]);
        assert!(result.is_err());
    }

    #[test]
    fn err_allowlist_start_with_period() {
        let result = Swdir::default().set_extension_allowlist(&[".txt"]);
        assert!(result.is_err());
    }

    #[test]
    fn err_denylist_start_with_period() {
        let result = Swdir::default().set_extension_denylist(&[".txt"]);
        assert!(result.is_err());
    }
}
