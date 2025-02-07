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
use std::time::{SystemTime, UNIX_EPOCH};

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

                    // todo: Value is still returned as a Date object
                    let backup_date = info
                    .as_ref()
                    .and_then(|v| {
                        match v.as_dictionary() {
                            Some(dict) => dict.get("Last Backup Date"),
                            None => None,
                        }
                    })
                    .map_or("Unknown Date".to_string(), |v| format!("{:?}", v));

                    let encrypted = info
                    .as_ref()
                    .and_then(|v| {
                        match v.as_dictionary() {
                            Some(dict) => dict.get("IsEncrypted"),
                            None => None,
                        }
                    })
                    .map_or("No".to_string(), |v| {
                        match v.as_boolean() {
                            Some(true) => "Yes".to_string(),
                            _ => "No".to_string(),
                        }
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

fn parse_manifest_db(manifest_db_path: &Path) -> Result<Vec<(String, String)>> {
    let conn = Connection::open(manifest_db_path)?;
    let mut stmt = conn.prepare("SELECT fileID, relativePath FROM Files WHERE relativePath LIKE '%DCIM/%' OR relativePath LIKE '%PhotoData/%'")?;
    let rows = stmt.query_map([], |row| {
        let file_id: String = match row.get(0) {
            Ok(fid) => fid,
            Err(err) => return Err(err),
        };
        let relative_path: String = match row.get(1) {
            Ok(rp) => rp,
            Err(err) => return Err(err),
        };
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

    // todo: threads are not really useful until manifest parsing is iterated
    let mut threads = Vec::new();
    for backup in backups {
        let manifest_db_path = backup.path.join("Manifest.db");
        println!("Manifest: {}", manifest_db_path.display());
        if manifest_db_path.exists() {
            match parse_manifest_db(&manifest_db_path) {
                Ok(files) => {
                    let files_cloned = files.clone();
                    let backup_cloned = backup.path.clone();
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
