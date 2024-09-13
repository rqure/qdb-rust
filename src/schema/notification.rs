use crate::schema::field::Field;

#[derive(Clone)]
pub struct Notification {
    pub token: String,
    pub current: Field,
    pub previous: Field,
    pub context: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub entity_id: String,
    pub entity_type: String,
    pub field: String,
    pub notify_on_change: bool,
    pub context: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Token(String);

impl Into<String> for &Token {
    fn into(self) -> String {
        self.0.clone()
    }
}

impl From<String> for Token {
    fn from(s: String) -> Self {
        Token(s)
    }
}

impl From<&str> for Token {
    fn from(s: &str) -> Self {
        Token(s.to_string())
    }
}
