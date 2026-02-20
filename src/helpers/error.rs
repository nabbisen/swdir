use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SwdirError {
    #[error("Invalid item in extension allowlist or denylist: {0}")]
    InvalidExtensionListItem(String),

    #[error("Either extension allowlist or denylist can be set")]
    DuplicateExtensionList,
}
