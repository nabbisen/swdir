#[derive(Clone)]
pub struct Recurse {
    pub enabled: bool,
    pub skip_hidden: bool,
    pub depth_limit: Option<usize>,
}

impl Default for Recurse {
    fn default() -> Self {
        Self {
            enabled: false,
            skip_hidden: true,
            depth_limit: None,
        }
    }
}
