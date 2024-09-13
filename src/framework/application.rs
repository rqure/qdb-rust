use crate::framework::database::Database;
use crate::framework::logger::Logger;
use crate::framework::workers::common::WorkerTrait;
use crate::loggers::common::LogLevel;
use crate::Result;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

pub trait ApplicationTrait {
    fn execute(&mut self);
    fn add_worker(&mut self, worker: Box<dyn WorkerTrait>);
}

type _BoolFlag = Rc<RefCell<bool>>;
pub struct BoolFlag(_BoolFlag);

impl BoolFlag {
    pub fn new() -> Self {
        BoolFlag(Rc::new(RefCell::new(false)))
    }

    pub fn set(&self, value: bool) {
        *self.0.borrow_mut() = value;
    }

    pub fn get(&self) -> bool {
        *self.0.borrow()
    }
}

impl Clone for BoolFlag {
    fn clone(&self) -> Self {
        BoolFlag(self.0.clone())
    }
}

struct _ApplicationContext {
    pub database: Database,
    pub logger: Logger,
    pub quit: BoolFlag,
}

type ApplicationContextRef = Rc<RefCell<_ApplicationContext>>;
pub struct ApplicationContext(ApplicationContextRef);

impl ApplicationContext {
    pub fn new(database: Database, logger: Logger) -> Self {
        ApplicationContext(Rc::new(RefCell::new(_ApplicationContext {
            database,
            logger,
            quit: BoolFlag::new(),
        })))
    }

    pub fn database(&self) -> Database {
        self.0.borrow().database.clone()
    }

    pub fn logger(&self) -> Logger {
        self.0.borrow().logger.clone()
    }

    pub fn quit(&self) -> BoolFlag {
        self.0.borrow().quit.clone()
    }
}

impl Clone for ApplicationContext {
    fn clone(&self) -> Self {
        ApplicationContext(self.0.clone())
    }
}

pub struct Application {
    ctx: ApplicationContext,
    workers: Vec<Box<dyn WorkerTrait>>,
    loop_interval_ms: u64,
}

impl Application {
    pub fn new(ctx: ApplicationContext, loop_interval_ms: u64) -> Self {
        Self {
            ctx,
            workers: vec![],
            loop_interval_ms,
        }
    }
}

impl WorkerTrait for Application {
    fn intialize(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::initialize] Initializing application",
        );
        for worker in &mut self.workers {
            match worker.intialize(ctx.clone()) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger().error(&format!(
                        "[qdb::Application::initialize] Error while initializing worker: {}",
                        e
                    ));
                }
            }
        }

        Ok(())
    }

    fn do_work(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::do_work] Application has started",
        );

        while {
            let start = Instant::now();

            for i in 0..self.workers.len() {
                let worker = &mut self.workers[i];
                match worker.do_work(ctx.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        ctx.logger().error(&format!(
                            "[qdb::Application::do_work] Error while executing worker: {}",
                            e
                        ));
                    }
                }

                match self.process_events() {
                    Ok(_) => {}
                    Err(e) => {
                        ctx.logger().error(&format!(
                            "[qdb::Application::do_work] Error while processing events: {}",
                            e
                        ));
                    }
                }
            }

            if !ctx.quit().get() {
                let loop_time = std::time::Duration::from_millis(self.loop_interval_ms);
                let elapsed_time = start.elapsed();

                if loop_time > elapsed_time {
                    let sleep_time = loop_time - start.elapsed();
                    std::thread::sleep(sleep_time);
                }
            }

            !ctx.quit().get()
        } {}

        Ok(())
    }

    fn deinitialize(&mut self, ctx: ApplicationContext) -> Result<()> {
        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::deinitialize] Deinitializing application",
        );

        for worker in &mut self.workers {
            match worker.deinitialize(ctx.clone()) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger().error(&format!(
                        "[qdb::Application::deinitialize] Error while deinitializing worker: {}",
                        e
                    ));
                }
            }
        }

        ctx.logger().log(
            &LogLevel::Info,
            "[qdb::Application::deinitialize] Shutting down now",
        );
        Ok(())
    }

    fn process_events(&mut self) -> Result<()> {
        for worker in &mut self.workers {
            match worker.process_events() {
                Ok(_) => {}
                Err(e) => {
                    self.ctx.logger().error(&format!(
                        "[qdb::Application::process_events] Error while processing events: {}",
                        e
                    ));
                }
            }
        }

        Ok(())
    }
}

impl ApplicationTrait for Application {
    fn execute(&mut self) {
        self.intialize(self.ctx.clone()).unwrap();
        self.do_work(self.ctx.clone()).unwrap();
        self.deinitialize(self.ctx.clone()).unwrap();
    }

    fn add_worker(&mut self, worker: Box<dyn WorkerTrait>) {
        self.workers.push(worker);
    }
}