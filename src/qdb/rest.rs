use super::Error;
use super::Result;
use super::DatabaseEntity;
use super::DatabaseField;

pub struct Client {
    url: String,
    request_template: ureq::serde_json::Map<String, ureq::serde_json::Value>
}

impl Client {
    pub fn new(url: &str) -> Client {
        Client {
            url: url.to_string(),
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
            _ => Err(Box::new(Error::ClientError("Invalid response from server".to_string())))
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

        Err(Error::ClientError("Failed to authenticate".to_string()))
    }
}

impl ClientTrait for Client {
    fn get_entity(&self, entity_id: &str) -> Result<DatabaseEntity> {
        let request = self.request_template.clone();
        let payload = ureq::serde_json::Map::new();
        payload.insert("@type".to_string(), "type.googleapis.com/qdb.WebRuntimeGetEntityRequest");
        payload.insert("entityId".to_string(), entity_id);
        request.insert("payload".to_string(), payload);

        let entity = self.send(request)?
            .as_object()
            .and_then(|o| o.get("payload"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("entity"))
            .and_then(|v| v.as_object())?;

        Ok(DatabaseEntity{
            entity_id: entity.get("id")?.as_str()?.to_string(),
            entity_type: entity.get("type")?.as_str()?.to_string(),
            entity_name: entity.get("name")?.as_str()?.to_string()
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
                    result.push(DatabaseEntity{
                        entity_id: entity["id"].as_str()?.to_string(),
                        entity_type: entity["type"].as_str()?.to_string(),
                        entity_name: entity["name"].as_str()?.to_string()
                    })
                },
                _ => return Err(Box::new(Error::ClientError("Invalid response from server: entity is not an object".to_string())))
            }
        }

        Ok(result)
    }

    fn read(&self, requests: &mut Vec<DatabaseField>) -> Result<()> {
        let request = self.request_template.clone();
        let payload = ureq::serde_json::Map::new();
        payload.insert("@type".to_string(), "type.googleapis.com/qdb.WebRuntimeDatabaseRequest");
        payload.insert("requestType".to_string(), "READ");

        {
            let requests = ureq::serde_json::Value::Array(requests.iter().map(|r| {
                let request = ureq::serde_json::Map::new();
                request.insert("id".to_string(), r.entity_id);
                request.insert("field".to_string(), r.field);
                request
            }).collect());
            payload.insert("requests".to_string(), requests);
        }
        request["payload"] = payload;

        let responses = self.send(request)?
            .as_object()
            .and_then(|o| o.get("payload"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("response"))
            .and_then(|v| v.as_array())?;

        for response in responses {
            match response {
                ureq::serde_json::Value::Object(response) => {
                    let entity_id = response["id"].as_str()?;
                    let field = response["field"].as_str()?;
                    let value = response["value"].as_object()?;
                    let write_time = response["writeTime"];
                    let writer_id = response["writerId"];

                    let field = requests.iter_mut().find(|r| r.entity_id == entity_id && r.field == field)?;
                    field.value = value.get(key);
                    field.write_time = write_time;
                    field.writer_id = writer_id;

                }
                _ => return Err(Box::new(Error::ClientError("Invalid response from server: response is not an object".to_string())))
            }
        }

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