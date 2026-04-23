//! Filesystem helpers shared across the crate.

use std::fs::DirEntry;

/// Is `entry` considered hidden by platform conventions?
///
/// * Unix: file name starts with `.`.
/// * Windows: file name starts with `.` *or* the filesystem's hidden
///   attribute bit is set.
pub fn is_hidden(entry: &DirEntry) -> bool {
    if entry.file_name().to_string_lossy().starts_with('.') {
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
