use crate::constant;
use crate::parser;
use plist::Value;
use rusqlite::{Connection, Result};
use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use tqdm::{pbar, Pbar};

/// Function to retrieve the value of a key from a plist file
///
/// # Arguments
///
/// * `info` - The plist file as a `Value`
/// * `key` - The key to retrieve the value for
/// * `default` - The default value to return if the key is not found
///
/// # Returns
///
/// A `String` containing the value of the key
pub fn get_plist_key(info: &Option<Value>, key: &str, default: &str) -> String {
    match info.as_ref() {
        Some(val) => match val.as_dictionary() {
            Some(dict) => match dict.get(key) {
                Some(value) => match value.as_string() {
                    Some(str) => str.to_string(),
                    None => default.to_string(),
                },
                None => default.to_string(),
            },
            None => default.to_string(),
        },
        None => default.to_string(),
    }
}

/// Function to parse the manifest database
///
/// # Arguments
///
/// * `manifest_db_path` - The path to the manifest database
/// * `backup` - The backup information
/// * `arguments` - The command line arguments
///
/// # Returns
///
/// * `Ok` - If the function completes successfully
/// * `Err` - If the function encounters an error
pub fn parse_manifest_db(
    manifest_db_path: &Path,
    backup: &constant::Backup,
    arguments: &parser::ArgConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(manifest_db_path)?;

    // Get count to update progress bar
    let mut count_stmt = conn.prepare(
        "SELECT COUNT(*) FROM Files WHERE relativePath LIKE '%DCIM/%' OR relativePath LIKE '%PhotoData/%'"
    )?;
    let count: usize = count_stmt.query_row([], |row| row.get(0))?;
    let progress_bar_base: Arc<Mutex<Pbar>> = Arc::new(Mutex::new(pbar(Some(count))));

    let mut stmt = conn.prepare("SELECT fileID, relativePath FROM Files WHERE relativePath LIKE '%DCIM/%' OR relativePath LIKE '%PhotoData/%'")?;
    let rows = stmt.query_map([], |row| {
        let file_id: String = row.get(0)?;
        let relative_path: String = row.get(1)?;
        Ok((file_id, relative_path))
    })?;

    // Create a thread pool with a fixed number of threads
    let pool = ThreadPool::new(arguments.workers);
    let (sender, receiver) = channel();

    for file in rows {
        match file {
            Ok((file_id, relative_path)) => {
                let backup_cloned = backup.path.clone();
                let output_dir_cloned = arguments.output_dir.clone();
                let sender_cloned = sender.clone();
                let progress_bar = Arc::clone(&progress_bar_base);
                pool.execute(move || {
                    let result =
                        extract_files(&backup_cloned, &output_dir_cloned, file_id, relative_path);
                    sender_cloned.send(result).expect("Failed to send result");
                    // Safely update progress bar
                    let mut progress = progress_bar.lock().unwrap();
                    progress.update(1).unwrap();
                });
            }
            Err(err) => {
                log::error!("Failed to submit thread operation: {}", err);
            }
        }
    }
    // Wait for all tasks to complete
    drop(sender); // Close the sending side of the channel
    pool.join();
    for result in receiver {
        if let Err(err) = result {
            log::error!("Error processing files: {:?}", err);
        }
    }
    Ok(())
}

/// Function to extract files from the backup
///
/// # Arguments
///
/// * `backup_path` - The path to the backup directory
/// * `output_path` - The path to the output directory
/// * `file_id` - The file ID
/// * `relative_path` - The relative path of the file
///
/// # Returns
///
/// * `Ok` - If the function completes successfully
/// * `Err` - If the function encounters an error
fn extract_files(
    backup_path: &Path,
    output_path: &Path,
    file_id: String,
    relative_path: String,
) -> std::io::Result<()> {
    let src_path = backup_path.join(&file_id[..2]).join(file_id);
    let dest_path = output_path.join(relative_path);
    if let Some(parent) = dest_path.parent() {
        match create_dir_all(parent) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }
    }
    if src_path.exists() {
        let mut src_file = File::open(&src_path)?;
        let mut dest_file = File::create(&dest_path)?;
        match copy(&mut src_file, &mut dest_file) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }
        log::debug!(
            "Extracted: {} -> {}",
            src_path.display(),
            dest_path.display()
        );
    }
    Ok(())
}
