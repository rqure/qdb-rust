use std::result;

use super::Error;
use super::Result;
use super::DatabaseEntity;
use super::DatabaseField;
use super::ValueTrait;

pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    EntityReference(String),
    Timestamp(String),
    ConnectionState(String),
    GarageDoorState(String),
}

impl ValueTrait for Value {
    fn get<T>(&self) -> &T {
        match self {
            Value::String(s) => s,
            Value::Integer(i) => i,
            Value::Float(f) => f,
            Value::Boolean(b) => b,
            Value::EntityReference(e) => e,
            Value::Timestamp(t) => t,
            Value::ConnectionState(c) => c,
            Value::GarageDoorState(g) => g,
        }
    }

    fn set<T>(&mut self, value: T) {
        match self {
            Value::String(s) => *s = value,
            Value::Integer(i) => *i = value,
            Value::Float(f) => *f = value,
            Value::Boolean(b) => *b = value,
            Value::EntityReference(e) => *e = value,
            Value::Timestamp(t) => *t = value,
            Value::ConnectionState(c) => *c = value,
            Value::GarageDoorState(g) => *g = value,
        }
    }
}

pub struct Client {
    url: String,
    request_template: ureq::serde_json::Map<String, ureq::serde_json::Value>
}

impl Client {
    pub fn new<T>(url: &T) -> Client {
        Client {
            url: String::from(url),
            request_template: ureq::serde_json::Map::new()
        }
    }

    fn authenticate(&mut self) -> Result<()> {
        let response = ureq::get(format!("{}/make-client-id", self.url).as_str())
            .call()
            .map_err(|e| Box::new(e))?
            .into_json()
            .map_err(|e| Box::new(e))?;

        match response {
            ureq::serde_json::Value::Object(client_id) => {
                self.request_template = client_id;
                Ok(())
            },
            _ => Err(Box::new(Error::ClientError(String::from("Invalid response from server"))))
        }
    }

    fn has_authenticated(&self, js: &ureq::serde_json::Value) -> bool {
        js
            .as_object()
            .and_then(|o| o.get("header"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("authenticationStatus"))
            .and_then(|v| v.as_str() )
            .and_then(|s| Some(s == "AUTHENTICATED"))
            .unwrap_or(false)
    }

    fn send(&self, request: ureq::serde_json::Map<String, ureq::serde_json::Value>) -> Result<ureq::serde_json::Value> {
        let attempts = 3;

        for n in 0..attempts {
            let response = ureq::post(format!("{}/api", self.url).as_str())
                .send_json(ureq::serde_json::Value::Object(request))
                .map_err(|e| Box::new(e))?
                .into_json()
                .map_err(|e| Box::new(e))?;

            if self.has_authenticated(&response) {
                Ok(response)
            } else {
                self.authenticate()?;
            }
        }

        Err(Error::ClientError(String::from("Failed to authenticate")))
    }
}

impl ClientTrait for Client {
    fn get_entity(&self, entity_id: &str) -> Result<DatabaseEntity> {
        let request = self.request_template.clone();
        let payload = ureq::serde_json::Map::new();
        payload["@type"] = "type.googleapis.com/qdb.WebRuntimeGetEntityRequest";
        payload["entityId"] = entity_id;
        request["payload"] = payload;

        let entity = self.send(request)?
            .as_object()
            .and_then(|o| o.get("payload"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("entity"))
            .and_then(|v| v.as_object())?;

        if !entity.contains_key("id") {
            return Err(Box::new(Error::ClientError(String::from("Invalid response from server: no id key"))))
        }

        if !entity.contains_key("type") {
            return Err(Box::new(Error::ClientError(String::from("Invalid response from server: no type key"))))
        }

        if !entity.contains_key("name") {
            return Err(Box::new(Error::ClientError(String::from("Invalid response from server: no name key"))))
        }

        Ok(DatabaseEntity{
            entity_id: entity["id"],
            entity_type: entity["type"],
            entity_name: entity["name"]
        })
    }

    fn get_entities(&self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        let request = self.request_template.clone();
        let payload = ureq::serde_json::Map::new();
        payload["@type"] = "type.googleapis.com/qdb.WebRuntimeGetEntitiesRequest";
        payload["entityType"] = entity_type;
        request["payload"] = payload;

        let entities = self.send(request)?
            .as_object()
            .and_then(|o| o.get("payload"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("entity"))
            .and_then(|v| v.as_array())?;

        let result = vec![];
        for entity in entities {
            match entity {
                ureq::serde_json::Value::Object(entity) => {
                    if !entity.contains_key("id") {
                        return Err(Box::new(Error::ClientError(String::from("Invalid response from server: no id key"))))
                    }

                    if !entity.contains_key("type") {
                        return Err(Box::new(Error::ClientError(String::from("Invalid response from server: no type key"))))
                    }

                    if !entity.contains_key("name") {
                        return Err(Box::new(Error::ClientError(String::from("Invalid response from server: no name key"))))
                    }

                    result.push(DatabaseEntity{
                        entity_id: entity["id"],
                        entity_type: entity["type"],
                        entity_name: entity["name"]
                    })
                },
                _ => return Err(Box::new(Error::ClientError(String::from("Invalid response from server: entity is not an object"))))
            }
        }

        Ok(result)
    }

    fn read(&self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        let request = self.request_template.clone();
        let payload = ureq::serde_json::Map::new();
        payload["@type"] = "type.googleapis.com/qdb.WebRuntimeDatabaseRequest";
        payload["requestType"] = "READ";

        let requests = ureq::serde_json::Value::Array(requests.iter().map(|r| {
            let request = ureq::serde_json::Map::new();
            request["id"] = r.entity_id;
            request["field"] = r.field;
            request
        }).collect());

        request["payload"] = payload;

        self.send(request)?;
        Ok(())
    }

    fn write(&self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        
    }
    
    fn register_notification(&self, config: NotificationConfig) -> Result<NotificationToken> {
        
    }

    fn unregister_notification(&self, token: NotificationToken) -> Result<()> {
        
    }

    fn process_notifications(&self) -> Result<Vec<DatabaseNotification>> {
        
    }
}