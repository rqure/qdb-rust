use crate::Result;
use crate::schema::field::Field;
use crate::schema::entity::Entity;
use crate::schema::notification::{Notification, Config, Token};

pub trait ClientTrait {
    fn connect(&mut self) -> Result<()>;
    fn connected(&self) -> bool;
    fn disconnect(&mut self) -> bool;
    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<Entity>>;
    fn get_entity(&mut self, entity_id: &str) -> Result<Entity>;
    fn get_notifications(&mut self) -> Result<Vec<Notification>>;
    fn read(&mut self, requests: &Vec<Field>) -> Result<()>;
    fn register_notification(&mut self, config: &Config) -> Result<Token>;
    fn unregister_notification(&mut self, token: &Token) -> Result<()>;
    fn write(&mut self, requests: &Vec<Field>) -> Result<()>;
}