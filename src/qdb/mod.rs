pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
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

pub struct NotificationToken(String);

impl Into<String> for NotificationToken {
    fn into(self) -> String {
        self.0
    }
}

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

pub trait DatabaseTrait {}

pub struct Database {
    notifications: HashMap<NotificationToken, NotificationConfig>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotToken(usize);

pub trait SignalTrait<F: FnMut(&T), T>
{
    fn connect(&mut self, slot: Slot<F>) -> SlotToken;
    fn disconnect(&mut self, token: &SlotToken);
    fn emit(&mut self, args: &T);
}

pub struct Signal<F: FnMut(&T), T>
{
    slots: HashMap<SlotToken, Slot<F>>,
    args: std::marker::PhantomData<T>,
}

impl<F: FnMut(&T), T> Signal<F, T>
{
    pub fn new() -> Self
    {
        Signal {
           slots: HashMap::new(),
           args: std::marker::PhantomData,
        }
    }
}

impl<F: FnMut(&T), T> SignalTrait<F, T> for Signal<F, T>
{
    fn connect(&mut self, slot: Slot<F>) -> SlotToken
    {
        static COUNTER : AtomicUsize = AtomicUsize::new(0);
        let id = SlotToken(COUNTER.fetch_add(1, Ordering::Relaxed));
        self.slots.insert(id, slot);
        id
    }

    fn disconnect(&mut self, id: &SlotToken)
    {
        self.slots.remove(id);
    }

    fn emit(&mut self, args: &T)
    {
        for (_, slot) in self.slots.iter_mut()
        {
            slot.call(args);
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum LogLevel
{
    Trace,
    Debug,
    Info,
    Warning,
    Error
}

pub trait LoggerTrait
{
    fn log(&mut self, level: &LogLevel, message: &str);

    fn trace(&mut self, message: &str)
    {
        self.log(&LogLevel::Trace, message);
    }

    fn debug(&mut self, message: &str)
    {
        self.log(&LogLevel::Debug, message);
    }

    fn info(&mut self, message: &str)
    {
        self.log(&LogLevel::Info, message);
    }

    fn warning(&mut self, message: &str)
    {
        self.log(&LogLevel::Warning, message);
    }

    fn error(&mut self, message: &str)
    {
        self.log(&LogLevel::Error, message);
    }
}

pub type LoggerRef = Rc<RefCell<dyn LoggerTrait>>;
pub struct Logger(LoggerRef);

impl Logger
{
    pub fn clone(&self) -> Self
    {
        Logger(self.0.clone())
    }    
}

impl LoggerTrait for Logger
{
    fn log(&mut self, level: &LogLevel, message: &str)
    {
        self.0.borrow_mut().log(level, message);
    }
}

pub struct PrintLogger
{
    level: LogLevel
}

impl PrintLogger
{
    pub fn new(level: LogLevel) -> Logger
    {
        Logger(Rc::new(RefCell::new(
            PrintLogger { level }
        )))
    }
}

impl LoggerTrait for PrintLogger
{
    fn log(&mut self, level: &LogLevel, message: &str)
    {
        if *level >= self.level
        {
            println!("{} | {} | {}", Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true), match level {
                LogLevel::Trace => "TRACE",
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warning => "WARNING",
                LogLevel::Error => "ERROR"
            }, message);
        }
    }
}

pub struct DefaultLogger
{
    loggers: Vec<Logger>
}

impl DefaultLogger
{
    pub fn new() -> Self
    {
        DefaultLogger { loggers: vec![] }
    }

    pub fn add_logger(&mut self, logger: Logger)
    {
        self.loggers.push(logger);
    }
}

impl LoggerTrait for DefaultLogger
{
    fn log(&mut self, level: &LogLevel, message: &str)
    {
        for logger in &mut self.loggers
        {
            logger.log(level, message);
        }
    }
}

pub trait ApplicationTrait
{
    fn execute(&mut self);
}

pub struct ApplicationContext
{
    logger: Logger,
    quit: bool
}

pub trait WorkerTrait
{
    fn intialize(&mut self, ctx: &mut ApplicationContext) -> Result<()>;
    fn do_work(&mut self, ctx: &mut ApplicationContext) -> Result<()>;
    fn deinitialize(&mut self, ctx: &mut ApplicationContext) -> Result<()>;
}

pub struct Application
{
    workers: Vec<Box<dyn WorkerTrait>>,
    loop_interval_ms: u64,
    logger: Logger
}

impl Application
{
    pub fn new(loop_interval_ms: u64, logger: Logger) -> Self
    {
        Application {
            workers: vec![],
            loop_interval_ms,
            logger
        }
    }
}

impl WorkerTrait for Application {
    fn intialize(&mut self, ctx: &mut ApplicationContext) -> Result<()>
    {
        ctx.quit = false;

        for worker in &mut self.workers
        {
            match worker.intialize(ctx) {
                Ok(_) => {},
                Err(e) => {
                    ctx.logger.error(&format!("[qdb::Application::initialize] Error while initializing worker: {}", e));
                }
            }
        }

        Ok(())
    }

    fn do_work(&mut self, ctx: &mut ApplicationContext) -> Result<()>
    {
        while !ctx.quit
        {
            let start = Instant::now();

            for worker in &mut self.workers
            {
                match worker.do_work(ctx) {
                    Ok(_) => {},
                    Err(e) => {
                        ctx.logger.error(&format!("[qdb::Application::do_work] Error while executing worker: {}", e));
                    }
                }
            }

            let sleep_time = std::time::Duration::from_millis(self.loop_interval_ms) - start.elapsed();
            std::thread::sleep(sleep_time);
        }

        Ok(())
    }

    fn deinitialize(&mut self, ctx: &mut ApplicationContext) -> Result<()>
    {
        for worker in &mut self.workers
        {
            match worker.deinitialize(ctx) {
                Ok(_) => {},
                Err(e) => {
                    ctx.logger.error(&format!("[qdb::Application::deinitialize] Error while deinitializing worker: {}", e));
                }
            }
        }

        Ok(())
    }
}

impl ApplicationTrait for Application
{
    fn execute(&mut self)
    {
        let mut ctx = ApplicationContext {
            logger: self.logger.clone(),
            quit: false
        };

        self.intialize(&mut ctx).unwrap_or_default();
        self.do_work(&mut ctx).unwrap_or_default();
        self.deinitialize(&mut ctx).unwrap_or_default();
    }
}

impl Application
{
    pub fn add_worker(&mut self, worker: Box<dyn WorkerTrait>)
    {
        self.workers.push(worker);
    }
}