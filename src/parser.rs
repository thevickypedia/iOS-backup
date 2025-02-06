use crate::constant;
use std::path::PathBuf;

/// Struct to construct the commandline arguments.
pub struct ArgConfig {
    pub list: bool,
    pub serial_number: String,
    pub backup_dir: PathBuf,
    pub output_dir: PathBuf,
}

fn missing_value(key: &str) {
    println!("{} requires a value.", key);
    std::process::exit(1)
}

/// Default backup directory
fn default_ios_backup_directory() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    if cfg!(target_os = "windows") {
        home.join("AppData/Roaming/Apple Computer/MobileSync/Backup")
    } else {
        home.join("Library/Application Support/MobileSync/Backup")
    }
}

/// Parses and returns the command-line arguments.
///
/// # Arguments
/// * `metadata` - Metadata object loaded with cargo information.
///
/// # Returns
/// * Commandline arguments loaded as an ``ArgConfig`` object.
pub fn arguments(metadata: &constant::MetaData) -> ArgConfig {
    let args: Vec<String> = std::env::args().collect();

    let mut version = false;
    let mut list = false;
    let mut serial = String::new();
    let mut backup_dir = String::new();
    let mut output_dir = String::new();

    // Loop through the command-line arguments and parse them.
    let mut i = 1; // Start from the second argument (args[0] is the program name).
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                let helper = "ios takes the following arguments\n\n\
                --version/-v\n\
                --backup-dir | --source: Custom path for the backup. Defaults to OS specific\n\
                --output-dir | --destination: Destination directory. Defaults to 'extracted'\n\
                --list | -L: List the available backups.\n"
                    .to_string();
                println!("Usage: {} [OPTIONS]\n\n{}", args[0], helper);
                std::process::exit(0)
            }
            "-V" | "-v" | "--version" => {
                version = true;
            }
            "--list" | "-L" => {
                list = true;
            }
            "--serial" | "-S" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    serial = args[i].clone();
                } else {
                    missing_value(&args[i]);
                }
            }
            "--backup-dir" | "--backup_dir" | "--source" | "--src" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    backup_dir = args[i].clone();
                } else {
                    missing_value(&args[i]);
                }
            }
            "--output-dir" | "--output_dir" | "--destination" | "--dst" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    output_dir = args[i].clone();
                } else {
                    missing_value(&args[i]);
                }
            }
            _ => {
                println!("Unknown argument: {}", args[i]);
                std::process::exit(1)
            }
        }
        i += 1;
    }
    if version {
        println!("{} {}", &metadata.pkg_name, &metadata.pkg_version);
        std::process::exit(0)
    }
    let backup_dir_final = if backup_dir.is_empty() {
        default_ios_backup_directory()
    } else {
        let tmp = PathBuf::from(backup_dir);
        if tmp.exists() {
            tmp
        } else {
            println!("Backup directory '{}' does not exist!", tmp.display());
            std::process::exit(1)
        }
    };
    let output_dir_final = if output_dir.is_empty() {
        PathBuf::from("extracted")
    } else {
        PathBuf::from(output_dir)
    };
    ArgConfig {
        list,
        serial_number: serial,
        backup_dir: backup_dir_final,
        output_dir: output_dir_final,
    }
}
