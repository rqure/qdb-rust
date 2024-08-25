pub type Result<T> = core::result::Result<T, IError>;
pub type IClient = Box<dyn ClientTrait>;
pub type IError = Box<dyn std::error::Error>;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum Error {
    ClientError(String),
    DatabaseFieldError(String),
}

impl Error {
    pub fn from_client(msg: &str) -> Box<Self> {
        Box::new(Error::ClientError(msg.to_string()))
    }

    pub fn from_database_field(msg: &str) -> Box<Self> {
        Box::new(Error::DatabaseFieldError(msg.to_string()))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ClientError(msg) => write!(f, "Client error: {}", msg),
            Error::DatabaseFieldError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ClientError(_) => None,
            Error::DatabaseFieldError(_) => None,
        }
    }
}

pub struct DatabaseEntity {
    pub entity_id: String,
    pub entity_type: String,
    pub entity_name: String,
}

pub struct DatabaseField {
    pub entity_id: String,
    pub name: String,
    pub value: DatabaseValue,
    pub write_time: DateTime<Utc>,
    pub writer_id: String,
}

impl DatabaseField {
    pub fn new(entity_id: impl Into<String>, field: impl Into<String>) -> Self {
        DatabaseField {
            entity_id: entity_id.into(),
            name: field.into(),
            value: DatabaseValue::Unspecified,
            write_time: Utc::now(),
            writer_id: "".to_string(),
        }
    }
}

pub struct DatabaseNotification {
    pub token: String,
    pub current: DatabaseField,
    pub previous: DatabaseField,
    pub context: Vec<DatabaseField>
}

pub struct NotificationConfig {
    pub entity_id: String,
    pub field: String,
    pub notify_on_change: bool,
    pub context: Vec<String>
}

pub type NotificationToken = String;

#[derive(Debug)]
pub enum DatabaseValue {
    Unspecified,
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    EntityReference(String),
    Timestamp(DateTime<Utc>),
    ConnectionState(String),
    GarageDoorState(String),
}

impl DatabaseValue {
    pub fn as_str(&self) -> Result<&String> {
        match self {
            DatabaseValue::String(s) => Ok(s),
            _ => Err(Error::from_database_field("Value is not a string")),
        }
    }

    pub fn as_i64(&self) -> Result<&i64> {
        match self {
            DatabaseValue::Integer(i) => Ok(i),
            _ => Err(Error::from_database_field("Value is not an integer")),
        }
    }

    pub fn as_f64(&self) -> Result<&f64> {
        match self {
            DatabaseValue::Float(f) => Ok(f),
            _ => Err(Error::from_database_field("Value is not a float")),
        }
    }

    pub fn as_bool(&self) -> Result<&bool> {
        match self {
            DatabaseValue::Boolean(b) => Ok(b),
            _ => Err(Error::from_database_field("Value is not a boolean")),
        }
    }

    pub fn as_entity_reference(&self) -> Result<&String> {
        match self {
            DatabaseValue::EntityReference(e) => Ok(e),
            _ => Err(Error::from_database_field("Value is not an entity reference")),
        }
    }

    pub fn as_timestamp(&self) -> Result<&DateTime<Utc>> {
        match self {
            DatabaseValue::Timestamp(t) => Ok(t),
            _ => Err(Error::from_database_field("Value is not a timestamp")),
        }
    }

    pub fn as_connection_state(&self) -> Result<&String> {
        match self {
            DatabaseValue::ConnectionState(c) => Ok(c),
            _ => Err(Error::from_database_field("Value is not a connection state")),
        }
    }

    pub fn as_garage_door_state(&self) -> Result<&String> {
        match self {
            DatabaseValue::GarageDoorState(g) => Ok(g),
            _ => Err(Error::from_database_field("Value is not a garage door state")),
        }
    }

    pub fn update_str(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::String(s) => {
                *s = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not a string")),
        }
    }

    pub fn update_i64(&mut self, value: i64) -> Result<()> {
        match self {
            DatabaseValue::Integer(i) => {
                *i = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not an integer")),
        }
    }

    pub fn update_f64(&mut self, value: f64) -> Result<()> {
        match self {
            DatabaseValue::Float(f) => {
                *f = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not a float")),
        }
    }

    pub fn update_bool(&mut self, value: bool) -> Result<()> {
        match self {
            DatabaseValue::Boolean(b) => {
                *b = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not a boolean")),
        }
    }

    pub fn update_entity_reference(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::EntityReference(e) => {
                *e = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not an entity reference")),
        }
    }

    pub fn update_timestamp(&mut self, value: DateTime<Utc>) -> Result<()> {
        match self {
            DatabaseValue::Timestamp(t) => {
                *t = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not a timestamp")),
        }
    }

    pub fn update_connection_state(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::ConnectionState(c) => {
                *c = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not a connection state")),
        }
    }

    pub fn update_garage_door_state(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::GarageDoorState(g) => {
                *g = value;
                Ok(())
            },
            _ => Err(Error::from_database_field("Value is not a garage door state")),
        }
    }
}

pub trait ClientTrait {
    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity>;
    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>>;
    fn read(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()>;
    fn write(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()>;
    // fn register_notification(&self, config: NotificationConfig) -> Result<NotificationToken>;
    // fn unregister_notification(&self, token: NotificationToken) -> Result<()>;
    // fn process_notifications(&self) -> Result<Vec<DatabaseNotification>>;
}

pub mod rest;
