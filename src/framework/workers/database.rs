use crate::framework::application::Context;
use crate::framework::workers::common::WorkerTrait;
use crate::framework::events::emitter::Emitter;

use crate::Result;

use std::sync::mpsc::Receiver;

pub struct Emitters {
    pub connection_status: Emitter<bool>,
}

pub struct Receivers {
    pub network_connection_status: Option<Receiver<bool>>,
}

pub struct Worker {
    is_db_connected: bool,
    is_nw_connected: bool,
    pub emitters: Emitters,
    pub receivers: Receivers,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            is_db_connected: false,
            is_nw_connected: false,
            emitters: Emitters {
                connection_status: Emitter::new(),
            },
            receivers: Receivers {
                network_connection_status: None,
            },
        }
    }
}

impl WorkerTrait for Worker {
    fn intialize(&mut self, ctx: Context) -> Result<()> {
        let c = format!("{}::{}", std::any::type_name::<Self>(), "initialize");

        ctx.logger().info(
            format!("[{}] Initializing database worker", c).as_str(),
        );
        Ok(())
    }

    fn do_work(&mut self, ctx: Context) -> Result<()> {
        let c = format!("{}::{}", std::any::type_name::<Self>(), "do_work");

        if !self.is_nw_connected {
            if self.is_db_connected {
                ctx.logger().warning(
                    format!("[{}] Network connection loss has disrupted database connection", c).as_str()
                );
                self.is_db_connected = false;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            return Ok(());
        }

        if !ctx.database().connected() {
            if self.is_db_connected {
                ctx.logger().warning(
                    format!("[{}] Disconnected from database", c).as_str(),
                );
                ctx.database().clear_notifications();
                self.is_db_connected = false;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            ctx.logger().debug(
                format!("[{}] Attempting to connect to the database...", c).as_str(),
            );

            ctx.database().disconnect();
            ctx.database().connect()?;

            if ctx.database().connected() {
                ctx.logger().info(
                    format!("[{}] Connected to the database", c).as_str(),
                );
                self.is_db_connected = true;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            return Ok(());
        }

        ctx.database().process_notifications()?;

        Ok(())
    }

    fn deinitialize(&mut self, ctx: Context) -> Result<()> {
        let c = format!("{}::{}", std::any::type_name::<Self>(), "deinitialize");

        ctx.logger().info(
            format!("[{}] Deinitializing database worker", c).as_str(),
        );
        Ok(())
    }

    fn process_events(&mut self) -> Result<()> {
        if let Some(receiver) = &self.receivers.network_connection_status {
            while let Ok(connected) = receiver.try_recv() {
                self.is_nw_connected = connected;
            }
        }

        Ok(())
    }
}