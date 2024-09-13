use crate::loggers::common::{LogLevel, LoggerTrait};
use chrono::Utc;

pub struct Console {
    level: LogLevel,
}

impl Console {
    pub fn new(level: LogLevel) -> Self {
        Console { level: level }
    }
}

impl LoggerTrait for Console {
    fn log(&self, level: &LogLevel, message: &str) {
        if *level >= self.level {
            println!(
                "{} | {} | {}",
                Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                match level {
                    LogLevel::Trace => "TRACE",
                    LogLevel::Debug => "DEBUG",
                    LogLevel::Info => "INFO",
                    LogLevel::Warning => "WARNING",
                    LogLevel::Error => "ERROR",
                },
                message
            );
        }
    }
}