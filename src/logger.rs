use chrono::Local;

pub struct SimpleLogger;

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
        println!(
            "{} - {} - [{}:{}:{}] - {}",
            formatted_time,
            record.level(),
            record.target(),
            record.file().unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        );
    }

    fn flush(&self) {}
}
