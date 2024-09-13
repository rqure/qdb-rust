use std::cell::RefCell;
use std::rc::Rc;

use crate::clients::common::ClientTrait;
use crate::Result;
use crate::schema::entity::DatabaseEntity;
use crate::schema::field::DatabaseField;
use crate::schema::notification::{DatabaseNotification, NotificationConfig, NotificationToken};

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