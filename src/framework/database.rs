use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use crate::framework::client::Client;
use crate::framework::notification::NotificationManager;
use crate::Result;
use crate::schema::field::{DatabaseField, RawField};
use crate::schema::notification::{DatabaseNotification, NotificationConfig, NotificationToken};
use crate::schema::entity::DatabaseEntity;

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