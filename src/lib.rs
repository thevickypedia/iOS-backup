mod backup;
mod constant;
mod parser;
mod squire;

use plist::Value;
use rusqlite::{Connection, Result};
use std::fs::{create_dir_all, read_dir, File};
use std::io::copy;
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::backup::Backup;

fn list_backups(backups: Vec<backup::Backup>) {
    let mut max_serial = "Serial Number".len();
    let mut max_device = "Device".len();
    let mut max_product = "Product Name".len();
    let mut max_date = "Backup Date".len();
    let mut max_encrypted = "Encrypted".len();
    let mut max_size = "Size".len();
    let mut backup_info = Vec::new();

    for backup in backups {
        // Update max lengths dynamically
        max_serial = max_serial.max(backup.serial_number.len());
        max_device = max_device.max(backup.device_name.len());
        max_product = max_product.max(backup.product_name.len());
        max_date = max_date.max(backup.backup_date.len());
        max_encrypted = max_encrypted.max(backup.encrypted.len());
        max_size = max_size.max(backup.backup_size.len());

        backup_info.push((
            backup.serial_number,
            backup.device_name,
            backup.product_name,
            backup.backup_date,
            backup.encrypted,
            backup.backup_size,
        ));
    }

    let table_width = max_serial + max_device + max_date + max_encrypted + 3 * 3; // 3 spaces between columns
    let title = "Available iOS Device Backups";
    println!("\n{0:^1$}", title, table_width);

    println!(
        "{:-<width_serial$} {:-<width_device$} {:-<width_product$} {:-<width_date$} {:-<width_enc$} {:-<width_size$}",
        "",
        "",
        "",
        "",
        "",
        "",
        width_serial = max_serial,
        width_device = max_device,
        width_product = max_product,
        width_date = max_date,
        width_enc = max_encrypted,
        width_size = max_size
    );

    println!(
        "{:<width_serial$} {:<width_device$} {:<width_product$} {:<width_date$} {:<width_enc$} {:<width_size$}",
        "Serial Number",
        "Device",
        "Product",
        "Backup Date",
        "Encrypted",
        "Size",
        width_serial = max_serial,
        width_device = max_device,
        width_product = max_product,
        width_date = max_date,
        width_enc = max_encrypted,
        width_size = max_size
    );

    println!(
        "{:-<width_serial$} {:-<width_device$} {:-<width_product$} {:-<width_date$} {:-<width_enc$} {:-<width_size$}",
        "",
        "",
        "",
        "",
        "",
        "",
        width_serial = max_serial,
        width_device = max_device,
        width_product = max_product,
        width_date = max_date,
        width_enc = max_encrypted,
        width_size = max_size
    );

    for (serial_number, device_name, product_name, backup_date, encrypted, backup_size) in
        &backup_info
    {
        println!(
            "{:<width_serial$} {:<width_device$} {:<width_product$} {:<width_date$} {:<width_enc$} {:<width_size$}",
            serial_number,
            device_name,
            product_name,
            backup_date,
            encrypted,
            backup_size,
            width_serial = max_serial,
            width_device = max_device,
            width_product = max_product,
            width_date = max_date,
            width_enc = max_encrypted,
            width_size = max_size
        );
    }
}

