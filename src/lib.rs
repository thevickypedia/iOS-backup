mod constant;
mod parser;
mod squire;

use crate::constant::build_info;
use crate::parser::arguments;
use plist::Value;
use rusqlite::{Connection, Result};
use std::fs::{create_dir_all, read_dir, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

fn list_backups(backups: Vec<PathBuf>) {
    let mut max_serial = "Serial Number".len();
    let mut max_device = "Device".len();
    let mut max_product = "Product Name".len();
    let mut max_date = "Backup Date".len();
    let mut max_encrypted = "Encrypted".len();
    let mut max_size = "Size".len();
    let mut backup_info = Vec::new();

    for path in backups {
        let info_plist = path.join("Info.plist");
        if info_plist.exists() {
            let info = Value::from_file(&info_plist).ok();
            // todo: currently this uses the directory name (not serial number) - fix it
            // println!("{:?}", &info);
            let serial_number = path.file_name().unwrap().to_string_lossy().to_string();
            let device_name = info
                .as_ref()
                .and_then(|v| v.as_dictionary()?.get("Device Name"))
                .and_then(Value::as_string)
                .unwrap_or("Unknown Device")
                .to_string();

            let product_name = info
                .as_ref()
                .and_then(|v| v.as_dictionary()?.get("Product Name"))
                .and_then(Value::as_string)
                .unwrap_or("Unknown Product")
                .to_string();

            let backup_date = info
                .as_ref()
                .and_then(|v| v.as_dictionary()?.get("Last Backup Date"))
                .map_or("Unknown Date".to_string(), |v| format!("{:?}", v));

            let encrypted = info
                .as_ref()
                .and_then(|v| v.as_dictionary()?.get("IsEncrypted"))
                .map_or("No".to_string(), |v| {
                    if v.as_boolean().unwrap_or(false) {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    }
                });

            let backup_size = squire::get_size(&path);
            let backup_size_str = squire::size_converter(backup_size);

            // Update max lengths dynamically
            max_serial = max_serial.max(serial_number.len());
            max_device = max_device.max(device_name.len());
            max_product = max_product.max(product_name.len());
            max_date = max_date.max(backup_date.len());
            max_encrypted = max_encrypted.max(encrypted.len());
            max_size = max_size.max(backup_size_str.len());

            backup_info.push((
                serial_number,
                device_name,
                product_name,
                backup_date,
                encrypted,
                backup_size_str,
            ));
        }
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

    for (serial_number, device_name, product_name, backup_date, encrypted, backup_size) in &backup_info {
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

fn get_backups(backup_root: &Path, serial_filter: &str, list: bool) -> Vec<PathBuf> {
    let mut backups = Vec::new();
    if let Ok(entries) = read_dir(backup_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let serial_number = path.file_name().unwrap().to_string_lossy().to_string();
                if list || serial_number == serial_filter {
                    backups.push(path);
                }
            }
        }
    }
    backups
}

fn parse_manifest_db(manifest_db_path: &Path) -> Result<Vec<(String, String)>> {
    let conn = Connection::open(manifest_db_path)?;
    let mut stmt = conn.prepare("SELECT fileID, relativePath FROM Files WHERE relativePath LIKE '%DCIM/%' OR relativePath LIKE '%PhotoData/%'")?;
    let rows = stmt.query_map([], |row| {
        let file_id: String = row.get(0)?;
        let relative_path: String = row.get(1)?;
        Ok((file_id, relative_path))
    })?;
    // todo: do a lazy instead of .collect
    //  tried several approaches to return the iterator but borrow checker won't allow
    rows.collect()
}

fn extract_files(
    backup_path: &Path,
    output_path: &Path,
    files: &Vec<(String, String)>,
) -> std::io::Result<()> {
    for (file_id, relative_path) in files {
        let src_path = backup_path.join(&file_id[..2]).join(file_id);
        let dest_path = output_path.join(relative_path);
        if let Some(parent) = dest_path.parent() {
            create_dir_all(parent)?;
        }
        if src_path.exists() {
            let mut src_file = File::open(&src_path)?;
            let mut dest_file = File::create(&dest_path)?;
            copy(&mut src_file, &mut dest_file)?;
            println!(
                "Extracted: {} -> {}",
                src_path.display(),
                dest_path.display()
            );
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn get_epoch() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub fn retriever() -> Result<String, String> {
    let metadata = build_info();
    let arguments = arguments(&metadata);
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

    // todo: threads are not really useful until manifest parsing is iterated
    let mut threads = Vec::new();
    for backup in backups {
        let manifest_db_path = backup.join("Manifest.db");
        println!("Manifest: {}", manifest_db_path.display());
        if manifest_db_path.exists() {
            match parse_manifest_db(&manifest_db_path) {
                Ok(files) => {
                    let files_cloned = files.clone();
                    let backup_cloned = backup.clone();
                    let output_dir_cloned = arguments.output_dir.clone();
                    let thread = thread::spawn(move || {
                        extract_files(&backup_cloned, &output_dir_cloned, &files_cloned)
                            .expect("Failed to extract files");
                    });
                    threads.push((files, thread));
                }
                Err(err) => {
                    let error = format!("Failed to parse manifest: {}", err);
                    return Err(error);
                }
            }
        }
    }
    for (files, thread) in threads {
        if thread.join().is_err() {
            println!("Error processing files {:?}", files);
        }
    }
    Ok("Success".into())
}
