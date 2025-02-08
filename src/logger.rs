use chrono::Local;

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info // Set log level filter here
    }

    fn log(&self, record: &log::Record) {
        let local_time = Local::now();
        let formatted_time = local_time.format("%b-%d-%Y %I:%M:%S %p");
        println!("{} - {} - [{}:{}:{}] - {}", formatted_time, record.level(), record.target(), record.file().unwrap_or_default(), record.line().unwrap_or_default(), record.args());
    }

    fn flush(&self) {}
}