fn get_plist_key(info: &Option<Value>, key: &str, default: &str) -> String {
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

fn get_backups(backup_root: &Path, serial_filter: &str, list: bool) -> Vec<backup::Backup> {
    let mut backups = Vec::new();
    if let Ok(entries) = read_dir(backup_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let info_plist = path.join("Info.plist");
                if info_plist.exists() {
                    let info = Value::from_file(&info_plist).ok();
                    let serial_number = get_plist_key(&info, "Serial Number", "NO_SERIAL");
                    let device_name = get_plist_key(&info, "Device Name", "Unknown Device");
                    let product_name = get_plist_key(&info, "Product Name", "Unknown Product");

                    let seconds = info
                        .as_ref()
                        .and_then(|v| v.as_dictionary()?.get("Last Backup Date"))
                        .and_then(Value::as_date)
                        .map_or(0, |date| {
                            let system_time: SystemTime = date.into();
                            let duration_since_epoch = system_time
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or(Duration::new(0, 0));
                            duration_since_epoch.as_secs()
                        });
                    let backup_date = format!(
                        "{} ago",
                        squire::convert_seconds((get_epoch() - seconds) as i64, 2)
                    );

                    let encrypted = info
                        .as_ref()
                        .and_then(|v| match v.as_dictionary() {
                            Some(dict) => dict.get("IsEncrypted"),
                            None => None,
                        })
                        .map_or("No".to_string(), |v| match v.as_boolean() {
                            Some(true) => "Yes".to_string(),
                            _ => "No".to_string(),
                        });
                    let backup_size_raw = squire::get_size(&path);
                    let backup_size = squire::size_converter(backup_size_raw);
                    if list || serial_number == serial_filter {
                        backups.push(backup::Backup {
                            path,
                            serial_number,
                            device_name,
                            product_name,
                            backup_date,
                            backup_size,
                            encrypted,
                        });
                    }
                }
            }
        }
    }
    backups
}

fn parse_manifest_db(manifest_db_path: &Path, backup: &Backup, arguments: &parser::ArgConfig) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(manifest_db_path)?;
    let mut stmt = conn.prepare("SELECT fileID, relativePath FROM Files WHERE relativePath LIKE '%DCIM/%' OR relativePath LIKE '%PhotoData/%'")?;
    let rows = stmt.query_map([], |row| {
        let file_id: String = row.get(0)?;
        let relative_path: String = row.get(1)?;
        Ok((file_id, relative_path))
    })?;

    let mut threads = Vec::new();
    let mut thread_spawned = 0;

    for file in rows {
        match file {
            Ok((file_id, relative_path)) => {
                thread_spawned += 1;
                let backup_cloned = backup.path.clone();
                let output_dir_cloned = arguments.output_dir.clone();
                let thread = thread::spawn(move || {
                    extract_files(&backup_cloned, &output_dir_cloned, file_id, relative_path)
                        .expect("Failed to extract files");
                });
                threads.push(thread);
            }
            Err(err) => {
                println!("Failed to submit thread operation: {}", err);
            }
        }
    }

    let mut thread_joined = 0;
    // Ensure all threads are joined before proceeding
    for thread in threads {
        if let Err(err) = thread.join() {
            println!("Error joining thread: {:?}", err);
        } else {
            thread_joined += 1
        }
    }
    println!("Threads Spawned: {}", thread_spawned);
    println!("Threads Joined: {}", thread_joined);
    Ok(())
}

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
        println!(
            "Extracted: {} -> {}",
            src_path.display(),
            dest_path.display()
        );
    }
    Ok(())
}

fn get_epoch() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub fn retriever() -> Result<String, String> {
    let metadata = constant::build_info();
    let arguments = parser::arguments(&metadata);
    if arguments.serial_number.is_empty() && !arguments.list {
        return Err(
            "Please provide the serial number (--serial) or use list (--list) option.".into(),
        );
    }
    let backups = get_backups(
        &arguments.backup_dir,
        &arguments.serial_number,
        arguments.list,
    );
    if backups.is_empty() {
        let err = if arguments.serial_number.is_empty() {
            format!("No backups found in '{}'", arguments.backup_dir.display())
        } else {
            format!(
                "No backups found for serial '{}' in '{}'",
                arguments.serial_number,
                arguments.backup_dir.display()
            )
        };
        return Err(err);
    }
    if arguments.list {
        list_backups(backups);
        return Ok("".into());
    }

    // todo: swap current threads to thread pool and set this to threadpool
    for backup in backups {
        let manifest_db_path = backup.path.join("Manifest.db");
        println!("Manifest: {}", manifest_db_path.display());
        if manifest_db_path.exists() {
            let _ = parse_manifest_db(&manifest_db_path, &backup, &arguments);
        }
    }
    Ok("Success".into())
}
