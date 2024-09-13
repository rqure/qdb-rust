use crate::error::Error;
use crate::framework::client::Client;
use crate::framework::events::emitter::Emitter;
use crate::Result;
use crate::schema::notification::{Notification, Config, Token};

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::mpsc::Receiver;

pub struct _NotificationManager {
    registered_config: HashSet<Config>,
    config_to_token: HashMap<Config, Token>,
    token_to_callback_list: HashMap<Token, Emitter<Notification>>,
}

type NotificationManagerRef = Rc<RefCell<_NotificationManager>>;
pub struct NotificationManager(NotificationManagerRef);

impl NotificationManager {
    pub fn new() -> Self {
        NotificationManager(Rc::new(RefCell::new(_NotificationManager::new())))
    }

    pub fn clone(&self) -> Self {
        NotificationManager(self.0.clone())
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub fn register(
        &self,
        client: Client,
        config: &Config,
    ) -> Result<Receiver<Notification>> {
        self.0.borrow_mut().register(client, config)
    }

    pub fn unregister(&self, client: Client, token: &Token) -> Result<()> {
        self.0.borrow_mut().unregister(client, token)
    }

    pub fn process_notifications(&self, client: Client) -> Result<()> {
        self.0.borrow_mut().process_notifications(client)
    }
}

impl _NotificationManager {
    pub fn new() -> Self {
        _NotificationManager {
            registered_config: HashSet::new(),
            config_to_token: HashMap::new(),
            token_to_callback_list: HashMap::new(),
        }
    }
}

impl _NotificationManager {
    fn clear(&mut self) {
        self.registered_config.clear();
        self.config_to_token.clear();
        self.token_to_callback_list.clear();
    }

    fn register(
        &mut self,
        client: Client,
        config: &Config,
    ) -> Result<Receiver<Notification>> {
        if self.registered_config.contains(&config) {
            let token = self
                .config_to_token
                .get(config)
                .ok_or(Error::from_notification(
                    "Inconsistent notification state during registration",
                ))?;

            let receiver = self
                .token_to_callback_list
                .get_mut(token)
                .ok_or(Error::from_notification(
                    "Inconsistent notification state during registration",
                ))?
                .new_receiver();

            return Ok(receiver);
        }

        let token = client.register_notification(config)?;

        self.registered_config.insert(config.clone());
        self.config_to_token.insert(config.clone(), token.clone());
        self.token_to_callback_list
            .insert(token.clone(), Emitter::new());

        let receiver = self
            .token_to_callback_list
            .get_mut(&token)
            .ok_or(Error::from_notification(
                "Inconsistent notification state during registration",
            ))?
            .new_receiver();

        Ok(receiver)
    }

    fn unregister(&mut self, client: Client, token: &Token) -> Result<()> {
        if !self.token_to_callback_list.contains_key(token) {
            return Err(Error::from_notification(
                "Token not found during unregistration",
            ));
        }

        client.unregister_notification(token)?;

        self.token_to_callback_list.remove(token);
        self.config_to_token.retain(|_, v| v != token);
        self.registered_config
            .retain(|c| self.config_to_token.contains_key(c));

        Ok(())
    }

    fn process_notifications(&mut self, client: Client) -> Result<()> {
        let notifications = client.get_notifications()?;

        for notification in &notifications {
            let token = Token::from(notification.token.clone());
            let emitter =
                self.token_to_callback_list
                    .get_mut(&token)
                    .ok_or(Error::from_notification(
                        "Cannot process notification: Callback list doesn't exist for token",
                    ))?;
            emitter.emit(notification.clone());
        }

        Ok(())
    }
}