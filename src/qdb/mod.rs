pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;
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

pub type FieldRef = Rc<RefCell<RawField>>;

pub struct DatabaseField(FieldRef);

impl DatabaseField {
    pub fn new(field: RawField) -> Self {
        DatabaseField(Rc::new(RefCell::new(field)))
    }

    pub fn clone(&self) -> Self {
        DatabaseField(self.0.clone())
    }

    pub fn into_raw(self) -> RawField {
        let field = self.0.borrow();
        RawField {
            entity_id: field.entity_id(),
            name: field.name(),
            value: field.value(),
            write_time: field.write_time(),
            writer_id: field.writer_id(),
        }
    }

    pub fn entity_id(&self) -> String {
        self.0.borrow().entity_id()
    }

    pub fn name(&self) -> String {
        self.0.borrow().name()
    }

    pub fn value(&self) -> DatabaseValue {
        self.0.borrow().value()
    }

    pub fn write_time(&self) -> DateTime<Utc> {
        self.0.borrow().write_time()
    }

    pub fn writer_id(&self) -> String {
        self.0.borrow().writer_id()
    }

    pub fn update_entity_id(&self, entity_id: &str) {
        self.0.borrow_mut().update_entity_id(entity_id);
    }

    pub fn update_value(&self, value: DatabaseValue) {
        self.0.borrow_mut().update_value(value);
    }

    pub fn update_write_time(&self, write_time: DateTime<Utc>) {
        self.0.borrow_mut().update_write_time(write_time);
    }

    pub fn update_writer_id(&self, writer_id: &str) {
        self.0.borrow_mut().update_writer_id(writer_id);
    }

    pub fn update_name(&self, name: &str) {
        self.0.borrow_mut().update_name(name);
    }
}

pub struct RawField {
    pub entity_id: String,
    pub name: String,
    pub value: DatabaseValue,
    pub write_time: DateTime<Utc>,
    pub writer_id: String,
}

impl RawField {
    fn entity_id(&self) -> String {
        self.entity_id.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn value(&self) -> DatabaseValue {
        self.value.clone()
    }

    fn write_time(&self) -> DateTime<Utc> {
        self.write_time
    }

    fn writer_id(&self) -> String {
        self.writer_id.clone()
    }

    fn update_entity_id(&mut self, entity_id: &str) {
        self.entity_id = entity_id.into();
    }

    fn update_value(&mut self, value: DatabaseValue) {
        self.value = value;
    }

    fn update_write_time(&mut self, write_time: DateTime<Utc>) {
        self.write_time = write_time;
    }

    fn update_writer_id(&mut self, writer_id: &str) {
        self.writer_id = writer_id.into();
    }

    fn update_name(&mut self, name: &str) {
        self.name = name.into();
    }
}

impl RawField {
    pub fn new(entity_id: impl Into<String>, field: impl Into<String>) -> Self {
        RawField {
            entity_id: entity_id.into(),
            name: field.into(),
            value: DatabaseValue::new(RawValue::Unspecified),
            write_time: Utc::now(),
            writer_id: "".to_string(),
        }
    }

    pub fn into_field(self) -> DatabaseField {
        DatabaseField::new(self)
    }
}

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


pub struct NotificationCallback(Box<dyn FnMut(&DatabaseNotification) -> Result<()>>);

impl NotificationCallback {
    pub fn new<'a>(callback: impl FnMut(&DatabaseNotification) -> Result<()> + 'a) -> Self {
        NotificationCallback(Box::new(callback))
    }
}

pub struct _NotificationManager {
    registered_config: HashSet<NotificationConfig>,
    config_to_token: HashMap<NotificationConfig, NotificationToken>,
    token_to_callback_list: HashMap<NotificationToken, Vec<NotificationCallback>>
}

type NotificationManagerRef = Rc<RefCell<_NotificationManager>>;
pub struct NotificationManager(NotificationManagerRef);

impl NotificationManager {
    pub fn new() -> Self {
        NotificationManager(Rc::new(RefCell::new(_NotificationManager::new())))
    }

