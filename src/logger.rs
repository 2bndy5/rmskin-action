#![cfg(feature = "bin")]

use colored::Colorize;
use log::{self, Level, Metadata, Record};
use std::io::{Write, stdout};

struct Logger;

impl Logger {
    fn level_color(level: &Level) -> String {
        let name = format!("{:>5}", level.as_str().to_uppercase());
        match level {
            Level::Error => name.red().bold().to_string(),
            Level::Warn => name.yellow().bold().to_string(),
            Level::Info => name.green().bold().to_string(),
            Level::Debug => name.blue().bold().to_string(),
            Level::Trace => name.magenta().bold().to_string(),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        let mut stdout = stdout().lock();
        if self.enabled(record.metadata()) {
            writeln!(
                stdout,
                "[{}]: {}",
                Self::level_color(&record.level()),
                record.args()
            )
            .expect("Failed to write detailed log message to stdout");
            stdout
                .flush()
                .expect("Failed to flush log message to stdout");
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn logger_init() {
    let _ = log::set_logger(&LOGGER);
}

#[cfg(test)]
mod test {
    use super::logger_init;

    #[test]
    fn dummy_logs() {
        logger_init();
        log::set_max_level(log::LevelFilter::Trace);
        // we only need to output certain levels that are not used in other tests
        log::error!("This is not an error");
        log::trace!("There is no traceback");
        log::logger().flush();
    }
}
