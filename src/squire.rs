use std::fs::metadata;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, thread};

/// Function to convert seconds to human-readable format
///
/// # Arguments
///
/// * `seconds` - The number of seconds to convert
///
/// # Returns
///
/// A `String` containing the human-readable format of the seconds
pub fn convert_seconds(seconds: i64, index: usize) -> String {
    if seconds == 0 {
        return "0 seconds".to_string();
    }

    let mut remaining_seconds = seconds;

    let years = remaining_seconds / (60 * 60 * 24 * 365.25 as i64); // Approximate years with leap years
    remaining_seconds %= 60 * 60 * 24 * 365.25 as i64;

    let months = remaining_seconds / (60 * 60 * 24 * 30.44 as i64); // Approximate months with average days
    remaining_seconds %= 60 * 60 * 24 * 30.44 as i64;

    let days = remaining_seconds / 86_400; // 86,400 seconds in a day
    remaining_seconds %= 86_400;

    let hours = remaining_seconds / 3_600; // 3,600 seconds in an hour
    remaining_seconds %= 3_600;

    let minutes = remaining_seconds / 60; // 60 seconds in a minute
    remaining_seconds %= 60;

    let mut result = Vec::new();

    if years > 0 {
        result.push(format!(
            "{} year{}",
            years,
            if years > 1 { "s" } else { "" }
        ));
    }
    if months > 0 {
        result.push(format!(
            "{} month{}",
            months,
            if months > 1 { "s" } else { "" }
        ));
    }
    if days > 0 {
        result.push(format!("{} day{}", days, if days > 1 { "s" } else { "" }));
    }
    if hours > 0 {
        result.push(format!(
            "{} hour{}",
            hours,
            if hours > 1 { "s" } else { "" }
        ));
    }
    if minutes > 0 && result.len() < 2 {
        result.push(format!(
            "{} minute{}",
            minutes,
            if minutes > 1 { "s" } else { "" }
        ));
    }
    if remaining_seconds > 0 && result.len() < 2 {
        result.push(format!(
            "{} second{}",
            remaining_seconds,
            if remaining_seconds > 1 { "s" } else { "" }
        ));
    }
    if result.len() >= index {
        return result[0..index].join(" and ");
    }
    result.join(" and ")
}

/// Function to get the size of a file or directory
///
/// # Arguments
///
/// * `path` - The path to the file or directory
///
/// # Returns
///
/// A `u64` containing the size of the file or directory
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

/// Returns the default number of worker threads (logical cores)
/// If the number of logical cores cannot be determined, it defaults to 8
///
/// # Returns
///
/// A `usize` containing the number of worker threads
pub fn default_workers() -> usize {
    let logical_cores = thread::available_parallelism();
    match logical_cores {
        Ok(cores) => cores.get(),
        Err(err) => {
            log::error!("{}", err);
            1
        }
    }
}

/// Returns the current epoch time in seconds
///
/// # Returns
///
/// A `u64` containing the current epoch time in seconds
pub fn get_epoch() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

/// Returns the media filter for the database query
///
/// # Returns
///
/// A `String` containing the media filter with the supported file extensions
pub fn media_filter() -> String {
    "WHERE lower(relativePath) LIKE '%.hevc'
       OR lower(relativePath) LIKE '%.h264'
       OR lower(relativePath) LIKE '%.mp4'
       OR lower(relativePath) LIKE '%.m4v'
       OR lower(relativePath) LIKE '%.mov'
       OR lower(relativePath) LIKE '%.avi'
       OR lower(relativePath) LIKE '%.aac'
       OR lower(relativePath) LIKE '%.mp3'
       OR lower(relativePath) LIKE '%.m4a'
       OR lower(relativePath) LIKE '%.alac'
       OR lower(relativePath) LIKE '%.aiff'
       OR lower(relativePath) LIKE '%.wav'
       OR lower(relativePath) LIKE '%.flac'
       OR lower(relativePath) LIKE '%.ac3'
       OR lower(relativePath) LIKE '%.eac3'
       OR lower(relativePath) LIKE '%.heic'
       OR lower(relativePath) LIKE '%.jpg'
       OR lower(relativePath) LIKE '%.jpeg'
       OR lower(relativePath) LIKE '%.png'
       OR lower(relativePath) LIKE '%.gif'
       OR lower(relativePath) LIKE '%.tiff'
       OR lower(relativePath) LIKE '%.bmp'
       OR lower(relativePath) LIKE '%.ico';"
        .to_string()
}

/// Function to get the file type
///
/// # Arguments
///
/// * `relative_path` - The path to the file
///
/// # Returns
///
/// A `String` with the file extension in upperacase
pub fn file_type(relative_path: &String) -> String {
    // I basically want the file extension in upper case
    let file_extension = Path::new(relative_path)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    file_extension.to_string()
}

/// Function to get the size of a file or directory
///
/// # Arguments
///
/// * `relative_path` - The path to the file or directory
///
/// # Returns
///
/// A `String` with human-readable format of the file/directory size
pub fn file_size(relative_path: &String) -> String {
    let file_size = get_size(Path::new(relative_path));
    size_converter(file_size)
}