    pub fn clone(&self) -> Self {
        NotificationManager(self.0.clone())
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub fn register(&self, client: Client, config: &NotificationConfig, callback: NotificationCallback) -> Result<NotificationToken> {
        self.0.borrow_mut().register(client, config, callback)
    }

    pub fn unregister(&self, client: Client, token: &NotificationToken) -> Result<()> {
        self.0.borrow_mut().unregister(client, token)
    }

    pub fn process_notifications(&self, client: Client) -> Result<()> {
        self.0.borrow_mut().process_notifications(client)
    }
}

impl _NotificationManager {
    pub fn new() -> Self {
        _NotificationManager {
            registered_config: HashSet::new(),
            config_to_token: HashMap::new(),
            token_to_callback_list: HashMap::new()
        }
    }
}

impl _NotificationManager {
    fn clear(&mut self) {
        self.registered_config.clear();
        self.config_to_token.clear();
        self.token_to_callback_list.clear();
    }

    fn register(&mut self, client: Client, config: &NotificationConfig, callback: NotificationCallback) -> Result<NotificationToken> {
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

    fn unregister(&mut self, client: Client, token: &NotificationToken) -> Result<()> {
        if !self.token_to_callback_list.contains_key(token) {
            return Err(Error::from_notification("Token not found during unregistration"));
        }

        client.unregister_notification(token)?;

        self.token_to_callback_list.remove(token);
        self.config_to_token.retain(|_, v| v != token);
        self.registered_config.retain(|c| self.config_to_token.contains_key(c));
        
        Ok(())
    }

    fn process_notifications(&mut self, client: Client) -> Result<()> {
        let notifications = client.get_notifications()?;

        for notification in &notifications {
            let token = NotificationToken(notification.token.clone());
            self.token_to_callback_list.get_mut(&token)
                .unwrap_or(&mut vec![])
                .iter_mut()
                .for_each(|callback| {
                    callback.0(notification);
                });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawValue {
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

impl RawValue {
    fn into_value(self) -> DatabaseValue {
        DatabaseValue::new(self)
    }

    fn as_str(&self) -> Result<String> {
        match self {
            RawValue::String(s) => Ok(s.clone()),
            _ => Err(Error::from_database_field("Value is not a string")),
        }
    }

    fn as_i64(&self) -> Result<i64> {
        match self {
            RawValue::Integer(i) => Ok(*i),
            _ => Err(Error::from_database_field("Value is not an integer")),
        }
    }

    fn as_f64(&self) -> Result<f64> {
        match self {
            RawValue::Float(f) => Ok(*f),
            _ => Err(Error::from_database_field("Value is not a float")),
        }
    }

    fn as_bool(&self) -> Result<bool> {
        match self {
            RawValue::Boolean(b) => Ok(*b),
            _ => Err(Error::from_database_field("Value is not a boolean")),
        }
    }

    fn as_entity_reference(&self) -> Result<String> {
        match self {
            RawValue::EntityReference(e) => Ok(e.clone()),
            _ => Err(Error::from_database_field(
                "Value is not an entity reference",
            )),
        }
    }

    fn as_timestamp(&self) -> Result<DateTime<Utc>> {
        match self {
            RawValue::Timestamp(t) => Ok(*t),
            _ => Err(Error::from_database_field("Value is not a timestamp")),
        }
    }

    fn as_connection_state(&self) -> Result<String> {
        match self {
            RawValue::ConnectionState(c) => Ok(c.clone()),
            _ => Err(Error::from_database_field(
                "Value is not a connection state",
            )),
        }
    }

    fn as_garage_door_state(&self) -> Result<String> {
        match self {
            RawValue::GarageDoorState(g) => Ok(g.clone()),
            _ => Err(Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }

    fn update_str(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::String(s) => {
                *s = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a string")),
        }
    }

    fn update_i64(&mut self, value: i64) -> Result<()> {
        match self {
            RawValue::Integer(i) => {
                *i = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not an integer")),
        }
    }

    fn update_f64(&mut self, value: f64) -> Result<()> {
        match self {
            RawValue::Float(f) => {
                *f = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a float")),
        }
    }

    fn update_bool(&mut self, value: bool) -> Result<()> {
        match self {
            RawValue::Boolean(b) => {
                *b = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a boolean")),
        }
    }

    fn update_entity_reference(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::EntityReference(e) => {
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
            RawValue::Timestamp(t) => {
                *t = value;
                Ok(())
            }
            _ => Err(Error::from_database_field("Value is not a timestamp")),
        }
    }

    fn update_connection_state(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::ConnectionState(c) => {
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
            RawValue::GarageDoorState(g) => {
                *g = value;
                Ok(())
            }
            _ => Err(Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }
}

type ValueRef = Rc<RefCell<RawValue>>;

pub struct DatabaseValue(ValueRef);

impl DatabaseValue {
    pub fn new(value: RawValue) -> Self {
        DatabaseValue(Rc::new(RefCell::new(value)))
    }

    pub fn clone(&self) -> Self {
        DatabaseValue(self.0.clone())
    }

    pub fn into_raw(self) -> RawValue {
        self.0.borrow().clone()
    }

    pub fn as_str(&self) -> Result<String> {
        self.0.borrow().as_str()
    }

    pub fn as_i64(&self) -> Result<i64> {
        self.0.borrow().as_i64()
    }

    pub fn as_f64(&self) -> Result<f64> {
        self.0.borrow().as_f64()
    }

    pub fn as_bool(&self) -> Result<bool> {
        self.0.borrow().as_bool()
    }

    pub fn as_entity_reference(&self) -> Result<String> {
        self.0.borrow().as_entity_reference()
    }

    pub fn as_timestamp(&self) -> Result<DateTime<Utc>> {
        self.0.borrow().as_timestamp()
    }

    pub fn as_connection_state(&self) -> Result<String> {
        self.0.borrow().as_connection_state()
    }

    pub fn as_garage_door_state(&self) -> Result<String> {
        self.0.borrow().as_garage_door_state()
    }

    pub fn update_str(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_str(value)
    }

    pub fn update_i64(&self, value: i64) -> Result<()> {
        self.0.borrow_mut().update_i64(value)
    }

    pub fn update_f64(&self, value: f64) -> Result<()> {
        self.0.borrow_mut().update_f64(value)
    }

    pub fn update_bool(&self, value: bool) -> Result<()> {
        self.0.borrow_mut().update_bool(value)
    }

    pub fn update_entity_reference(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_entity_reference(value)
    }

    pub fn update_timestamp(&self, value: DateTime<Utc>) -> Result<()> {
        self.0.borrow_mut().update_timestamp(value)
    }

    pub fn update_connection_state(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_connection_state(value)
    }

    pub fn update_garage_door_state(&self, value: String) -> Result<()> {
        self.0.borrow_mut().update_garage_door_state(value)
    }
}

pub trait ClientTrait {
    fn connect(&mut self) -> Result<()>;
    fn connected(&self) -> bool;
    fn disconnect(&mut self) -> bool;
    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>>;
    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity>;
    fn get_notifications(&mut self) -> Result<Vec<DatabaseNotification>>;
    fn read(&mut self, requests: &Vec<DatabaseField>) -> Result<()>;
    fn register_notification(&mut self, config: &NotificationConfig) -> Result<NotificationToken>;
    fn unregister_notification(&mut self, token: &NotificationToken) -> Result<()>;
    fn write(&mut self, requests: &Vec<DatabaseField>) -> Result<()>;
}

type ClientRef = Rc<RefCell<dyn ClientTrait>>;
pub struct Client(ClientRef);

impl Client {
    pub fn new(client: impl ClientTrait + 'static) -> Self {
        Client(Rc::new(RefCell::new(client)))
    }

    pub fn clone(&self) -> Self {
        Client(self.0.clone())
    }

    pub fn connect(&self) -> Result<()> {
        self.0.borrow_mut().connect()
    }

    pub fn connected(&self) -> bool {
        self.0.borrow().connected()
    }

    pub fn disconnect(&self) -> bool {
        self.0.borrow_mut().disconnect()
    }

    pub fn get_entities(&self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        self.0.borrow_mut().get_entities(entity_type)
    }

    pub fn get_entity(&self, entity_id: &str) -> Result<DatabaseEntity> {
        self.0.borrow_mut().get_entity(entity_id)
    }

    pub fn get_notifications(&self) -> Result<Vec<DatabaseNotification>> {
        self.0.borrow_mut().get_notifications()
    }

    pub fn read(&self, requests: &Vec<DatabaseField>) -> Result<()> {
        self.0.borrow_mut().read(requests)
    }

    pub fn register_notification(&self, config: &NotificationConfig) -> Result<NotificationToken> {
        self.0.borrow_mut().register_notification(config)
    }

    pub fn unregister_notification(&self, token: &NotificationToken) -> Result<()> {
        self.0.borrow_mut().unregister_notification(token)
    }

    pub fn write(&self, requests: &Vec<DatabaseField>) -> Result<()> {
        self.0.borrow_mut().write(requests)
    }
}

pub mod rest;

pub struct _Database {
    client: Client,
    notification_manager: NotificationManager,
}

type DatabaseRef = Rc<RefCell<_Database>>;
pub struct Database(DatabaseRef);

impl Database {
    pub fn new(client: Client) -> Self {
        Database(Rc::new(RefCell::new(_Database::new(client))))
    }

    pub fn clone(&self) -> Self {
        Database(self.0.clone())
    }

    pub fn connect(&self) -> Result<()> {
        self.0.borrow().connect()
    }

    pub fn connected(&self) -> bool {
        self.0.borrow().connected()
    }

    pub fn disconnect(&self) -> bool {
        self.0.borrow().disconnect()
    }

    pub fn find(&self, entity_type: &str, field: &Vec<String>, predicate: fn(&HashMap<String, DatabaseField>) -> bool) -> Result<Vec<DatabaseEntity>> {
        self.0.borrow().find(entity_type, field, predicate)
    }

    pub fn get_entity(&self, entity_id: &str) -> Result<DatabaseEntity> {
        self.0.borrow().get_entity(entity_id)
    }

    pub fn get_entities(&self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        self.0.borrow().get_entities(entity_type)
    }

    pub fn read(&self, requests: &Vec<DatabaseField>) -> Result<()> {
        self.0.borrow().read(requests)
    }

    pub fn write(&self, requests: &Vec<DatabaseField>) -> Result<()> {
        self.0.borrow().write(requests)
    }

    pub fn clear_notifications(&self) {
        self.0.borrow().clear_notifications();
    }

    pub fn register_notification(&self, config: &NotificationConfig, callback: NotificationCallback) -> Result<NotificationToken> {
        self.0.borrow().register_notification(config, callback)
    }

    pub fn unregister_notification(&self, token: &NotificationToken) -> Result<()> {
        self.0.borrow().unregister_notification(token)
    }

    pub fn process_notifications(&self) -> Result<()> {
        self.0.borrow().process_notifications()
    }
}

impl _Database {
    pub fn new(client: Client) -> Self {
        _Database {
            client,
            notification_manager: NotificationManager::new(),
        }
    }
}

impl _Database {
    fn clear_notifications(&self) {
        self.notification_manager.clear();
    }

    fn connect(&self) -> Result<()> {
        return self.client.connect();
    }

    fn connected(&self) -> bool {
        self.client.connected()
    }

    fn disconnect(&self) -> bool {
        self.client.disconnect()
    }

    fn get_entity(&self, entity_id: &str) -> Result<DatabaseEntity> {
        self.client.get_entity(entity_id)
    }

    fn get_entities(&self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        self.client.get_entities(entity_type)
    }

    fn find(&self, entity_type: &str, fields: &Vec<String>, predicate: fn(&HashMap<String, DatabaseField>) -> bool) -> Result<Vec<DatabaseEntity>> {
        let entities = self.get_entities(entity_type)?;
        let mut result = vec![];

        for entity in &entities {
            let mut requests = vec![];

            for field in fields {
                let field = RawField::new(entity.entity_id.clone(), field.clone());
                requests.push(DatabaseField::new(field));
            }
            
            self.read(&mut requests)?;

            let mut fields_map = HashMap::new();
            for field in &requests {
                fields_map.insert(field.name(), field.clone());
            }

            if predicate(&fields_map) {
                result.push(entity.clone());
            }
        }

        Ok(result)
    }

    fn read(&self, requests: &Vec<DatabaseField>) -> Result<()> {
        self.client.read(requests)
    }

    fn write(&self, requests: &Vec<DatabaseField>) -> Result<()> {
        self.client.write(requests)
    }

    fn register_notification(&self, config: &NotificationConfig, callback: NotificationCallback) -> Result<NotificationToken> {
        self.notification_manager.register(self.client.clone(), config, callback)
    }

    fn unregister_notification(&self, token: &NotificationToken) -> Result<()> {
        self.notification_manager.unregister(self.client.clone(), token)
    }

    fn process_notifications(&self) -> Result<()> {
        return self.notification_manager.process_notifications(self.client.clone());
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

    pub fn call<T>(&mut self, args: &mut T)
    where
        F: FnMut(&mut T),
    {
        (self.callback)(args);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotToken(usize);

pub trait SignalTrait<F: FnMut(&mut T), T> {
    fn connect(&mut self, slot: Slot<F>) -> SlotToken;
    fn disconnect(&mut self, token: &SlotToken);
    fn emit(&mut self, args: &mut T);
}

pub struct Signal<F: FnMut(&mut T), T> {
    slots: HashMap<SlotToken, Slot<F>>,
    args: std::marker::PhantomData<T>,
}

impl<F: FnMut(&mut T), T> Signal<F, T> {
    pub fn new() -> Self {
        Signal {
            slots: HashMap::new(),
            args: std::marker::PhantomData,
        }
    }
}

impl<F: FnMut(&mut T), T> SignalTrait<F, T> for Signal<F, T> {
    fn connect(&mut self, slot: Slot<F>) -> SlotToken {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = SlotToken(COUNTER.fetch_add(1, Ordering::Relaxed));
        self.slots.insert(id, slot);
        id
    }

    fn disconnect(&mut self, id: &SlotToken) {
        self.slots.remove(id);
    }

    fn emit(&mut self, args: &mut T) {
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

pub type LoggerRef = Rc<RefCell<dyn LoggerTrait>>;
pub struct Logger(LoggerRef);

impl Logger {
    pub fn new(logger: impl LoggerTrait + 'static) -> Self {
        Logger(Rc::new(RefCell::new(logger)))
    }

    pub fn clone(&self) -> Self {
        Logger(self.0.clone())
    }

    pub fn log(&self, level: &LogLevel, message: &str) {
        self.0.borrow_mut().log(level, message);
    }

    pub fn trace(&self, message: &str) {
        self.0.borrow_mut().trace(message);
    }

    pub fn debug(&self, message: &str) {
        self.0.borrow_mut().debug(message);
    }

    pub fn info(&self, message: &str) {
        self.0.borrow_mut().info(message);
    }

    pub fn warning(&self, message: &str) {
        self.0.borrow_mut().warning(message);
    }

    pub fn error(&self, message: &str) {
        self.0.borrow_mut().error(message);
    }
}

pub trait ApplicationTrait {
    fn execute(&mut self, ctx: &mut ApplicationContext);
}

type _QuitFlag = Rc<RefCell<bool>>;
pub struct BoolFlag(_QuitFlag);

impl BoolFlag {
    pub fn new() -> Self {
        BoolFlag(Rc::new(RefCell::new(false)))
    }

    pub fn set(&self, value: bool) {
        *self.0.borrow_mut() = value;
    }

    pub fn get(&self) -> bool {
        *self.0.borrow()
    }
}

pub struct ApplicationContext {
    pub database: Database,
    pub logger: Logger,
    pub quit: BoolFlag,
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

            if !ctx.quit.get() {
                let loop_time = std::time::Duration::from_millis(self.loop_interval_ms);
                let elapsed_time = start.elapsed();
                
                if loop_time > elapsed_time {
                    let sleep_time = loop_time - start.elapsed();
                    std::thread::sleep(sleep_time);
                }
            }

            !ctx.quit.get()
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
    pub connected: Signal<fn(&mut ApplicationContext), ApplicationContext>,
    pub disconnected: Signal<fn(&mut ApplicationContext), ApplicationContext>,
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
                self.signals.disconnected.emit(ctx);
            }

            ctx.logger.log(&LogLevel::Debug, "[qdb::DatabaseWorker::do_work] Attempting to connect to the database...");
            
            ctx.database.disconnect();
            ctx.database.connect()?;

            if ctx.database.connected() {
                self.connected = true;
                ctx.logger.log(&LogLevel::Info, "[qdb::DatabaseWorker::do_work] Connected to the database");
                self.signals.connected.emit(ctx);
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