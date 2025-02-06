use std::fs;
use std::fs::metadata;
use std::path::Path;

pub fn get_size(path: &Path) -> u64 {
    if path.is_file() {
        metadata(path).map(|meta| meta.len()).unwrap_or(0)
    } else if path.is_dir() {
        fs::read_dir(path)
            .unwrap()
            .flatten()
            .map(|entry| get_size(&entry.path()))
            .sum()
    } else {
        0
    }
}

/// Function to convert byte size to human-readable format
///
/// # Arguments
///
/// * `byte_size` - The size in bytes to convert
///
/// # Returns
///
/// A `String` containing the human-readable format of the byte size
pub fn size_converter(byte_size: u64) -> String {
    let size_name = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut index = 0;
    let mut size = byte_size as f64;

    while size >= 1024.0 && index < size_name.len() - 1 {
        size /= 1024.0;
        index += 1;
    }

    format!("{:.2} {}", size, size_name[index])
}
