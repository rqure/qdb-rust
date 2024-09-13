
pub struct ConsoleLogger {
    level: LogLevel,
}

impl ConsoleLogger {
    pub fn new(level: LogLevel) -> Self {
        ConsoleLogger { level: level }
    }
}

impl LoggerTrait for ConsoleLogger {
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