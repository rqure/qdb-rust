use super::ClientTrait;
use super::DatabaseEntity;
use super::RawField;
use super::DatabaseNotification;
use super::RawValue;
use super::Error;
use super::DatabaseField;
use super::DatabaseValue;
use super::NotificationConfig;
use super::NotificationToken;
use super::Result;

use ureq::serde_json::Map;
use ureq::serde_json::Number;
use ureq::serde_json::Value;

use chrono::{prelude, DateTime, Utc};

pub struct Client {
    auth_failure: bool,
    endpoint_reachable: bool,
    request_template: Map<String, Value>,
    url: String,
}

impl Client {
    pub fn new(url: &str) -> super::Client {
        super::Client::new(Client {
            auth_failure: false,
            endpoint_reachable: false,
            url: url.to_string(),
            request_template: Map::new(),
        })
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
            }
            _ => Err(Box::new(Error::ClientError(
                "Invalid response from server".to_string(),
            ))),
        }
    }

    fn has_authenticated(&self, js: &Value) -> bool {
        js.as_object()
            .and_then(|o| o.get("header"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("authenticationStatus"))
            .and_then(|v| v.as_str())
            .and_then(|s| Some(s == "AUTHENTICATED"))
            .unwrap_or(false)
    }

    fn parse_database_field(&self, notification: &Value, prefix: &str) -> Result<RawField> {
        let entity_id = notification
            .pointer(&format!("{}/id", prefix))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::from_client("Invalid response from server: entity ID is not valid")
            })?
            .to_string();

        let name = notification
            .pointer(&format!("{}/name", prefix))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::from_client("Invalid response from server: name is not valid")
            })?
            .to_string();

        let write_time = DateTime::parse_from_rfc3339(
            notification
                .pointer(&format!("{}/writeTime", prefix))
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    Error::from_client("Invalid response from server: writeTime is not valid")
                })?,
        )?
        .with_timezone(&Utc);

        let writer_id = notification
            .pointer(&format!("{}/writerId", prefix))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let value = Client::extract_value(
            notification
                .pointer(&format!("{}/value", prefix))
                .and_then(|v| v.as_object())
                .unwrap_or(&Map::new()),
        )
        .unwrap_or(RawValue::Unspecified.into_value());

        Ok(RawField {
            entity_id,
            name,
            write_time,
            writer_id,
            value,
        })
    }

    fn send(&mut self, payload: &Map<String, Value>) -> Result<Value> {
        let url = format!("{}/api", self.url);
        self.endpoint_reachable = false;
        
        let mut request = self.request_template.clone();
        request.insert("payload".to_string(), Value::Object(payload.clone()));

        let response = ureq::post(&url)
            .send_json(Value::Object(request.clone()))
            .map_err(|e| Box::new(e))?
            .into_json()
            .map_err(|e| Box::new(e))?;

        if !self.has_authenticated(&response) {
            self.auth_failure = true;

            return Err(Error::from_client("Failed to authenticate"));
        }

        let response = response.get("payload").ok_or(Error::from_client(
            "Invalid response from server: payload is not valid",
        ))?;

        self.endpoint_reachable = true;
        
        return Ok(response.clone());
    }

    fn extract_value(value: &Map<String, Value>) -> Result<DatabaseValue> {
        let value_type = value
            .get("@type")
            .and_then(|v| v.as_str())
            .ok_or(Error::from_client(
                "Invalid response from server: value type is not valid",
            ))?;

        let value = match value_type {
            "type.googleapis.com/qdb.String" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client(
                        "Invalid response from server: value is not valid",
                    ))?
                    .to_string();
                RawValue::String(value)
            }
            "type.googleapis.com/qdb.Int" => {
                let value = value
                    .get("raw")
                    // should be as i64 but it's a limitation with jsonpb marshaller on server side
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<i64>().ok())
                    .ok_or(Error::from_client(
                        "Invalid response from server: value is not valid",
                    ))?;
                RawValue::Integer(value)
            }
            "type.googleapis.com/qdb.Float" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_f64())
                    .ok_or(Error::from_client(
                        "Invalid response from server: value is not valid",
                    ))?;
                RawValue::Float(value)
            }
            "type.googleapis.com/qdb.Bool" => {
                let value =
                    value
                        .get("raw")
                        .and_then(|v| v.as_bool())
                        .ok_or(Error::from_client(
                            "Invalid response from server: value is not valid",
                        ))?;
                RawValue::Boolean(value)
            }
            "type.googleapis.com/qdb.EntityReference" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client(
                        "Invalid response from server: value is not valid",
                    ))?
                    .to_string();
                RawValue::EntityReference(value)
            }
            "type.googleapis.com/qdb.Timestamp" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client(
                        "Invalid response from server: value is not valid",
                    ))?;
                let timestamp = DateTime::parse_from_rfc3339(value)?.to_utc();
                RawValue::Timestamp(timestamp)
            }
            "type.googleapis.com/qdb.ConnectionState" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client(
                        "Invalid response from server: value is not valid",
                    ))?
                    .to_string();
                RawValue::ConnectionState(value)
            }
            "type.googleapis.com/qdb.GarageDoorState" => {
                let value = value
                    .get("raw")
                    .and_then(|v| v.as_str())
                    .ok_or(Error::from_client(
                        "Invalid response from server: value is not valid",
                    ))?
                    .to_string();
                RawValue::GarageDoorState(value)
            }
            _ => {
                return Err(Error::from_client(
                    "Invalid response from server: value type is not valid",
                ))
            }
        };

        Ok(value.into_value())
    }
}

