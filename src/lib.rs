#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../README.md")]

/// Module to handle backup operations
pub mod backup;
/// Module to load the required structs
pub mod constant;
/// Module to handle database operations
pub mod fileio;
/// Module to construct a custom logger
pub mod logger;
/// Module to parse command line arguments
pub mod parser;
/// Module for helper functions
pub mod squire;

use rusqlite::Result;

/// Function to parse and extracrt iOS backup data
///
/// # Returns
///
/// * `Ok` - If the function completes successfully
/// * `Err` - If the function encounters an error
pub fn retriever() -> Result<String, String> {
    let metadata = constant::build_info();
    let arguments = parser::arguments(&metadata);
    if arguments.serial_numbers.is_empty() && !arguments.list {
        return Err(
            "Please provide a serial number (--serial) or use all (--all) / list (--list) options."
                .into(),
        );
    }
    log::set_logger(&logger::SimpleLogger).unwrap();
    if arguments.debug {
        log::set_max_level(log::LevelFilter::Debug);
        log::debug!("Debug mode enabled!!")
    } else {
        log::set_max_level(log::LevelFilter::Info);
    }
    log::info!(
        "Searching for backup data in '{}'",
        &arguments.backup_dir.display()
    );
    let backups = backup::get_backups(
        &arguments.backup_dir,
        &arguments.serial_numbers,
        arguments.list || arguments.all,
    );
    if backups.is_empty() {
        let err = if arguments.serial_numbers.is_empty() {
            format!("No backups found in '{}'", arguments.backup_dir.display())
        } else {
            format!(
                "No backups found for serial(s) '{:?}' in '{}'",
                arguments.serial_numbers,
                arguments.backup_dir.display()
            )
        };
        return Err(err);
    }
    if arguments.list {
        backup::list_backups(&backups);
        return Ok("".into());
    }

    let mut manifests = Vec::new();
    for backup in backups {
        let manifest_db_path = backup.path.join("Manifest.db");
        if manifest_db_path.exists() {
            manifests.push((backup, manifest_db_path));
        }
    }
    log::info!(
        "Number of manifests staged for extraction: {}",
        manifests.len()
    );
    log::info!("Number of workers assigned: {}", arguments.workers);
    for (backup, manifest) in manifests {
        let manifest_id = manifest
            .iter()
            .rev()
            .nth(1)
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        log::info!("Extracting manifest: '{}'", &manifest_id);
        let start = squire::get_epoch();
        match fileio::parse_manifest_db(&manifest, &backup, &arguments) {
            Ok(_) => {
                log::info!("Extraction completed for manifest: {:?}", manifest_id);
                log::info!(
                    "Time taken: {}",
                    squire::convert_seconds((squire::get_epoch() - start) as i64, 1)
                );
            }
            Err(err) => {
                log::error!("{}", err);
                return Err("".into());
            }
        }
    }
    Ok("".into())
}
