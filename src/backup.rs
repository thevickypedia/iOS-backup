use std::path::PathBuf;

pub struct Backup {
    pub path: PathBuf,
    pub serial_number: String,
    pub device_name: String,
    pub product_name: String,
    pub backup_date: String,
    pub backup_size: String,
    pub encrypted: String,
}
