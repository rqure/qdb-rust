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