pub type Result<T> = core::result::Result<T, IError>;
pub type IClient = Box<dyn ClientTrait>;
pub type IError = Box<dyn std::error::Error>;

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
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

#[derive(Debug)]
pub struct DatabaseEntity {
    pub entity_id: String,
    pub entity_type: String,
    pub entity_name: String,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct DatabaseNotification {
    pub token: String,
    pub current: DatabaseField,
    pub previous: DatabaseField,
    pub context: Vec<DatabaseField>
}

#[derive(Debug)]
pub struct NotificationConfig {
    pub entity_id: String,
    pub entity_type: String,
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
    fn register_notification(&mut self, config: NotificationConfig) -> Result<NotificationToken>;
    fn unregister_notification(&mut self, token: NotificationToken) -> Result<()>;
    fn process_notifications(&mut self) -> Result<Vec<DatabaseNotification>>;
}

pub mod rest;

pub trait SlotTrait<T> {
    fn call(&mut self, args: T);
}

pub struct Slot<F>
{
    callback: F
}

impl<F> Slot<F>
{
    pub fn new(callback: F) -> Self
    {
        Slot { callback }
    }

    pub fn call<T>(&mut self, args: &T)
    where
        F: FnMut(&T)
    {
        (self.callback)(args);
    }
}

pub trait SignalTrait<F: FnMut(&T), T>
{
    fn connect(&mut self, slot: Slot<F>) -> SignalSlotConnection<F, T>;
    fn disconnect(&mut self, id: usize);
    fn emit(&mut self, args: &T);
}

struct SignalInternal<F: FnMut(&T), T>
{
    slots: HashMap<usize, Slot<F>>,
    args: std::marker::PhantomData<T>,
}

pub struct Signal<F: FnMut(&T), T>
{
    internal: Rc<RefCell<SignalInternal<F, T>>>
}

pub struct SignalSlotConnection<F: FnMut(&T), T>
{
    id: usize,
    signal: std::rc::Weak<RefCell<SignalInternal<F, T>>>
}

impl<F: FnMut(&T), T> SignalSlotConnection<F, T>
{
    pub fn disconnect(&mut self)
    {
        if let Some(signal) = self.signal.upgrade()
        {
            signal.borrow_mut().slots.remove(&self.id);
        }
    }
}

impl<F: FnMut(&T), T> Signal<F, T>
{
    pub fn new() -> Self
    {
        Signal {
            internal: Rc::new(RefCell::new(SignalInternal { slots: HashMap::new(), args: std::marker::PhantomData }))
        }
    }
}

impl<F: FnMut(&T), T> SignalTrait<F, T> for Signal<F, T>
{
    fn connect(&mut self, slot: Slot<F>) -> SignalSlotConnection<F, T>
    {
        static COUNTER : AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        self.internal.borrow_mut().slots.insert(id, slot);
        SignalSlotConnection { id, signal: Rc::downgrade(&self.internal) }
    }

    fn disconnect(&mut self, id: usize)
    {
        self.internal.borrow_mut().slots.remove(&id);
    }

    fn emit(&mut self, args: &T)
    {
        for (_, slot) in self.internal.borrow_mut().slots.iter_mut()
        {
            slot.call(args);
        }
    }
}
