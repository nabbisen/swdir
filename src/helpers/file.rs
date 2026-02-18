/// check if file is hidden
pub fn is_hidden(entry: &std::fs::DirEntry) -> bool {
    let start_with_dot = entry.file_name().to_string_lossy().starts_with('.');

    if start_with_dot {
        return true;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = entry.metadata() {
            if metadata.file_attributes() & 0x2 != 0 {
                return true;
            }
        }
    }

    false
}
