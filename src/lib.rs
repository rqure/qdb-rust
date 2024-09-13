pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;

mod error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatabaseEntity {
    id: String,
    type_name: String,
    name: String,
}

impl DatabaseEntity {
    pub fn new(id: &str, type_name: &str, name: &str) -> Self {
        DatabaseEntity {
            id: id.into(),
            type_name: type_name.into(),
            name: name.into(),
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn type_name(&self) -> String {
        self.type_name.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn update_id(&mut self, id: &str) {
        self.id = id.into();
    }

    pub fn update_type_name(&mut self, type_name: &str) {
        self.type_name = type_name.into();
    }

    pub fn update_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn field(&self, name: &str) -> DatabaseField {
        DatabaseField::new(RawField::new(self.id(), name))
    }
}

pub type FieldRef = Rc<RefCell<RawField>>;

pub struct DatabaseField(FieldRef);

impl Clone for DatabaseField {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl DatabaseField {
    pub fn new(field: RawField) -> Self {
        Self(Rc::new(RefCell::new(field)))
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

    pub fn set_str_value(&self, value: String) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::String(value)));
        self
    }

    pub fn set_i64_value(&self, value: i64) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Integer(value)));
        self
    }

    pub fn set_f64_value(&self, value: f64) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Float(value)));
        self
    }

    pub fn set_bool_value(&self, value: bool) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Boolean(value)));
        self
    }

    pub fn set_entity_reference_value(&self, value: String) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::EntityReference(value)));
        self
    }

    pub fn set_timestamp_value(&self, value: DateTime<Utc>) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::Timestamp(value)));
        self
    }

    pub fn set_connection_state_value(&self, value: String) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::ConnectionState(value)));
        self
    }

    pub fn set_garage_door_state_value(&self, value: String) -> &Self {
        self.0
            .borrow_mut()
            .update_value(DatabaseValue::new(RawValue::GarageDoorState(value)));
        self
    }

    pub fn set_unspecified_value(&self) -> &Self {
        self.0.borrow_mut().update_value(DatabaseValue::new(RawValue::Unspecified));
        self
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
    pub fn entity_id(&self) -> String {
        self.entity_id.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn value(&self) -> DatabaseValue {
        self.value.clone()
    }

    pub fn write_time(&self) -> DateTime<Utc> {
        self.write_time
    }

    pub fn writer_id(&self) -> String {
        self.writer_id.clone()
    }

    pub fn update_entity_id(&mut self, entity_id: &str) {
        self.entity_id = entity_id.into();
    }

    pub fn update_value(&mut self, value: DatabaseValue) {
        self.value = value;
    }

    pub fn update_write_time(&mut self, write_time: DateTime<Utc>) {
        self.write_time = write_time;
    }

    pub fn update_writer_id(&mut self, writer_id: &str) {
        self.writer_id = writer_id.into();
    }

    pub fn update_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn new(entity_id: impl Into<String>, field: impl Into<String>) -> Self {
        RawField {
            entity_id: entity_id.into(),
            name: field.into(),
            value: DatabaseValue::new(RawValue::Unspecified),
            write_time: Utc::now(),
            writer_id: "".to_string(),
        }
    }

    pub fn new_with_value(
        entity_id: impl Into<String>,
        field: impl Into<String>,
        value: RawValue,
    ) -> Self {
        RawField {
            entity_id: entity_id.into(),
            name: field.into(),
            value: DatabaseValue::new(value),
            write_time: Utc::now(),
            writer_id: "".to_string(),
        }
    }

    pub fn into_field(self) -> DatabaseField {
        DatabaseField::new(self)
    }
}

#[derive(Clone)]
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

pub struct _NotificationManager {
    registered_config: HashSet<NotificationConfig>,
    config_to_token: HashMap<NotificationConfig, NotificationToken>,
    token_to_callback_list: HashMap<NotificationToken, EventEmitter<DatabaseNotification>>,
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

    pub fn register(
        &self,
        client: Client,
        config: &NotificationConfig,
    ) -> Result<Receiver<DatabaseNotification>> {
        self.0.borrow_mut().register(client, config)
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
            token_to_callback_list: HashMap::new(),
        }
    }
}

impl _NotificationManager {
    fn clear(&mut self) {
        self.registered_config.clear();
        self.config_to_token.clear();
        self.token_to_callback_list.clear();
    }

