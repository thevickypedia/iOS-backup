use crate::constant;
use std::path::PathBuf;

/// Struct to construct the commandline arguments.
pub struct ArgConfig {
    // todo: fail if backup dir doesn't exist
    pub backup_dir: PathBuf,
}

fn missing_value(key: &str) {
    println!("{} requires a value.", key);
    std::process::exit(1)
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
    let mut backup_dir = String::new();

    // Loop through the command-line arguments and parse them.
    let mut i = 1; // Start from the second argument (args[0] is the program name).
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                let helper = "VaultAPI-Client takes the arguments, --env_file and --version/-v\n\n\
                --env_file: Custom filename to load the environment variables. Defaults to '.env'\n\
                --cipher: Cipher text to decrypt\n\
                --version: Get the package version.\n"
                    .to_string();
                println!("Usage: {} [OPTIONS]\n\n{}", args[0], helper);
                std::process::exit(0)
            }
            "-V" | "-v" | "--version" => {
                version = true;
            }
            // todo: add more args
            "--backup-dir" | "--backup_dir" | "--source" | "--src" => {
                i += 1; // Move to the next argument.
                if i < args.len() {
                    backup_dir = args[i].clone();
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
    ArgConfig {
        backup_dir: PathBuf::from(backup_dir),
    }
}
