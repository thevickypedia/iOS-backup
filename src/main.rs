use rusqlite::{Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};

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

fn main() -> Result<()> {
    let backup_root = get_ios_backup_directory();
    let backups = list_backups(&backup_root);

    for backup in backups {
        let manifest_db_path = backup.join("Manifest.db");
        if manifest_db_path.exists() {
            let files = parse_manifest_db(&manifest_db_path)?;
            for (file_id, relative_path) in files {
                println!("Found filepath: {} -> {}", file_id, relative_path);
                // Extraction logic can be implemented here
            }
        }
    }
    Ok(())
}
