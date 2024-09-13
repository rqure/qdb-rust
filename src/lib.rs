pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

pub mod clients;
pub mod error;
pub mod framework;
pub mod loggers;
pub mod schema;