impl ClientTrait for Client {
    fn connect(&mut self) -> Result<()> {
        self.authenticate()?;

        self.auth_failure = false;
        self.endpoint_reachable = true;

        Ok(())
    }

    fn connected(&self) -> bool {
        self.endpoint_reachable && !self.auth_failure
    }

    fn disconnect(&mut self) -> bool {
        self.auth_failure = false;
        self.endpoint_reachable = false;
        true
    }

    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity> {
        let mut request = Map::new();
        request.insert(
            "@type".to_string(),
            Value::String("type.googleapis.com/qdb.WebConfigGetEntityRequest".to_string()),
        );
        request.insert("id".to_string(), Value::String(entity_id.to_string()));

        let response = self.send(&request)?;
        let entity = response
            .as_object()
            .and_then(|o| o.get("entity"))
            .and_then(|v| v.as_object())
            .ok_or(Error::from_client(
                "Invalid response from server: Failed to extract entity",
            ))?;

        Ok(DatabaseEntity {
            entity_id: entity
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client(
                    "Invalid response from server: entity id is not valid",
                ))?
                .to_string(),
            entity_type: entity
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client(
                    "Invalid response from server: entity type is not valid",
                ))?
                .to_string(),
            entity_name: entity
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client(
                    "Invalid response from server: entity name is not valid",
                ))?
                .to_string(),
        })
    }

    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        let mut request = Map::new();
        request.insert(
            "@type".to_string(),
            Value::String("type.googleapis.com/qdb.WebRuntimeGetEntitiesRequest".to_string()),
        );
        request.insert(
            "entityType".to_string(),
            Value::String(entity_type.to_string()),
        );

        let response = self.send(&request)?;
        let entities = response
            .as_object()
            .and_then(|o| o.get("entities"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client(
                "Invalid response from server: Failed to extract entities",
            ))?;

        let mut result = vec![];
        for entity in entities {
            match entity {
                Value::Object(entity) => result.push(DatabaseEntity {
                    entity_id: entity
                        .get("id")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client(
                            "Invalid response from server: entity id is not valid",
                        ))?
                        .to_string(),
                    entity_type: entity
                        .get("type")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client(
                            "Invalid response from server: entity type is not valid",
                        ))?
                        .to_string(),
                    entity_name: entity
                        .get("name")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client(
                            "Invalid response from server: entity name is not valid",
                        ))?
                        .to_string(),
                }),
                _ => {
                    return Err(Error::from_client(
                        "Invalid response from server: entity is not an object",
                    ))
                }
            }
        }

        Ok(result)
    }

    fn read(&mut self, requests: &Vec<DatabaseField>) -> Result<()> {
        let mut request = Map::new();
        request.insert(
            "@type".to_string(),
            Value::String("type.googleapis.com/qdb.WebRuntimeDatabaseRequest".to_string()),
        );
        request.insert("requestType".to_string(), Value::String("READ".to_string()));

        {
            let requests = Value::Array(
                requests
                    .iter()
                    .map(|r| {
                        let mut request = Map::new();
                        request.insert("id".to_string(), Value::String(r.entity_id()));
                        request.insert("field".to_string(), Value::String(r.name()));
                        Value::Object(request)
                    })
                    .collect(),
            );
            request.insert("requests".to_string(), requests);
        }

        let response = self.send(&request)?;
        let entities = response
            .as_object()
            .and_then(|o| o.get("response"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client(
                "Invalid response from server: response is not valid",
            ))?;

        for entity in entities {
            match entity {
                Value::Object(entity) => {
                    let entity_id = entity
                        .get("id")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client(
                            "Invalid response from server: entity id is not valid",
                        ))?
                        .to_string();

                    let field_name = entity
                        .get("field")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client(
                            "Invalid response from server: field name is not valid",
                        ))?
                        .to_string();

                    let field = requests
                        .iter()
                        .find(|r: &&DatabaseField| {
                            r.entity_id() == entity_id && r.name() == field_name
                        })
                        .ok_or(Error::from_client(
                            "Invalid response from server: Field not found",
                        ))?;

                    let value = entity
                        .get("value")
                        .and_then(|v: &Value| v.as_object())
                        .ok_or(Error::from_client(
                            "Invalid response from server: value is not valid",
                        ))?;

                    let write_time = entity
                        .get("writeTime")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client(
                            "Invalid response from server: write time is not valid",
                        ))?
                        .get("raw")
                        .ok_or(Error::from_client(
                            "Invalid response from server: write time is not valid",
                        ))?
                        .as_str()
                        .ok_or(Error::from_client(
                            "Invalid response from server: write time is not valid",
                        ))?;

                    let writer_id = entity
                        .get("writerId")
                        .and_then(|v| v.as_object())
                        .ok_or(Error::from_client(
                            "Invalid response from server: writer id is not valid",
                        ))?
                        .get("raw")
                        .ok_or(Error::from_client(
                            "Invalid response from server: writer id is not valid",
                        ))?
                        .as_str()
                        .ok_or(Error::from_client(
                            "Invalid response from server: writer id is not valid",
                        ))?
                        .to_string();

                    field.update_value(Client::extract_value(value)?);
                    field.update_write_time(DateTime::parse_from_rfc3339(write_time)?.to_utc());
                    field.update_writer_id(writer_id.as_str());
                }
                _ => {
                    return Err(Box::new(Error::ClientError(
                        "Invalid response from server: response is not an object".to_string(),
                    )))
                }
            }
        }

        Ok(())
    }

    fn write(&mut self, requests: &Vec<DatabaseField>) -> Result<()> {
        let mut request = Map::new();
        request.insert(
            "@type".to_string(),
            Value::String("type.googleapis.com/qdb.WebRuntimeDatabaseRequest".to_string()),
        );
        request.insert(
            "requestType".to_string(),
            Value::String("WRITE".to_string()),
        );

        {
            let requests = Value::Array(
                requests
                    .iter()
                    .map(|r| {
                        let mut request = Map::new();
                        request.insert("id".to_string(), Value::String(r.entity_id()));
                        request.insert("field".to_string(), Value::String(r.name()));
                        let value = match &r.value().into_raw() {
                            RawValue::String(s) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String("type.googleapis.com/qdb.String".to_string()),
                                );
                                value.insert("raw".to_string(), Value::String(s.clone()));
                                Value::Object(value)
                            }
                            RawValue::Integer(i) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String("type.googleapis.com/qdb.Int".to_string()),
                                );
                                let n = Number::from(*i);
                                value.insert("raw".to_string(), Value::Number(n));
                                Value::Object(value)
                            }
                            RawValue::Float(f) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String("type.googleapis.com/qdb.Float".to_string()),
                                );
                                let n = Number::from_f64(*f).unwrap_or(Number::from(0));
                                value.insert("raw".to_string(), Value::Number(n));
                                Value::Object(value)
                            }
                            RawValue::Boolean(b) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String("type.googleapis.com/qdb.Bool".to_string()),
                                );
                                value.insert("raw".to_string(), Value::Bool(*b));
                                Value::Object(value)
                            }
                            RawValue::EntityReference(e) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String(
                                        "type.googleapis.com/qdb.EntityReference".to_string(),
                                    ),
                                );
                                value.insert("raw".to_string(), Value::String(e.clone()));
                                Value::Object(value)
                            }
                            RawValue::Timestamp(t) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String("type.googleapis.com/qdb.Timestamp".to_string()),
                                );
                                let seconds = t.timestamp();
                                let nanos = t.timestamp_subsec_nanos();
                                let mut raw = Map::new();
                                raw.insert(
                                    "seconds".to_string(),
                                    Value::Number(Number::from(seconds)),
                                );
                                raw.insert(
                                    "nanos".to_string(),
                                    Value::Number(Number::from(nanos as i64)),
                                );
                                value.insert("raw".to_string(), Value::Object(raw));
                                Value::Object(value)
                            }
                            RawValue::ConnectionState(c) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String(
                                        "type.googleapis.com/qdb.ConnectionState".to_string(),
                                    ),
                                );
                                value.insert("raw".to_string(), Value::String(c.clone()));
                                Value::Object(value)
                            }
                            RawValue::GarageDoorState(g) => {
                                let mut value = Map::new();
                                value.insert(
                                    "@type".to_string(),
                                    Value::String(
                                        "type.googleapis.com/qdb.GarageDoorState".to_string(),
                                    ),
                                );
                                value.insert("raw".to_string(), Value::String(g.clone()));
                                Value::Object(value)
                            }
                            _ => Value::Null,
                        };
                        request.insert("value".to_string(), value);
                        Value::Object(request)
                    })
                    .collect(),
            );
            request.insert("requests".to_string(), requests);
        }

        self.send(&request)?;

        Ok(())
    }

    fn register_notification(&mut self, config: &NotificationConfig) -> Result<NotificationToken> {
        let context = config
            .context
            .iter()
            .map(|v| Value::String(v.into()))
            .collect();

        let mut notification = Map::new();
        notification.insert("id".to_string(), Value::String(config.entity_id.clone()));
        notification.insert("type".to_string(), Value::String(config.entity_type.clone()));
        notification.insert("field".to_string(), Value::String(config.field.clone()));
        notification.insert(
            "notifyOnChange".to_string(),
            Value::Bool(config.notify_on_change),
        );
        notification.insert("contextFields".to_string(), Value::Array(context));

        let mut request = Map::new();
        request.insert(
            "@type".to_string(),
            Value::String(
                "type.googleapis.com/qdb.WebRuntimeRegisterNotificationRequest".to_string(),
            ),
        );
        request.insert(
            "requests".to_string(),
            Value::Array(vec![Value::Object(notification)]),
        );

        let response = self.send(&request)?;
        let token = response
            .as_object()
            .and_then(|o| o.get("tokens"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client(
                "Invalid response from server: token is not valid",
            ))?
            .get(0)
            .ok_or(Error::from_client(
                "Invalid response from server: token is not valid",
            ))?
            .as_str()
            .ok_or(Error::from_client(
                "Invalid response from server: token is not valid",
            ))?;

        Ok(NotificationToken(token.to_string()))
    }

    fn unregister_notification(&mut self, token: &NotificationToken) -> Result<()> {
        let mut request = Map::new();
        request.insert(
            "@type".to_string(),
            Value::String(
                "type.googleapis.com/qdb.WebRuntimeUnregisterNotificationRequest".to_string(),
            ),
        );
        request.insert(
            "tokens".to_string(),
            Value::Array(vec![Value::String(token.into())]),
        );

        self.send(&request)?;

        Ok(())
    }

    fn get_notifications(&mut self) -> Result<Vec<DatabaseNotification>> {
        let mut request = Map::new();
        request.insert(
            "@type".to_string(),
            Value::String("type.googleapis.com/qdb.WebRuntimeGetNotificationsRequest".to_string()),
        );

        let response = self.send(&request)?;
        let notifications = response
            .as_object()
            .and_then(|o| o.get("notifications"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                Error::from_client("Invalid response from server: notifications is not valid")
            })?;

        let mut result = Vec::with_capacity(notifications.len());
        for notification in notifications {
            let token = notification
                .pointer("/token")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    Error::from_client("Invalid response from server: notification token is not valid")
                })?
                .to_string();

            let current = self.parse_database_field(notification, "/current")?;
            let previous = self.parse_database_field(notification, "/previous")?;

            let context = notification
                .pointer("/context")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    Error::from_client("Invalid response from server: notification context is not valid")
                })?
                .iter()
                .map(|v| self.parse_database_field(v, ""))
                .collect::<Result<Vec<RawField>>>()?;

            result.push(DatabaseNotification {
                token,
                current,
                previous,
                context,
            });
        }

        Ok(result)
    }
}
