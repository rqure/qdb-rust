use super::Error;
use super::Result;
use super::DatabaseEntity;
use super::DatabaseField;
use super::DatabaseValue;
use super::ClientTrait;
use super::NotificationConfig;
use super::NotificationToken;
use super::DatabaseNotification;

use ureq::serde_json::Number;
use ureq::serde_json::Value;
use ureq::serde_json::Map;

use chrono::{DateTime, Utc, prelude};

pub struct Client {
    url: String,
    request_template: Map<String, Value>
}

impl Client {
    pub fn new(url: &str) -> Client {
        Client {
            url: url.to_string(),
            request_template: Map::new()
        }
    }

    fn authenticate(&mut self) -> Result<()> {
        let response = ureq::get(format!("{}/make-client-id", self.url).as_str())
            .call()
            .map_err(|e| Box::new(e))?
            .into_json()
            .map_err(|e| Box::new(e))?;

        match response {            
            Value::Object(client_id) => {
                self.request_template = client_id;
                Ok(())
            },
            _ => Err(Box::new(Error::ClientError("Invalid response from server".to_string())))
        }
    }

    fn has_authenticated(&self, js: &Value) -> bool {
        js
            .as_object()
            .and_then(|o| o.get("header"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("authenticationStatus"))
            .and_then(|v| v.as_str() )
            .and_then(|s| Some(s == "AUTHENTICATED"))
            .unwrap_or(false)
    }

    fn send(&mut self, payload: &Map<String, Value>) -> Result<Value> {
        let attempts = 3;
        let url = format!("{}/api", self.url);

        for _ in 0..attempts {
            let mut request = self.request_template.clone();
            request.insert("payload".to_string(), Value::Object(payload.clone()));

            let response = ureq::post(&url)
                .send_json(Value::Object(request.clone()))
                .map_err(|e| Box::new(e))?
                .into_json()
                .map_err(|e| Box::new(e))?;

            if self.has_authenticated(&response) {
                let response = response.get("payload")
                    .ok_or(Error::from_client("Invalid response from server: payload is not valid"))?;
                return Ok(response.clone());
            } else {
                self.authenticate()?;
            }
        }

        Err(Box::new(Error::ClientError("Failed to authenticate".to_string())))
    }

    fn extract_value(value: &Map<String, Value>) -> Result<DatabaseValue> {
        let value_type = value
            .get("@type")
            .and_then(|v| v.as_str())
            .ok_or(Error::from_client("Invalid response from server: value type is not valid"))?;

        let value = match value_type {
            "type.googleapis.com/qdb.String" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?
                    .to_string();
                DatabaseValue::String(value)
            },
            "type.googleapis.com/qdb.Int" => {                    
                let value = value
                    .get("raw")
                    // should be as i64 but it's a limitation with jsonpb marshaller on server side
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<i64>().ok() )
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?;
                DatabaseValue::Integer(value)
            },
            "type.googleapis.com/qdb.Float" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_f64())
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?;
                DatabaseValue::Float(value)
            },
            "type.googleapis.com/qdb.Bool" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_bool())
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?;
                DatabaseValue::Boolean(value)
            },
            "type.googleapis.com/qdb.EntityReference" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?
                    .to_string();
                DatabaseValue::EntityReference(value)
            },
            "type.googleapis.com/qdb.Timestamp" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?;
                let timestamp = DateTime::parse_from_rfc3339(value)?.to_utc();
                DatabaseValue::Timestamp(timestamp)
            },
            "type.googleapis.com/qdb.ConnectionState" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?
                    .to_string();
                DatabaseValue::ConnectionState(value)
            },
            "type.googleapis.com/qdb.GarageDoorState" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client("Invalid response from server: value is not valid"))?
                    .to_string();
                DatabaseValue::GarageDoorState(value)
            },
            _ => return Err(Error::from_client("Invalid response from server: value type is not valid"))
        };

        Ok(value)
    }
}

