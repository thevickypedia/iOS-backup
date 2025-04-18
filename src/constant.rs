use std::env;
use std::path::PathBuf;

/// Struct to store the backup information
///
/// This includes the path to the backup, serial number, device name, product name, backup date, backup size, and encryption status.
pub struct Backup {
    pub path: PathBuf,
    pub serial_number: String,
    pub device_name: String,
    pub product_name: String,
    pub backup_date: String,
    pub backup_size: String,
    pub encrypted: String,
}

/// Struct to store the cargo information gathered at compile time using the `env!` macro.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetaData {
    pub crate_name: String,
    pub manifest_dir: String,
    pub authors: Vec<String>,
    pub description: String,
    pub homepage: String,
    pub pkg_name: String,
    pub pkg_repo: String,
    pub pkg_version: String,
    pub pkg_version_major: String,
    pub pkg_version_minor: String,
    pub pkg_version_patch: String,
    pub pkg_version_pre: String,
}

/// Uses compile time macros to load Cargo metadata via environment variables during compilation process
///
/// ## References
/// - [Official Docs](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
/// - [GitHub Issues](https://github.com/rust-lang/cargo/issues/8251#issuecomment-631731144)
/// - [GitHub Issues](https://github.com/rust-lang/cargo/issues/11966#issue-1664748892)
pub fn build_info() -> MetaData {
    MetaData {
        crate_name: env!("CARGO_CRATE_NAME").to_string(),
        manifest_dir: env!("CARGO_MANIFEST_DIR").to_string(),
        authors: env!("CARGO_PKG_AUTHORS")
            .split(',')
            .map(String::from)
            .collect(),
        description: env!("CARGO_PKG_DESCRIPTION").to_string(),
        homepage: env!("CARGO_PKG_HOMEPAGE").to_string(),
        pkg_name: env!("CARGO_PKG_NAME").to_string(),
        pkg_repo: env!("CARGO_PKG_REPOSITORY").to_string(),
        pkg_version: env!("CARGO_PKG_VERSION").to_string(),
        pkg_version_major: env!("CARGO_PKG_VERSION_MAJOR").to_string(),
        pkg_version_minor: env!("CARGO_PKG_VERSION_MINOR").to_string(),
        pkg_version_patch: env!("CARGO_PKG_VERSION_PATCH").to_string(),
        pkg_version_pre: env!("CARGO_PKG_VERSION_PRE").to_string(),
    }
}
