/// Struct for implementing a simple logger
pub struct SimpleLogger;

use chrono::Local;

/// ANSI encoded text styling for bold, underline and italic
struct Format;
#[allow(dead_code)]
impl Format {
    const BOLD: &'static str = "\x1b[1m";
    const UNDERLINE: &'static str = "\x1b[4m";
    const ITALIC: &'static str = "\x1b[3m";
}

/// ANSI encoded color scheme for text
struct Colors;
#[allow(dead_code)]
impl Colors {
    const VIOLET: &'static str = "\x1b[95m";
    const BLUE: &'static str = "\x1b[94m";
    const CYAN: &'static str = "\x1b[96m";
    const GREEN: &'static str = "\x1b[92m";
    const YELLOW: &'static str = "\x1b[93m";
    const RED: &'static str = "\x1b[91m";
    const END: &'static str = "\x1b[0m";
    const LIGHT_GREEN: &'static str = "\x1b[32m";
    const LIGHT_YELLOW: &'static str = "\x1b[2;33m";
    const LIGHT_RED: &'static str = "\x1b[31m";
}

/// Struct controller for different log levels
struct Echo;
#[allow(dead_code)]
impl Echo {
    fn error(msg: &String) {
        println!("{}ERROR{}:{:<6}{}", Colors::RED, Colors::END, "", msg);
    }
    fn warning(msg: &String) {
        println!("{}WARNING{}:{:<4}{}", Colors::YELLOW, Colors::END, "", msg);
    }
    fn info(msg: &String) {
        println!("{}INFO{}:{:<7}{}", Colors::GREEN, Colors::END, "", msg);
    }
    fn debug(msg: &String) {
        println!(
            "{}DEBUG{}:{:<6}{}",
            Colors::LIGHT_GREEN,
            Colors::END,
            "",
            msg
        );
    }
    fn trace(msg: &String) {
        println!(
            "{}{}CRITICAL{}:{:<1}{}",
            Colors::LIGHT_GREEN,
            Format::BOLD,
            Colors::END,
            "",
            msg
        );
    }
}

/// Implementation of the `Log` trait for the `SimpleLogger` struct
///
/// This implementation provides the `enabled`, `log`, and `flush` functions
/// for the `SimpleLogger` struct
///
/// The log format is customized to include the current time, log level, target,
/// file, line, and the message
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info // Set log level filter here
    }

    fn log(&self, record: &log::Record) {
        let local_time = Local::now();
        let formatted_time = local_time.format("%b-%d-%Y %I:%M:%S %p");
        let msg = format!(
            "{}  [{}:{}:{}]  {}",
            formatted_time,
            record.target(),
            record.file().unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        );
        match record.level() {
            log::Level::Debug => Echo::debug(&msg),
            log::Level::Info => Echo::info(&msg),
            log::Level::Warn => Echo::warning(&msg),
            log::Level::Error => Echo::error(&msg),
            log::Level::Trace => Echo::trace(&msg),
        }
    }

    fn flush(&self) {}
}
