use crate::helpers::error::SwdirError;

/// validate:
/// - extension items in allowlist / denylist don't start with period
/// - both allowlist / denylist are specified
pub fn validate_list_extensions(
    list: &Vec<String>,
    reference: Option<&Vec<String>>,
) -> Result<(), SwdirError> {
    for x in list {
        if x.starts_with(".") {
            return Err(SwdirError::InvalidExtensionListItem(x.to_owned()));
        }
    }
    if reference.is_some() {
        return Err(SwdirError::DuplicateExtensionList);
    }
    Ok(())
}
