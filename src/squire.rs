use std::{fs, thread};
use std::fs::metadata;
use std::path::Path;



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
pub fn default_workers() -> usize {
    let logical_cores = thread::available_parallelism();
    match logical_cores {
        Ok(cores) => cores.get(),
        Err(err) => {
            log::error!("{}", err);
            8
        }
    }
}
