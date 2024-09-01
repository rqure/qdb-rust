pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Debug)]
pub enum Error {
    ClientError(String),
    DatabaseFieldError(String),
    NotificationError(String),
}

impl Error {
    pub fn from_client(msg: &str) -> Box<Self> {
        Box::new(Error::ClientError(msg.to_string()))
    }

    pub fn from_notification(msg: &str) -> Box<Self> {
        Box::new(Error::NotificationError(msg.to_string()))
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
            Error::NotificationError(msg) => write!(f, "Notification error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ClientError(_) => None,
            Error::DatabaseFieldError(_) => None,
            Error::NotificationError(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatabaseEntity {
    pub entity_id: String,
    pub entity_type: String,
    pub entity_name: String,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub context: Vec<DatabaseField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationConfig {
    pub entity_id: String,
    pub entity_type: String,
    pub field: String,
    pub notify_on_change: bool,
    pub context: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NotificationToken(String);

impl Into<String> for &NotificationToken {
    fn into(self) -> String {
        self.0.clone()
    }
}

impl From<String> for NotificationToken {
    fn from(s: String) -> Self {
        NotificationToken(s)
    }
}

impl From<&str> for NotificationToken {
    fn from(s: &str) -> Self {
        NotificationToken(s.to_string())
    }
}

pub trait NotificationManagerTrait {
    fn clear(&mut self);
    fn register(&mut self, client: &mut dyn ClientTrait, config: &NotificationConfig, callback: fn(&DatabaseNotification)) -> Result<NotificationToken>;
    fn unregister(&mut self, client: &mut dyn ClientTrait, token: &NotificationToken) -> Result<()>;
    fn process_notifications(&mut self, client: &mut dyn ClientTrait) -> Result<()>;
}

pub struct NotificationManager {
    registered_config: HashSet<NotificationConfig>,
    config_to_token: HashMap<NotificationConfig, NotificationToken>,
    token_to_callback_list: HashMap<NotificationToken, Vec<fn(&DatabaseNotification)>>
}

impl NotificationManager {
    pub fn new() -> Self {
        NotificationManager {
            registered_config: HashSet::new(),
            config_to_token: HashMap::new(),
            token_to_callback_list: HashMap::new()
        }
    }
}

impl NotificationManagerTrait for NotificationManager {
    fn clear(&mut self) {
        self.registered_config.clear();
        self.config_to_token.clear();
        self.token_to_callback_list.clear();
    }

    fn register(&mut self, client: &mut dyn ClientTrait, config: &NotificationConfig, callback: fn(&DatabaseNotification)) -> Result<NotificationToken> {
        if self.registered_config.contains(&config) {
            let token = self.config_to_token.get(config)
                .ok_or(Error::from_notification("Inconsistent notification state during registration"))?;
            
            self.token_to_callback_list.get_mut(token)
                .ok_or(Error::from_notification("Inconsistent notification state during registration"))?
                .push(callback);

            return Ok(token.clone());
        }

        let token = client.register_notification(config)?;
        
        self.registered_config.insert(config.clone());
        self.config_to_token.insert(config.clone(), token.clone());
        self.token_to_callback_list.insert(token.clone(), vec![callback]);

        Ok(token)
    }

    fn unregister(&mut self, client: &mut dyn ClientTrait, token: &NotificationToken) -> Result<()> {
        if !self.token_to_callback_list.contains_key(token) {
            return Err(Error::from_notification("Token not found during unregistration"));
        }

        client.unregister_notification(token)?;

        self.token_to_callback_list.remove(token);
        self.config_to_token.retain(|_, v| v != token);
        self.registered_config.retain(|c| self.config_to_token.contains_key(c));
        
        Ok(())
    }

    fn process_notifications(&mut self, client: &mut dyn ClientTrait) -> Result<()> {
        let notifications = client.get_notifications()?;

        for notification in &notifications {
            let token = NotificationToken(notification.token.clone());
            self.token_to_callback_list.get(&token)
                .unwrap_or(&vec![])
                .iter()
                .for_each(|callback| callback(&notification));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
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

pub trait ValueTrait {
    fn as_str(&self) -> Result<String>;
    fn as_i64(&self) -> Result<i64>;
    fn as_f64(&self) -> Result<f64>;
    fn as_bool(&self) -> Result<bool>;
    fn as_entity_reference(&self) -> Result<String>;
    fn as_timestamp(&self) -> Result<DateTime<Utc>>;
    fn as_connection_state(&self) -> Result<String>;
    fn as_garage_door_state(&self) -> Result<String>;
    fn update_str(&mut self, value: String) -> Result<()>;
    fn update_i64(&mut self, value: i64) -> Result<()>;
    fn update_f64(&mut self, value: f64) -> Result<()>;
    fn update_bool(&mut self, value: bool) -> Result<()>;
    fn update_entity_reference(&mut self, value: String) -> Result<()>;
    fn update_timestamp(&mut self, value: DateTime<Utc>) -> Result<()>;
    fn update_connection_state(&mut self, value: String) -> Result<()>;
    fn update_garage_door_state(&mut self, value: String) -> Result<()>;
}

impl ValueTrait for DatabaseValue {
    fn as_str(&self) -> Result<String> {
        match self {
            DatabaseValue::String(s) => Ok(s.clone()),
            _ => Err(Error::from_database_field("Value is not a string")),
        }
    }

    fn as_i64(&self) -> Result<i64> {
        match self {
            DatabaseValue::Integer(i) => Ok(*i),
            _ => Err(Error::from_database_field("Value is not an integer")),
        }
    }

    fn as_f64(&self) -> Result<f64> {
        match self {
            DatabaseValue::Float(f) => Ok(*f),
            _ => Err(Error::from_database_field("Value is not a float")),
        }
    }

    fn as_bool(&self) -> Result<bool> {
        match self {
            DatabaseValue::Boolean(b) => Ok(*b),
            _ => Err(Error::from_database_field("Value is not a boolean")),
        }
    }

    fn as_entity_reference(&self) -> Result<String> {
        match self {
            DatabaseValue::EntityReference(e) => Ok(e.clone()),
            _ => Err(Error::from_database_field(
                "Value is not an entity reference",
            )),
        }
    }

    fn as_timestamp(&self) -> Result<DateTime<Utc>> {
        match self {
            DatabaseValue::Timestamp(t) => Ok(*t),
            _ => Err(Error::from_database_field("Value is not a timestamp")),
        }
    }

    fn as_connection_state(&self) -> Result<String> {
        match self {
            DatabaseValue::ConnectionState(c) => Ok(c.clone()),
            _ => Err(Error::from_database_field(
                "Value is not a connection state",
            )),
        }
    }

    fn as_garage_door_state(&self) -> Result<String> {
        match self {
            DatabaseValue::GarageDoorState(g) => Ok(g.clone()),
            _ => Err(Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }

    fn update_str(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::String(s) => {
                *s = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a string")),
        }
    }

    fn update_i64(&mut self, value: i64) -> Result<()> {
        match self {
            DatabaseValue::Integer(i) => {
                *i = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not an integer")),
        }
    }

    fn update_f64(&mut self, value: f64) -> Result<()> {
        match self {
            DatabaseValue::Float(f) => {
                *f = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a float")),
        }
    }

    fn update_bool(&mut self, value: bool) -> Result<()> {
        match self {
            DatabaseValue::Boolean(b) => {
                *b = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a boolean")),
        }
    }

    fn update_entity_reference(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::EntityReference(e) => {
                *e = value;
                Ok(())
            }
            _ => Err(Error::from_database_field(
                "Value is not an entity reference",
            )),
        }
    }

    fn update_timestamp(&mut self, value: DateTime<Utc>) -> Result<()> {
        match self {
            DatabaseValue::Timestamp(t) => {
                *t = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a timestamp")),
        }
    }

    fn update_connection_state(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::ConnectionState(c) => {
                *c = value;
                Ok(())
            }
            _ => Err(Error::from_database_field(
                "Value is not a connection state",
            )),
        }
    }

    fn update_garage_door_state(&mut self, value: String) -> Result<()> {
        match self {
            DatabaseValue::GarageDoorState(g) => {
                *g = value;
                Ok(())
            }
            _ => Err(Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }
}

pub trait ClientTrait {
    fn connect(&mut self) -> Result<()>;
    fn connected(&self) -> bool;
    fn disconnect(&mut self) -> bool;
    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>>;
    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity>;
    fn get_notifications(&mut self) -> Result<Vec<DatabaseNotification>>;
    fn read(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()>;
    fn register_notification(&mut self, config: &NotificationConfig) -> Result<NotificationToken>;
    fn unregister_notification(&mut self, token: &NotificationToken) -> Result<()>;
    fn write(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()>;
}

pub mod rest;

pub trait DatabaseTrait {
    fn connect(&mut self) -> Result<()>;
    fn connected(&self) -> bool;
    fn disconnect(&mut self) -> bool;
    fn find(&mut self, entity_type: &str, field: &Vec<String>, predicate: fn(&HashMap<String, DatabaseField>) -> bool) -> Result<Vec<DatabaseEntity>>;
    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity>;
    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>>;
    fn read(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()>;
    fn write(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()>;
    fn clear_notifications(&mut self);
    fn register_notification(&mut self, config: &NotificationConfig, callback: fn(&DatabaseNotification)) -> Result<NotificationToken>;
    fn unregister_notification(&mut self, token: &NotificationToken) -> Result<()>;
    fn process_notifications(&mut self) -> Result<()>;
}

pub struct Database {
    client: Box<dyn ClientTrait>,
    notification_manager: Box<dyn NotificationManagerTrait>,
}

impl Database {
    pub fn new(client: Box<dyn ClientTrait>) -> Self {
        Database {
            client,
            notification_manager: Box::new(NotificationManager::new()),
        }
    }
}

impl DatabaseTrait for Database {
    fn clear_notifications(&mut self) {
        self.notification_manager.clear();
    }

    fn connect(&mut self) -> Result<()> {
        return self.client.connect();
    }

    fn connected(&self) -> bool {
        self.client.connected()
    }

    fn disconnect(&mut self) -> bool {
        self.client.disconnect()
    }

    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity> {
        self.client.get_entity(entity_id)
    }

    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        self.client.get_entities(entity_type)
    }

    fn find(&mut self, entity_type: &str, fields: &Vec<String>, predicate: fn(&HashMap<String, DatabaseField>) -> bool) -> Result<Vec<DatabaseEntity>> {
        let entities = self.get_entities(entity_type)?;
        let mut result = vec![];

        for entity in &entities {
            let mut requests = vec![];

            for field in fields {
                let field = DatabaseField::new(entity.entity_id.clone(), field.clone());
                requests.push(field);
            }
            
            self.read(&mut requests)?;

            let mut fields_map = HashMap::new();
            for field in &requests {
                fields_map.insert(field.name.clone(), field.clone());
            }

            if predicate(&fields_map) {
                result.push(entity.clone());
            }
        }

        Ok(result)
    }

    fn read(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        self.client.read(requests)
    }

    fn write(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        self.client.write(requests)
    }

    fn register_notification(&mut self, config: &NotificationConfig, callback: fn(&DatabaseNotification)) -> Result<NotificationToken> {
        self.notification_manager.register(&mut *self.client, config, callback)
    }

    fn unregister_notification(&mut self, token: &NotificationToken) -> Result<()> {
        self.notification_manager.unregister(&mut *self.client, token)
    }

    fn process_notifications(&mut self) -> Result<()> {
        return self.notification_manager.process_notifications(&mut *self.client);
    }
}

pub trait SlotTrait<T> {
    fn call(&mut self, args: T);
}

pub struct Slot<F> {
    callback: F,
}

impl<F> Slot<F> {
    pub fn new(callback: F) -> Self {
        Slot { callback }
    }

    pub fn call<T>(&mut self, args: &T)
    where
        F: FnMut(&T),
    {
        (self.callback)(args);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotToken(usize);

pub trait SignalTrait<F: FnMut(&T), T> {
    fn connect(&mut self, slot: Slot<F>) -> SlotToken;
    fn disconnect(&mut self, token: &SlotToken);
    fn emit(&mut self, args: &T);
}

pub struct Signal<F: FnMut(&T), T> {
    slots: HashMap<SlotToken, Slot<F>>,
    args: std::marker::PhantomData<T>,
}

impl<F: FnMut(&T), T> Signal<F, T> {
    pub fn new() -> Self {
        Signal {
            slots: HashMap::new(),
            args: std::marker::PhantomData,
        }
    }
}

impl<F: FnMut(&T), T> SignalTrait<F, T> for Signal<F, T> {
    fn connect(&mut self, slot: Slot<F>) -> SlotToken {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = SlotToken(COUNTER.fetch_add(1, Ordering::Relaxed));
        self.slots.insert(id, slot);
        id
    }

    fn disconnect(&mut self, id: &SlotToken) {
        self.slots.remove(id);
    }

    fn emit(&mut self, args: &T) {
        for (_, slot) in self.slots.iter_mut() {
            slot.call(args);
        }
    }
}

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

pub struct ConsoleLogger {
    level: LogLevel,
}

impl ConsoleLogger {
    pub fn new(level: LogLevel) -> Self {
        ConsoleLogger {
            level: level,
        }
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

// pub struct DefaultLogger {
//     loggers: Vec<Logger>,
// }

// impl DefaultLogger {
//     pub fn new() -> Self {
//         DefaultLogger { loggers: vec![] }
//     }

//     pub fn add_logger(&mut self, logger: Logger) {
//         self.loggers.push(logger);
//     }
// }

// impl LoggerTrait for DefaultLogger {
//     fn log(&self, level: &LogLevel, message: &str) {
//         for logger in &mut self.loggers {
//             logger.log(level, message);
//         }
//     }
// }

pub trait ApplicationTrait {
    fn execute(&mut self, ctx: &mut ApplicationContext);
}

pub struct ApplicationContext {
    pub database: Box<dyn DatabaseTrait>,
    pub logger: Box<dyn LoggerTrait>,
    pub quit: bool,
}

pub trait WorkerTrait {
    fn intialize(&mut self, ctx: &mut ApplicationContext) -> Result<()>;
    fn do_work(&mut self, ctx: &mut ApplicationContext) -> Result<()>;
    fn deinitialize(&mut self, ctx: &mut ApplicationContext) -> Result<()>;
}

pub struct Application {
    workers: Vec<Box<dyn WorkerTrait>>,
    loop_interval_ms: u64
}

impl Application {
    pub fn new(loop_interval_ms: u64) -> Self {
        Application {
            workers: vec![],
            loop_interval_ms
        }
    }
}

impl WorkerTrait for Application {
    fn intialize(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        ctx.logger.log(&LogLevel::Info, "[qdb::Application::initialize] Initializing application");
        for worker in &mut self.workers {
            match worker.intialize(ctx) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger.error(&format!(
                        "[qdb::Application::initialize] Error while initializing worker: {}",
                        e
                    ));
                }
            }
        }

        Ok(())
    }

    fn do_work(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        ctx.logger.log(&LogLevel::Info, "[qdb::Application::do_work] Application has started");

        while {
            let start = Instant::now();

            for worker in &mut self.workers {
                match worker.do_work(ctx) {
                    Ok(_) => {}
                    Err(e) => {
                        ctx.logger.error(&format!(
                            "[qdb::Application::do_work] Error while executing worker: {}",
                            e
                        ));
                    }
                }
            }

            if !ctx.quit {
                let loop_time = std::time::Duration::from_millis(self.loop_interval_ms);
                let elapsed_time = start.elapsed();
                
                if loop_time > elapsed_time {
                    let sleep_time = loop_time - start.elapsed();
                    std::thread::sleep(sleep_time);
                }
            }

            !ctx.quit
        } {}

        Ok(())
    }

    fn deinitialize(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        ctx.logger.log(&LogLevel::Info, "[qdb::Application::deinitialize] Deinitializing application");

        for worker in &mut self.workers {
            match worker.deinitialize(ctx) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger.error(&format!(
                        "[qdb::Application::deinitialize] Error while deinitializing worker: {}",
                        e
                    ));
                }
            }
        }

        ctx.logger.log(&LogLevel::Info, "[qdb::Application::deinitialize] Shutting down now");
        Ok(())
    }
}

impl ApplicationTrait for Application {
    fn execute(&mut self, ctx: &mut ApplicationContext) {
        self.intialize(ctx).unwrap();
        self.do_work(ctx).unwrap();
        self.deinitialize(ctx).unwrap();
    }
}

impl Application {
    pub fn add_worker(&mut self, worker: Box<dyn WorkerTrait>) {
        self.workers.push(worker);
    }
}

pub struct DatabaseWorkerSignals {
    pub connected: Signal<fn(&()), ()>,
    pub disconnected: Signal<fn(&()), ()>,
}

pub struct DatabaseWorker {
    connected: bool,
    pub signals: DatabaseWorkerSignals,
}

impl DatabaseWorker {
    pub fn new() -> Self {
        DatabaseWorker {
            connected: false,
            signals: DatabaseWorkerSignals {
                connected: Signal::new(),
                disconnected: Signal::new(),
            },
        }
    }
}

impl WorkerTrait for DatabaseWorker {
    fn intialize(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        ctx.logger.log(&LogLevel::Info, "[qdb::DatabaseWorker::initialize] Initializing database worker");
        Ok(())
    }

    fn do_work(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        if !ctx.database.connected() {
            if self.connected {
                ctx.logger.log(&LogLevel::Warning, "[qdb::DatabaseWorker::do_work] Disconnected from database");
                self.connected = false;
                ctx.database.clear_notifications();
                self.signals.disconnected.emit(&());
            }

            ctx.logger.log(&LogLevel::Debug, "[qdb::DatabaseWorker::do_work] Attempting to connect to the database...");
            
            ctx.database.disconnect();
            ctx.database.connect()?;

            if ctx.database.connected() {
                self.connected = true;
                ctx.logger.log(&LogLevel::Info, "[qdb::DatabaseWorker::do_work] Connected to the database");
                self.signals.connected.emit(&());
            }

            return Ok(())
        }

        ctx.database.process_notifications()?;

        Ok(())
    }

    fn deinitialize(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        ctx.logger.log(&LogLevel::Info, "[qdb::DatabaseWorker::deinitialize] Deinitializing database worker");
        Ok(())
    }
}