#[derive(Clone)]
pub struct Recurse {
    pub enabled: bool,
    pub depth_limit: Option<usize>,
}

impl Default for Recurse {
    fn default() -> Self {
        Self {
            enabled: false,
            depth_limit: None,
        }
    }
}