    fn register(
        &mut self,
        client: Client,
        config: &NotificationConfig,
    ) -> Result<Receiver<DatabaseNotification>> {
        if self.registered_config.contains(&config) {
            let token = self
                .config_to_token
                .get(config)
                .ok_or(error::Error::from_notification(
                    "Inconsistent notification state during registration",
                ))?;

            let receiver = self
                .token_to_callback_list
                .get_mut(token)
                .ok_or(error::Error::from_notification(
                    "Inconsistent notification state during registration",
                ))?
                .new_receiver();

            return Ok(receiver);
        }

        let token = client.register_notification(config)?;

        self.registered_config.insert(config.clone());
        self.config_to_token.insert(config.clone(), token.clone());
        self.token_to_callback_list
            .insert(token.clone(), EventEmitter::new());

        let receiver = self
            .token_to_callback_list
            .get_mut(&token)
            .ok_or(error::Error::from_notification(
                "Inconsistent notification state during registration",
            ))?
            .new_receiver();

        Ok(receiver)
    }

    fn unregister(&mut self, client: Client, token: &NotificationToken) -> Result<()> {
        if !self.token_to_callback_list.contains_key(token) {
            return Err(error::Error::from_notification(
                "Token not found during unregistration",
            ));
        }

        client.unregister_notification(token)?;

        self.token_to_callback_list.remove(token);
        self.config_to_token.retain(|_, v| v != token);
        self.registered_config
            .retain(|c| self.config_to_token.contains_key(c));

        Ok(())
    }