impl ClientTrait for Client {
    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity> {
        let mut request = Map::new();
        request.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebConfigGetEntityRequest".to_string()));
        request.insert("id".to_string(), Value::String(entity_id.to_string()));

        let response = self.send(&request)?;
        let entity = response
            .as_object()
            .and_then(|o| o.get("entity"))
            .and_then(|v| v.as_object())
            .ok_or(Error::from_client("Invalid response from server: Failed to extract entity"))?;

        Ok(DatabaseEntity{
            entity_id: entity
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client("Invalid response from server: entity id is not valid"))?
                .to_string(),
            entity_type: entity
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client("Invalid response from server: entity type is not valid"))?
                .to_string(),
            entity_name: entity
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client("Invalid response from server: entity name is not valid"))?
                .to_string()
        })
    }

    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        let mut request = Map::new();
        request.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeGetEntitiesRequest".to_string()));
        request.insert("entityType".to_string(), Value::String(entity_type.to_string()));

        let response = self.send(&request)?;
        let entities = response
            .as_object()
            .and_then(|o| o.get("entities"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client("Invalid response from server: Failed to extract entities"))?;

        let mut result = vec![];
        for entity in entities {
            match entity {
                Value::Object(entity) => {
                    result.push(DatabaseEntity{
                        entity_id: entity
                            .get("id")
                            .and_then(|v| v.as_str())
                            .ok_or(Error::from_client("Invalid response from server: entity id is not valid"))?
                            .to_string(),
                        entity_type: entity
                            .get("type")
                            .and_then(|v| v.as_str())
                            .ok_or(Error::from_client("Invalid response from server: entity type is not valid"))?
                            .to_string(),
                        entity_name: entity
                            .get("name")
                            .and_then(|v| v.as_str())
                            .ok_or(Error::from_client("Invalid response from server: entity name is not valid"))?
                            .to_string()
                    })
                },
                _ => return Err(Error::from_client("Invalid response from server: entity is not an object"))
            }
        }

        Ok(result)
    }

    fn read(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        let mut request = Map::new();
        request.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeDatabaseRequest".to_string()));
        request.insert("requestType".to_string(), Value::String("READ".to_string()));

        {
            let requests = Value::Array(requests.iter().map(|r| {
                let mut request = Map::new();
                request.insert("id".to_string(), Value::String(r.entity_id.clone()));
                request.insert("field".to_string(), Value::String(r.name.clone()));
                Value::Object(request)
            }).collect());
            request.insert("requests".to_string(), requests);
        }

        let response = self.send(&request)?;
        let entities = response
            .as_object()
            .and_then(|o| o.get("response"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client("Invalid response from server: response is not valid"))?;

        for entity in entities {
            match entity {
                Value::Object(entity) => {
                    println!("{:?}", entity);

                    let entity_id = entity
                        .get("id")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client("Invalid response from server: entity id is not valid"))?
                        .to_string();

                    let field_name = entity
                        .get("field")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client("Invalid response from server: field name is not valid"))?
                        .to_string();

                    let field = requests
                        .iter_mut()
                        .find(|r: &&mut DatabaseField| r.entity_id == entity_id && r.name == field_name)
                        .ok_or(Error::from_client("Invalid response from server: Field not found"))?;

                    let value = entity
                        .get("value")
                        .and_then(|v: &Value| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: value is not valid"))?;

                    let write_time = entity
                        .get("writeTime")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: write time is not valid"))?
                        .get("raw")
                        .ok_or(Error::from_client("Invalid response from server: write time is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: write time is not valid"))?;

                    let writer_id = entity
                        .get("writerId")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: writer id is not valid"))?
                        .get("raw")
                        .ok_or(Error::from_client("Invalid response from server: writer id is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: writer id is not valid"))?
                        .to_string();

                    field.value = Client::extract_value(value)?;
                    field.write_time = DateTime::parse_from_rfc3339(write_time)?.to_utc();
                    field.writer_id = writer_id;
                }
                _ => return Err(Box::new(Error::ClientError("Invalid response from server: response is not an object".to_string())))
            }
        }

        Ok(())
    }

    fn write(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        let mut request = Map::new();
        request.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeDatabaseRequest".to_string()));
        request.insert("requestType".to_string(), Value::String("WRITE".to_string()));

        {
            let requests = Value::Array(requests.iter().map(|r| {
                let mut request = Map::new();
                request.insert("id".to_string(), Value::String(r.entity_id.clone()));
                request.insert("field".to_string(), Value::String(r.name.clone()));
                let value = match &r.value {
                    DatabaseValue::String(s) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.String".to_string()));
                        value.insert("raw".to_string(), Value::String(s.clone()));
                        Value::Object(value)
                    },
                    DatabaseValue::Integer(i) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.Int".to_string()));
                        let n = Number::from(*i);
                        value.insert("raw".to_string(), Value::Number(n));
                        Value::Object(value)
                    },
                    DatabaseValue::Float(f) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.Float".to_string()));
                        let n = Number::from_f64(*f).unwrap_or(Number::from(0));
                        value.insert("raw".to_string(), Value::Number(n));
                        Value::Object(value)
                    },
                    DatabaseValue::Boolean(b) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.Bool".to_string()));
                        value.insert("raw".to_string(), Value::Bool(*b));
                        Value::Object(value)
                    },
                    DatabaseValue::EntityReference(e) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.EntityReference".to_string()));
                        value.insert("raw".to_string(), Value::String(e.clone()));
                        Value::Object(value)
                    },
                    DatabaseValue::Timestamp(t) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.Timestamp".to_string()));
                        let seconds = t.timestamp();
                        let nanos = t.timestamp_subsec_nanos();
                        let mut raw = Map::new();
                        raw.insert("seconds".to_string(), Value::Number(Number::from(seconds)));
                        raw.insert("nanos".to_string(), Value::Number(Number::from(nanos as i64)));
                        value.insert("raw".to_string(), Value::Object(raw));
                        Value::Object(value)
                    },
                    DatabaseValue::ConnectionState(c) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.ConnectionState".to_string()));
                        value.insert("raw".to_string(), Value::String(c.clone()));
                        Value::Object(value)
                    },
                    DatabaseValue::GarageDoorState(g) => {
                        let mut value = Map::new();
                        value.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.GarageDoorState".to_string()));
                        value.insert("raw".to_string(), Value::String(g.clone()));
                        Value::Object(value)
                    },
                    _ => Value::Null
                };
                request.insert("value".to_string(), value);
                Value::Object(request)
            }).collect());
            request.insert("requests".to_string(), requests);
        }

        self.send(&request)?;

        Ok(())
    }
    
    fn register_notification(&mut self, config: NotificationConfig) -> Result<NotificationToken> {
        let context = config.context.iter().map(|v| {
            Value::String(v.into())
        }).collect();

        let mut notification = Map::new();
        notification.insert("id".to_string(), Value::String(config.entity_id));
        notification.insert("type".to_string(), Value::String(config.entity_type));
        notification.insert("field".to_string(), Value::String(config.field));
        notification.insert("notifyOnChange".to_string(), Value::Bool(config.notify_on_change));
        notification.insert("context".to_string(), Value::Array(context));
        
        let mut request = Map::new();
        request.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeRegisterNotificationRequest".to_string()));
        request.insert("requests".to_string(), Value::Array(vec![Value::Object(notification)]));

        let response = self.send(&request)?;
        let token = response
            .as_object()
            .and_then(|o| o.get("tokens"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client("Invalid response from server: token is not valid"))?
            .get(0)
            .ok_or(Error::from_client("Invalid response from server: token is not valid"))?
            .as_str()
            .ok_or(Error::from_client("Invalid response from server: token is not valid"))?;

        Ok(NotificationToken(token.to_string()))
    }

    fn unregister_notification(&mut self, token: NotificationToken) -> Result<()> {
        let mut request = Map::new();
        request.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeUnregisterNotificationRequest".to_string()));
        request.insert("tokens".to_string(), Value::Array(vec![Value::String(token.into())]));

        self.send(&request)?;

        Ok(())
    }

    fn process_notifications(&mut self) -> Result<Vec<DatabaseNotification>> {
        let mut request = Map::new();
        request.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeGetNotificationsResponse".to_string()));

        let response = self.send(&request)?;
        let notifications = response
            .as_object()
            .and_then(|o| o.get("notifications"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?;

        let mut result = vec![];
        for notification in notifications {
            result.push(DatabaseNotification{
                token: notification.pointer("/token")
                    .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                    .as_str()
                    .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                    .into(),
                current: DatabaseField{
                    entity_id: notification.pointer("/current/id")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .into(),
                    name: notification.pointer("/current/name")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .into(),
                    write_time: DateTime::parse_from_rfc3339(notification.pointer("/current/writeTime")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .get("raw")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?)?
                        .to_utc(),
                    writer_id: notification.pointer("/current/writeTime")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .get("raw")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .to_string(),
                    value: Client::extract_value(notification.pointer("/current/value")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?)?
                },
                previous: DatabaseField{
                    entity_id: notification.pointer("/previous/id")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .into(),
                    name: notification.pointer("/previous/name")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .into(),
                    write_time: DateTime::parse_from_rfc3339(notification.pointer("/previous/writeTime")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .get("raw")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?)?
                        .to_utc(),
                    writer_id: notification.pointer("/previous/writerId")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .get("raw")
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .as_str()
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                        .to_string(),
                    value: Client::extract_value(notification.pointer("/previous/value")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?)?
                },
                context: notification.pointer("/context")
                    .and_then(|v| v.as_array())
                    .ok_or(Error::from_client("Invalid response from server: notifications is not valid"))?
                    .iter()
                    .map(|v| {
                        DatabaseField{
                            entity_id: v.pointer("/id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            name: v.pointer("/name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            write_time: DateTime::parse_from_rfc3339(v.pointer("/writeTime")
                                .and_then(|v| v.as_object())
                                .and_then(|v| v.get("raw"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string()
                                .as_str())
                                .unwrap_or_else(|_| Utc::now().fixed_offset() )
                                .to_utc(),
                            writer_id: v.pointer("/writerId")
                                .and_then(|v| v.as_object())
                                .and_then(|v| v.get("raw"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            value: Client::extract_value(v.pointer("/value")
                                .and_then(|v| v.as_object())
                                .unwrap_or(&Map::new()))
                                .unwrap_or(DatabaseValue::Unspecified)
                        }
                    })
                    .collect()
            });
        }

        Ok(result)
    }
}