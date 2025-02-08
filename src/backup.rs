use crate::{constant, fileio, squire};

use chrono::{DateTime, Local, Utc};
use plist::Value;
use std::fs::read_dir;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Function to list the available backups
///
/// # Arguments
///
/// * `backups` - A vector of `Backup` structs
pub fn list_backups(backups: &Vec<constant::Backup>) {
    let mut max_serial = "Serial Number".len() + 3;
    let mut max_device = "Device".len() + 3;
    let mut max_product = "Product Name".len() + 3;
    let mut max_date = "Backup Date".len() + 3;
    let mut max_encrypted = "Encrypted".len() + 3;
    let mut max_size = "Size".len() + 3;
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
            &backup.serial_number,
            &backup.device_name,
            &backup.product_name,
            &backup.backup_date,
            &backup.encrypted,
            &backup.backup_size,
        ));
    }

    let table_width = max_serial + max_device + max_date + max_encrypted + 3 * 3; // 3 spaces between columns
    let title = "Available iOS Device Backups";
    println!("\n\n{0:^1$}", title, table_width);

    println!(
        "{:-<width_serial$}  {:-<width_device$}  {:-<width_product$}  {:-<width_date$}  {:-<width_enc$}  {:-<width_size$}",
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
        "{:<width_serial$}  {:<width_device$}  {:<width_product$}  {:<width_date$}  {:<width_enc$}  {:<width_size$}",
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
        "{:-<width_serial$}  {:-<width_device$}  {:-<width_product$}  {:-<width_date$}  {:-<width_enc$}  {:-<width_size$}",
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
            "{:<width_serial$}  {:<width_device$}  {:<width_product$}  {:<width_date$}  {:<width_enc$}  {:<width_size$}",
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

/// Function to get the available backups
///
/// # Arguments
///
/// * `backup_root` - The path to the backup root directory
/// * `serial_filters` - The serial number(s) to filter the backups
/// * `no_filter` - Boolean flag to include all backups
///
/// # Returns
///
/// A vector of `Backup` structs
pub fn get_backups(
    backup_root: &Path,
    serial_filters: &[String],
    no_filter: bool,
) -> Vec<constant::Backup> {
    let mut backups = Vec::new();
    if let Ok(entries) = read_dir(backup_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let info_plist = path.join("Info.plist");
                if info_plist.exists() {
                    let info = Value::from_file(&info_plist).ok();
                    let serial_number = fileio::get_plist_key(&info, "Serial Number", "NO_SERIAL");
                    let device_name = fileio::get_plist_key(&info, "Device Name", "Unknown Device");
                    let product_name =
                        fileio::get_plist_key(&info, "Product Name", "Unknown Product");

                    let date = info
                        .as_ref()
                        .and_then(|v| v.as_dictionary()?.get("Last Backup Date"))
                        .and_then(Value::as_date);
                    let datetime = date.map_or("".to_string(), |date| {
                        let system_time: SystemTime = date.into();
                        let datetime_utc: DateTime<Utc> = system_time.into();
                        let datetime_local = datetime_utc.with_timezone(&Local);
                        datetime_local.format("%b %d, %Y %I:%M %p").to_string()
                    });
                    let seconds = date.map_or(0, |date| {
                        let system_time: SystemTime = date.into();
                        let duration_since_epoch = system_time
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or(Duration::new(0, 0));
                        duration_since_epoch.as_secs()
                    });
                    let backup_date = format!(
                        "{} ({} ago)",
                        datetime,
                        squire::convert_seconds((squire::get_epoch() - seconds) as i64, 1)
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
                    if no_filter || serial_filters.contains(&serial_number) {
                        backups.push(constant::Backup {
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
