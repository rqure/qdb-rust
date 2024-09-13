
#[derive(Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

pub trait LoggerTrait {
    fn log(&self, level: &LogLevel, message: &str);

    fn trace(&self, message: &str) {
        self.log(&LogLevel::Trace, message);
    }

    fn debug(&self, message: &str) {
        self.log(&LogLevel::Debug, message);
    }

    fn info(&self, message: &str) {
        self.log(&LogLevel::Info, message);
    }

    fn warning(&self, message: &str) {
        self.log(&LogLevel::Warning, message);
    }

    fn error(&self, message: &str) {
        self.log(&LogLevel::Error, message);
    }
}