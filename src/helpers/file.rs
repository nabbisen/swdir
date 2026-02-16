#[cfg(not(windows))]
/// check if file is hidden
pub fn is_hidden(entry: &std::fs::DirEntry) -> bool {
    entry.file_name().to_string_lossy().starts_with('.')
}

#[cfg(windows)]
/// check if file is hidden
pub fn is_hidden(entry: &std::fs::DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    if let Ok(metadata) = entry.metadata() {
        return metadata.file_attributes() & 0x2 != 0;
    }
    false
}
