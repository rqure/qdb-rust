use super::Error;
use super::Result;
use super::DatabaseEntity;
use super::DatabaseField;
use super::ClientTrait;

use ureq::serde_json::Value;
use ureq::serde_json::Map;

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

    fn send(&mut self, request: &Map<String, Value>) -> Result<Value> {
        let attempts = 3;

        for _ in 0..attempts {
            let response = ureq::post(format!("{}/api", self.url).as_str())
                .send_json(Value::Object(request.clone()))
                .map_err(|e| Box::new(e))?
                .into_json()
                .map_err(|e| Box::new(e))?;

            if self.has_authenticated(&response) {
                return Ok(response)
            } else {
                self.authenticate()?;
            }
        }

        Err(Box::new(Error::ClientError("Failed to authenticate".to_string())))
    }
}

impl ClientTrait for Client {
    fn get_entity(&mut self, entity_id: &str) -> Result<DatabaseEntity> {
        let mut request = self.request_template.clone();
        let mut payload = Map::new();
        payload.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeGetEntityRequest".to_string()));
        payload.insert("entityId".to_string(), Value::String(entity_id.to_string()));
        request.insert("payload".to_string(), Value::Object(payload));

        let response = self.send(&request)?;
        let entity = response
            .as_object()
            .and_then(|o| o.get("payload"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("entity"))
            .and_then(|v| v.as_object())
            .ok_or(Error::from_client("Invalid response from server: Failed to extract entity"))?;

        Ok(DatabaseEntity{
            entity_id: entity
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client("Invalid response from server: Entity id is not valid"))?
                .to_string(),
            entity_type: entity
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client("Invalid response from server: Entity type is not valid"))?
                .to_string(),
            entity_name: entity
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(Error::from_client("Invalid response from server: Entity name is not valid"))?
                .to_string()
        })
    }

    fn get_entities(&mut self, entity_type: &str) -> Result<Vec<DatabaseEntity>> {
        let mut request = self.request_template.clone();
        let mut payload = Map::new();
        payload.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeGetEntitiesRequest".to_string()));
        payload.insert("entityType".to_string(), Value::String(entity_type.to_string()));
        request.insert("payload".to_string(), Value::Object(payload));

        let response = self.send(&request)?;
        let entities = response
            .as_object()
            .and_then(|o| o.get("payload"))
            .and_then(|v| v.as_object())
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
                            .ok_or(Error::from_client("Invalid response from server: Entity id is not valid"))?
                            .to_string(),
                        entity_type: entity
                            .get("type")
                            .and_then(|v| v.as_str())
                            .ok_or(Error::from_client("Invalid response from server: Entity type is not valid"))?
                            .to_string(),
                        entity_name: entity
                            .get("name")
                            .and_then(|v| v.as_str())
                            .ok_or(Error::from_client("Invalid response from server: Entity name is not valid"))?
                            .to_string()
                    })
                },
                _ => return Err(Error::from_client("Invalid response from server: entity is not an object"))
            }
        }

        Ok(result)
    }

    fn read(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        let mut request = self.request_template.clone();
        let mut payload = Map::new();
        payload.insert("@type".to_string(), Value::String("type.googleapis.com/qdb.WebRuntimeDatabaseRequest".to_string()));
        payload.insert("requestType".to_string(), Value::String("READ".to_string()));

        {
            let requests = Value::Array(requests.iter().map(|r| {
                let request = Map::new();
                request.insert("id".to_string(), Value::String(r.entity_id));
                request.insert("field".to_string(), Value::String(r.field));
                Value::Object(request)
            }).collect());
            payload.insert("requests".to_string(), requests);
        }
        request.insert("payload".to_string(), Value::Object(payload));

        let response = self.send(&request)?;
        let entities = response
            .as_object()
            .and_then(|o| o.get("payload"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("response"))
            .and_then(|v| v.as_array())
            .ok_or(Error::from_client("Invalid response from server: response is not valid"))?;

        for entity in entities {
            match entity {
                Value::Object(entity) => {
                    let entity_id = entity
                        .get("id")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client("Invalid response from server: Entity id is not valid"))?
                        .to_string();

                    let field_name = entity
                        .get("field")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client("Invalid response from server: Entity id is not valid"))?
                        .to_string();

                    let field = requests
                        .iter_mut()
                        .find(|r: &&mut DatabaseField| r.entity_id == entity_id && r.field == field_name)
                        .ok_or(Error::from_client("Invalid response from server: Field not found"))?;

                    let value = entity
                        .get("value")
                        .and_then(|v: &Value| v.as_str())
                        .ok_or(Error::from_client("Invalid response from server: value is not valid"))?
                        .to_string();

                    let write_time = entity
                        .get("writeTime")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client("Invalid response from server: write time is not valid"))?
                        .to_string();

                    let writer_id = entity
                        .get("writerId")
                        .and_then(|v| v.as_str())
                        .ok_or(Error::from_client("Invalid response from server: writer id is not valid"))?
                        .to_string();

                    field.value = value;
                    field.write_time = write_time;
                    field.writer_id = writer_id;
                }
                _ => return Err(Box::new(Error::ClientError("Invalid response from server: response is not an object".to_string())))
            }
        }

        Ok(())
    }

    fn write(&mut self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        let request = self.request_template.clone();
        let payload = Map::new();
        payload.insert("@type".to_string(), "type.googleapis.com/qdb.WebRuntimeDatabaseRequest");
        payload.insert("requestType".to_string(), "WRITE");

        {
            let requests = Value::Array(requests.iter().map(|r| {
                let request = Map::new();
                request.insert("id".to_string(), r.entity_id);
                request.insert("field".to_string(), r.field);
                request.insert("value".to_string(), r.value);
                request
            }).collect());
            payload.insert("requests".to_string(), requests);
        }
        request.insert("payload".to_string(), payload);

        self.send(request)?;

        Ok(())
    }
    
    // fn register_notification(&self, config: NotificationConfig) -> Result<NotificationToken> {

    // }

    // fn unregister_notification(&self, token: NotificationToken) -> Result<()> {
        
    // }

    // fn process_notifications(&self) -> Result<Vec<DatabaseNotification>> {
        
    // }
}