    fn process_notifications(&mut self, client: Client) -> Result<()> {
        let notifications = client.get_notifications()?;

        for notification in &notifications {
            let token = NotificationToken(notification.token.clone());
            let emitter =
                self.token_to_callback_list
                    .get_mut(&token)
                    .ok_or(error::Error::from_notification(
                        "Cannot process notification: Callback list doesn't exist for token",
                    ))?;
            emitter.emit(notification.clone());
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
    pub fn into_value(self) -> DatabaseValue {
        DatabaseValue::new(self)
    }

    pub fn as_str(&self) -> Result<String> {
        match self {
            RawValue::String(s) => Ok(s.clone()),
            _ => Err(error::Error::from_database_field("Value is not a string")),
        }
    }

    pub fn as_i64(&self) -> Result<i64> {
        match self {
            RawValue::Integer(i) => Ok(*i),
            _ => Err(error::Error::from_database_field("Value is not an integer")),
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        match self {
            RawValue::Float(f) => Ok(*f),
            _ => Err(error::Error::from_database_field("Value is not a float")),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            RawValue::Boolean(b) => Ok(*b),
            _ => Err(error::Error::from_database_field("Value is not a boolean")),
        }
    }

    pub fn as_entity_reference(&self) -> Result<String> {
        match self {
            RawValue::EntityReference(e) => Ok(e.clone()),
            _ => Err(error::Error::from_database_field(
                "Value is not an entity reference",
            )),
        }
    }

    pub fn as_timestamp(&self) -> Result<DateTime<Utc>> {
        match self {
            RawValue::Timestamp(t) => Ok(*t),
            _ => Err(error::Error::from_database_field("Value is not a timestamp")),
        }
    }

    pub fn as_connection_state(&self) -> Result<String> {
        match self {
            RawValue::ConnectionState(c) => Ok(c.clone()),
            _ => Err(error::Error::from_database_field(
                "Value is not a connection state",
            )),
        }
    }

    pub fn as_garage_door_state(&self) -> Result<String> {
        match self {
            RawValue::GarageDoorState(g) => Ok(g.clone()),
            _ => Err(error::Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }

    pub fn update_str(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::String(s) => {
                *s = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a string")),
        }
    }

    pub fn update_i64(&mut self, value: i64) -> Result<()> {
        match self {
            RawValue::Integer(i) => {
                *i = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not an integer")),
        }
    }

    pub fn update_f64(&mut self, value: f64) -> Result<()> {
        match self {
            RawValue::Float(f) => {
                *f = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a float")),
        }
    }

    pub fn update_bool(&mut self, value: bool) -> Result<()> {
        match self {
            RawValue::Boolean(b) => {
                *b = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a boolean")),
        }
    }

    pub fn update_entity_reference(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::EntityReference(e) => {
                *e = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field(
                "Value is not an entity reference",
            )),
        }
    }

    pub fn update_timestamp(&mut self, value: DateTime<Utc>) -> Result<()> {
        match self {
            RawValue::Timestamp(t) => {
                *t = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field("Value is not a timestamp")),
        }
    }

    pub fn update_connection_state(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::ConnectionState(c) => {
                *c = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field(
                "Value is not a connection state",
            )),
        }
    }

    pub fn update_garage_door_state(&mut self, value: String) -> Result<()> {
        match self {
            RawValue::GarageDoorState(g) => {
                *g = value;
                Ok(())
            }
            _ => Err(error::Error::from_database_field(
                "Value is not a garage door state",
            )),
        }
    }

    pub fn set_str(&mut self, value: String) {
        *self = RawValue::String(value);
    }

    pub fn set_i64(&mut self, value: i64) {
        *self = RawValue::Integer(value);
    }

    pub fn set_f64(&mut self, value: f64) {
        *self = RawValue::Float(value);
    }

    pub fn set_bool(&mut self, value: bool) {
        *self = RawValue::Boolean(value);
    }

    pub fn set_entity_reference(&mut self, value: String) {
        *self = RawValue::EntityReference(value);
    }

    pub fn set_timestamp(&mut self, value: DateTime<Utc>) {
        *self = RawValue::Timestamp(value);
    }

    pub fn set_connection_state(&mut self, value: String) {
        *self = RawValue::ConnectionState(value);
    }

    pub fn set_garage_door_state(&mut self, value: String) {
        *self = RawValue::GarageDoorState(value);
    }

    pub fn set_unspecified(&mut self) {
        *self = RawValue::Unspecified;
    }

    pub fn is_unspecified(&self) -> bool {
        matches!(self, RawValue::Unspecified)
    }

    pub fn is_str(&self) -> bool {
        matches!(self, RawValue::String(_))
    }

    pub fn is_i64(&self) -> bool {
        matches!(self, RawValue::Integer(_))
    }

    pub fn is_f64(&self) -> bool {
        matches!(self, RawValue::Float(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, RawValue::Boolean(_))
    }

    pub fn is_entity_reference(&self) -> bool {
        matches!(self, RawValue::EntityReference(_))
    }

    pub fn is_timestamp(&self) -> bool {
        matches!(self, RawValue::Timestamp(_))
    }

    pub fn is_connection_state(&self) -> bool {
        matches!(self, RawValue::ConnectionState(_))
    }

    pub fn is_garage_door_state(&self) -> bool {
        matches!(self, RawValue::GarageDoorState(_))
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

    pub fn set_str(&self, value: String) {
        self.0.borrow_mut().set_str(value)
    }

    pub fn set_i64(&self, value: i64) {
        self.0.borrow_mut().set_i64(value)
    }

    pub fn set_f64(&self, value: f64) {
        self.0.borrow_mut().set_f64(value)
    }

    pub fn set_bool(&self, value: bool) {
        self.0.borrow_mut().set_bool(value)
    }

    pub fn set_entity_reference(&self, value: String) {
        self.0.borrow_mut().set_entity_reference(value)
    }

    pub fn set_timestamp(&self, value: DateTime<Utc>) {
        self.0.borrow_mut().set_timestamp(value)
    }

    pub fn set_connection_state(&self, value: String) {
        self.0.borrow_mut().set_connection_state(value)
    }

    pub fn set_garage_door_state(&self, value: String) {
        self.0.borrow_mut().set_garage_door_state(value)
    }

    pub fn set_unspecified(&self) {
        self.0.borrow_mut().set_unspecified()
    }

    pub fn is_unspecified(&self) -> bool {
        self.0.borrow().is_unspecified()
    }

    pub fn is_str(&self) -> bool {
        self.0.borrow().is_str()
    }

    pub fn is_i64(&self) -> bool {
        self.0.borrow().is_i64()
    }

    pub fn is_f64(&self) -> bool {
        self.0.borrow().is_f64()
    }

    pub fn is_bool(&self) -> bool {
        self.0.borrow().is_bool()
    }

    pub fn is_entity_reference(&self) -> bool {
        self.0.borrow().is_entity_reference()
    }

    pub fn is_timestamp(&self) -> bool {
        self.0.borrow().is_timestamp()
    }

    pub fn is_connection_state(&self) -> bool {
        self.0.borrow().is_connection_state()
    }

    pub fn is_garage_door_state(&self) -> bool {
        self.0.borrow().is_garage_door_state()
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

    pub fn find(
        &self,
        entity_type: &str,
        field: &Vec<String>,
        predicate: fn(&HashMap<String, DatabaseField>) -> bool,
    ) -> Result<Vec<DatabaseEntity>> {
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

    pub fn register_notification(
        &self,
        config: &NotificationConfig,
    ) -> Result<Receiver<DatabaseNotification>> {
        self.0.borrow().register_notification(config)
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

    fn find(
        &self,
        entity_type: &str,
        fields: &Vec<String>,
        predicate: fn(&HashMap<String, DatabaseField>) -> bool,
    ) -> Result<Vec<DatabaseEntity>> {
        let entities = self.get_entities(entity_type)?;
        let mut result = vec![];

        for entity in &entities {
            let mut requests = vec![];

            for field in fields {
                let field = RawField::new(entity.id.clone(), field.clone());
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

    fn register_notification(
        &self,
        config: &NotificationConfig,
    ) -> Result<Receiver<DatabaseNotification>> {
        self.notification_manager
            .register(self.client.clone(), config)
    }

    fn unregister_notification(&self, token: &NotificationToken) -> Result<()> {
        self.notification_manager
            .unregister(self.client.clone(), token)
    }

    fn process_notifications(&self) -> Result<()> {
        return self
            .notification_manager
            .process_notifications(self.client.clone());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotToken(usize);

pub struct EventEmitter<T> {
    senders: HashMap<SlotToken, Sender<T>>,
    args: std::marker::PhantomData<T>,
}

impl<T> EventEmitter<T> {
    pub fn new() -> Self {
        EventEmitter {
            senders: HashMap::new(),
            args: std::marker::PhantomData,
        }
    }
}

impl<T: Clone> EventEmitter<T> {
    pub fn connect(&mut self, sender: Sender<T>) -> SlotToken {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = SlotToken(COUNTER.fetch_add(1, Ordering::Relaxed));
        self.senders.insert(id, sender);
        id
    }

    pub fn disconnect(&mut self, id: &SlotToken) {
        self.senders.remove(id);
    }

    pub fn new_receiver(&mut self) -> Receiver<T> {
        let (sender, receiver) = channel();
        self.connect(sender);
        receiver
    }

    pub fn emit(&mut self, args: T) {
        self.senders
            .retain(|_, sender| sender.send(args.clone()).is_ok());
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
    fn execute(&mut self);
}

type _BoolFlag = Rc<RefCell<bool>>;
pub struct BoolFlag(_BoolFlag);

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

impl Clone for BoolFlag {
    fn clone(&self) -> Self {
        BoolFlag(self.0.clone())
    }
}

struct _ApplicationContext {
    pub database: Database,
    pub logger: Logger,
    pub quit: BoolFlag,
}

type ApplicationContextRef = Rc<RefCell<_ApplicationContext>>;
pub struct ApplicationContext(ApplicationContextRef);

impl ApplicationContext {
    pub fn new(database: Database, logger: Logger) -> Self {
        ApplicationContext(Rc::new(RefCell::new(_ApplicationContext {
            database,
            logger,
            quit: BoolFlag::new(),
        })))
    }

    pub fn database(&self) -> Database {
        self.0.borrow().database.clone()
    }

    pub fn logger(&self) -> Logger {
        self.0.borrow().logger.clone()
    }

    pub fn quit(&self) -> BoolFlag {
        self.0.borrow().quit.clone()
    }
}

impl Clone for ApplicationContext {
    fn clone(&self) -> Self {
        ApplicationContext(self.0.clone())
    }
}

pub trait WorkerTrait {
    fn intialize(&mut self, ctx: ApplicationContext) -> Result<()>;
    fn do_work(&mut self, ctx: ApplicationContext) -> Result<()>;
    fn deinitialize(&mut self, ctx: ApplicationContext) -> Result<()>;
    fn process_events(&mut self) -> Result<()>;
}

pub struct Application {
    ctx: ApplicationContext,
    workers: Vec<Box<dyn WorkerTrait>>,
    loop_interval_ms: u64,
}

impl Application {
    pub fn new(ctx: ApplicationContext, loop_interval_ms: u64) -> Self {
        Self {
            ctx,
            workers: vec![],
            loop_interval_ms,
        }
    }
}

impl WorkerTrait for Application {
    fn intialize(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::initialize] Initializing application",
        );
        for worker in &mut self.workers {
            match worker.intialize(ctx.clone()) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger().error(&format!(
                        "[qdb::Application::initialize] Error while initializing worker: {}",
                        e
                    ));
                }
            }
        }

        Ok(())
    }

    fn do_work(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::do_work] Application has started",
        );

        while {
            let start = Instant::now();

            for i in 0..self.workers.len() {
                let worker = &mut self.workers[i];
                match worker.do_work(ctx.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        ctx.logger().error(&format!(
                            "[qdb::Application::do_work] Error while executing worker: {}",
                            e
                        ));
                    }
                }

                match self.process_events() {
                    Ok(_) => {}
                    Err(e) => {
                        ctx.logger().error(&format!(
                            "[qdb::Application::do_work] Error while processing events: {}",
                            e
                        ));
                    }
                }
            }

            if !ctx.quit().get() {
                let loop_time = std::time::Duration::from_millis(self.loop_interval_ms);
                let elapsed_time = start.elapsed();

                if loop_time > elapsed_time {
                    let sleep_time = loop_time - start.elapsed();
                    std::thread::sleep(sleep_time);
                }
            }

            !ctx.quit().get()
        } {}

        Ok(())
    }

    fn deinitialize(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::deinitialize] Deinitializing application",
        );

        for worker in &mut self.workers {
            match worker.deinitialize(ctx.clone()) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger().error(&format!(
                        "[qdb::Application::deinitialize] Error while deinitializing worker: {}",
                        e
                    ));
                }
            }
        }

        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::deinitialize] Shutting down now",
        );
        Ok(())
    }

    fn process_events(&mut self) -> Result<()> {
        for worker in &mut self.workers {
            match worker.process_events() {
                Ok(_) => {}
                Err(e) => {
                    self.ctx.logger().error(&format!(
                        "[qdb::Application::process_events] Error while processing events: {}",
                        e
                    ));
                }
            }
        }

        Ok(())
    }
}

impl ApplicationTrait for Application {
    fn execute(&mut self) {
        self.intialize(self.ctx.clone()).unwrap();
        self.do_work(self.ctx.clone()).unwrap();
        self.deinitialize(self.ctx.clone()).unwrap();
    }
}

impl Application {
    pub fn add_worker(&mut self, worker: Box<dyn WorkerTrait>) {
        self.workers.push(worker);
    }
}

pub struct DatabaseWorkerEventEmitters {
    pub connection_status: EventEmitter<bool>,
}

pub struct DatabaseWorker {
    is_db_connected: bool,
    is_nw_connected: bool,
    pub emitters: DatabaseWorkerEventEmitters,
    pub network_connection_events: Option<Receiver<bool>>,
}

impl DatabaseWorker {
    pub fn new() -> Self {
        Self {
            is_db_connected: false,
            is_nw_connected: false,
            emitters: DatabaseWorkerEventEmitters {
                connection_status: EventEmitter::new(),
            },
            network_connection_events: None,
        }
    }
}

impl WorkerTrait for DatabaseWorker {
    fn intialize(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::DatabaseWorker::initialize] Initializing database worker",
        );
        Ok(())
    }

    fn do_work(&mut self, ctx: ApplicationContext) -> Result<()> {
        if !self.is_nw_connected {
            if self.is_db_connected {
                ctx.logger().log(&LogLevel::Warning, "[qdb::DatabaseWorker::do_work] Network connection loss has disrupted database connection");
                self.is_db_connected = false;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            return Ok(());
        }

        if !ctx.database().connected() {
            if self.is_db_connected {
                ctx.logger().log(
                    &LogLevel::Warning,
                    "[qdb::DatabaseWorker::do_work] Disconnected from database",
                );
                ctx.database().clear_notifications();
                self.is_db_connected = false;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            ctx.logger().log(
                &LogLevel::Debug,
                "[qdb::DatabaseWorker::do_work] Attempting to connect to the database...",
            );

            ctx.database().disconnect();
            ctx.database().connect()?;

            if ctx.database().connected() {
                ctx.logger().log(
                    &LogLevel::Info,
                    "[qdb::DatabaseWorker::do_work] Connected to the database",
                );
                self.is_db_connected = true;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            return Ok(());
        }

        ctx.database().process_notifications()?;

        Ok(())
    }

    fn deinitialize(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::DatabaseWorker::deinitialize] Deinitializing database worker",
        );
        Ok(())
    }

    fn process_events(&mut self) -> Result<()> {
        if let Some(receiver) = &self.network_connection_events {
            while let Ok(connected) = receiver.try_recv() {
                self.is_nw_connected = connected;
            }
        }

        Ok(())
    }
}
