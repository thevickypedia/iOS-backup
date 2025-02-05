mod constant;
mod parser;

use rusqlite::{Connection, Result};
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf};
use crate::constant::build_info;
use crate::parser::arguments;

fn get_ios_backup_directory() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    if cfg!(target_os = "windows") {
        home.join("AppData/Roaming/Apple Computer/MobileSync/Backup")
    } else {
        home.join("Library/Application Support/MobileSync/Backup")
    }
}

fn list_backups(backup_root: &Path) -> Vec<PathBuf> {
    let mut backups = Vec::new();
    if let Ok(entries) = fs::read_dir(backup_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("Info.plist").exists() {
                backups.push(path);
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

pub fn retriever() -> Result<()> {
    let backup_root = get_ios_backup_directory();
    let backups = list_backups(&backup_root);
    let metadata = build_info();
    let arguments = arguments(&metadata);

    for backup in backups {
        let manifest_db_path = backup.join("Manifest.db");
        if manifest_db_path.exists() {
            let files = parse_manifest_db(&manifest_db_path)?;
            extract_files(&backup, &arguments.backup_dir, &files).expect("Failed to extract files");
        }
    }
    // todo: Raise an error if no backups are found (manifest)
    Ok(())
}
