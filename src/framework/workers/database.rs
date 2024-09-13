use crate::framework::application::Context;
use crate::framework::workers::common::WorkerTrait;
use crate::framework::events::emitter::Emitter;

use crate::loggers::common::LogLevel;

use crate::Result;

use std::sync::mpsc::Receiver;

pub struct Emitters {
    pub connection_status: Emitter<bool>,
}

pub struct Receivers {
    pub network_connection_events: Option<Receiver<bool>>,
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
                network_connection_events: None,
            },
        }
    }
}

impl WorkerTrait for Worker {
    fn intialize(&mut self, ctx: Context) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::DatabaseWorker::initialize] Initializing database worker",
        );
        Ok(())
    }

    fn do_work(&mut self, ctx: Context) -> Result<()> {
        if !self.is_nw_connected {
            if self.is_db_connected {
                ctx.logger().log(&LogLevel::Warning, "[qdb::DatabaseWorker::do_work] Network connection loss has disrupted database connection");
                self.is_db_connected = false;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            return Ok(());
        }

        if !ctx.database().connected() {
            if self.is_db_connected {
                ctx.logger().log(
                    &LogLevel::Warning,
                    "[qdb::DatabaseWorker::do_work] Disconnected from database",
                );
                ctx.database().clear_notifications();
                self.is_db_connected = false;
                self.emitters.connection_status.emit(self.is_db_connected);
            }

            ctx.logger().log(
                &LogLevel::Debug,
                "[qdb::DatabaseWorker::do_work] Attempting to connect to the database...",
            );

            ctx.database().disconnect();
            ctx.database().connect()?;

            if ctx.database().connected() {
                ctx.logger().log(
                    &LogLevel::Info,
                    "[qdb::DatabaseWorker::do_work] Connected to the database",
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
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::DatabaseWorker::deinitialize] Deinitializing database worker",
        );
        Ok(())
    }

    fn process_events(&mut self) -> Result<()> {
        if let Some(receiver) = &self.receivers.network_connection_events {
            while let Ok(connected) = receiver.try_recv() {
                self.is_nw_connected = connected;
            }
        }

        Ok(())
    }
}