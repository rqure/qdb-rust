use crate::schema::field::DatabaseField;

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
