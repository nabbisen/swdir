pub fn validate_list_extensions(
    list: &Vec<String>,
    reference: Option<&Vec<String>>,
) -> Result<(), String> {
    for x in list {
        if x.starts_with(".") {
            return Err(format!("Should not start with \".\": {}", x));
        }
    }
    if reference.is_some() {
        return Err("Cannot specify both allowlist and denylist. Please choose one".to_owned());
    }
    Ok(())
}
