use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, thread};

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
/// If the number of logical cores cannot be determined, it defaults to 1
///
/// # Returns
///
/// A `usize` containing the number of worker threads
pub fn default_workers() -> usize {
    let logical_cores = thread::available_parallelism();
    match logical_cores {
        Ok(cores) => {
            let workers = cores.get();
            if workers > 1 {
                workers / 2
            } else {
                workers
            }
        }
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
/// * `filename` - Name of the file
///
/// # Returns
///
/// A `String` with the file extension in uppercase
pub fn file_type(relative_path: &PathBuf, filename: &String) -> PathBuf {
    // I basically want the file extension in upper case
    let file_extension = &relative_path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    if file_extension.is_empty() {
        return relative_path.to_owned();
    }
    PathBuf::from(&file_extension).join(filename).to_owned()
}

/// Classifies size into less than 1 MB, 10 MB, 100 MB, 1 GB, and the rest as unclassified.
///
/// # Arguments
///
/// * `byte_size` - Takes bytes as a usize object
///
/// # Returns
///
/// A `String` with the classified category.
fn classify_size(byte_size: usize) -> String {
    if byte_size < 1_000_000 {
        "Less than 1 MB".to_string()
    } else if byte_size <= 10_000_000 {
        "1 MB - 10 MB".to_string()
    } else if byte_size <= 100_000_000 {
        "10 MB - 100 MB".to_string()
    } else if byte_size <= 1_000_000_000 {
        "100 MB - 1 GB".to_string()
    } else if byte_size <= 10_000_000_000 {
        "1 GB - 10 GB".to_string()
    } else {
        "More than 100 GB".to_string()
    }
}

/// Creates a file path categorized by its size
///
/// # Arguments
///
/// * `src_path` - The path to the file or directory
/// * `filename` - Name of the file
///
/// # Returns
///
/// A `String` with human-readable format of the file/directory size
pub fn file_size(src_path: &Path, filename: &String) -> PathBuf {
    PathBuf::from(&classify_size(get_size(src_path) as usize))
        .join(filename)
        .to_owned()
}

/// Loads an environment variable by looking for both upper/lower case of the key
///
/// # Arguments
///
/// * `key` - Takes the key for env var as an argument
///
/// # Returns
///
/// A `String` with the response value from the environment variable
pub fn env_var(env_key: &'static str, alias: Option<Vec<&'static str>>) -> Option<String> {
    let mut alias_list = match alias {
        Some(v) => v,
        None => vec![env_key],
    };
    if !alias_list.contains(&env_key) {
        alias_list.push(env_key);
    }
    for key in alias_list {
        return match env::var(key.to_uppercase()) {
            Ok(val) => Some(val),
            Err(_) => match env::var(key.to_lowercase()) {
                Ok(val) => Some(val),
                Err(_) => continue,
            },
        };
    }
    None
}
