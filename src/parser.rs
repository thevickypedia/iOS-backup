use crate::{constant, squire};
use std::path::PathBuf;

/// Enum to represent the different ways to organize the extracted files.
#[derive(Debug, Clone, Copy)]
pub enum Organizer {
    Type,
    Size,
    Root,
    Auto,
}

/// Struct to construct the commandline arguments.
pub struct ArgConfig {
    pub list: bool,
    pub all: bool,
    pub debug: bool,
    pub serial_numbers: Vec<String>,
    pub backup_dir: PathBuf,
    pub output_dir: PathBuf,
    pub workers: usize,
    pub organize: Organizer,
}

/// Function to print an error message and exit when a value is missing.
///
/// # Arguments
///
/// * `key` - The key that requires a value
fn missing_value(key: &str) {
    println!("ERROR: '{}' flag requires a value.", key);
    std::process::exit(1)
}

/// Default backup directory
///
/// # Returns
///
/// A `PathBuf` containing the default backup directory path
fn default_ios_backup_directory() -> PathBuf {
    let backup_dir = PathBuf::from(
        squire::env_var("backup_dir", Some(vec!["source_dir", "source", "src"]))
            .unwrap_or_default(),
    );
    if backup_dir.exists() {
        return backup_dir;
    }
    let home = dirs::home_dir().expect("Could not determine home directory");
    if cfg!(target_os = "windows") {
        home.join("AppData/Roaming/Apple Computer/MobileSync/Backup")
    } else {
        home.join("Library/Application Support/MobileSync/Backup")
    }
}

/// Default output directory
///
/// # Returns
///
/// A `PathBuf` containing the default output directory path
fn default_output_directory() -> PathBuf {
    let output_str = squire::env_var(
        "output_dir",
        Some(vec!["destination_dir", "destination", "dst"]),
    )
    .unwrap_or_default();
    if output_str.is_empty() {
        return PathBuf::from("extracted");
    }
    PathBuf::from(output_str)
}

/// Helper function to print the command-line arguments.
///
/// # Returns
///
/// A `String` containing the command-line arguments
fn helper() -> String {
    "ios crate takes the following arguments\n\n\
    \t--version: Print project version.\n\n\
    \t--list: List the available backups.\n\
    \t--debug: Enable debug level logging.\n\
    \t--all: Extract all available backups.\n\
    \t--serial: Initiate backup extraction for given serial number(s).\n\
    \t--organize: Organize the extracted files by type, size, root, and auto.\n\
    \t--workers | --threads: Numbers of workers (threads) to spin up for extraction.\n\
    \t--backup-dir | --source: Custom path for the backup. Defaults to OS specific path.\n\
    \t--output-dir | --destination: Destination directory. Defaults to 'extracted' in current path.\n"
    .to_string()
}

/// Parses and returns the command-line arguments.
///
/// # Arguments
/// * `metadata` - Metadata object loaded with cargo information.
///
/// # Returns
///
/// Commandline arguments loaded as an ``ArgConfig`` object.
pub fn arguments(metadata: &constant::MetaData) -> ArgConfig {
    let args: Vec<String> = std::env::args().collect();

    let mut version = false;
    let mut list = false;
    let mut all = false;
    let mut debug = false;
    let mut serial = String::new();
    let mut workers = String::new();
    let mut env_file = String::new();
    let mut backup_dir = String::new();
    let mut output_dir = String::new();
    let mut organize = Organizer::Auto;

    // Loop through the command-line arguments and parse them.
    let mut i = 1; // Start from the second argument (args[0] is the program name).
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" | "-H" => {
                println!("Usage: {} [OPTIONS]\n\n{}", args[0], helper());
                std::process::exit(0)
            }
            "-V" | "-v" | "--version" => {
                version = true;
            }
            "--list" => {
                list = true;
            }
            "--all" => {
                all = true;
            }
            "--debug" => {
                debug = true;
            }
            "--serial" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    serial = args[i].clone();
                } else {
                    missing_value(&args[i - 1]);
                }
            }
            "--workers" | "--threads" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    workers = args[i].clone();
                } else {
                    missing_value(&args[i - 1]);
                }
            }
            "--env" | "--env-file" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    env_file = args[i].clone();
                } else {
                    missing_value(&args[i - 1]);
                }
            }
            "--organize" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    match args[i].as_str() {
                        "type" => organize = Organizer::Type,
                        "size" => organize = Organizer::Size,
                        "root" => organize = Organizer::Root,
                        "auto" => organize = Organizer::Auto,
                        _ => {
                            println!("ERROR: '--organize' can only be 'type', 'size', 'root' or 'auto' (default)");
                            std::process::exit(1)
                        }
                    }
                } else {
                    missing_value(&args[i - 1]);
                }
            }
            "--backup-dir" | "--backup_dir" | "--source" | "--src" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    backup_dir = args[i].clone();
                } else {
                    missing_value(&args[i - 1]);
                }
            }
            "--output-dir" | "--output_dir" | "--destination" | "--dst" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    output_dir = args[i].clone();
                } else {
                    missing_value(&args[i - 1]);
                }
            }
            _ => {
                println!("\nERROR: Unknown argument: {}\n\n{}", args[i], helper());
                std::process::exit(1)
            }
        }
        i += 1;
    }
    if version {
        println!("{} {}", &metadata.pkg_name, &metadata.pkg_version);
        std::process::exit(0)
    }

    if env_file.is_empty() {
        env_file = squire::env_var("env_file", None).unwrap_or(".env".to_string())
    }
    let _ = dotenv::from_path(env_file);

    let backup_dir_final = if backup_dir.is_empty() {
        default_ios_backup_directory()
    } else {
        let tmp = PathBuf::from(backup_dir);
        if tmp.exists() {
            tmp
        } else {
            println!(
                "ERROR: Backup directory '{}' does not exist!",
                tmp.display()
            );
            std::process::exit(1)
        }
    };
    let output_dir_final = if output_dir.is_empty() {
        default_output_directory()
    } else {
        PathBuf::from(output_dir)
    };
    let workers_final = if workers.is_empty() {
        squire::default_workers()
    } else {
        workers.parse::<usize>().unwrap()
    };
    if serial.is_empty() {
        serial = squire::env_var("serial", None).unwrap_or_default()
    }
    let serial_numbers: Vec<String> = serial
        .split(",")
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    // todo: implement env var check for all CLI arguments
    // todo: remove the debug step
    println!("{:?}", backup_dir_final);
    println!("{:?}", output_dir_final);
    ArgConfig {
        list,
        all,
        debug,
        serial_numbers,
        backup_dir: backup_dir_final,
        output_dir: output_dir_final,
        workers: workers_final,
        organize,
    }
}
