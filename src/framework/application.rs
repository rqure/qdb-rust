use crate::framework::database::Database;
use crate::framework::logger::Logger;
use crate::framework::workers::common::WorkerTrait;
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

struct _Context {
    pub database: Database,
    pub logger: Logger,
    pub quit: BoolFlag,
}

type ContextRef = Rc<RefCell<_Context>>;
pub struct Context(ContextRef);

impl Context {
    pub fn new(database: Database, logger: Logger) -> Self {
        Context(Rc::new(RefCell::new(_Context {
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

impl Clone for Context {
    fn clone(&self) -> Self {
        Context(self.0.clone())
    }
}

pub struct Application {
    ctx: Context,
    workers: Vec<Box<dyn WorkerTrait>>,
    loop_interval_ms: u64,
}

impl Application {
    pub fn new(ctx: Context, loop_interval_ms: u64) -> Self {
        Self {
            ctx,
            workers: vec![],
            loop_interval_ms,
        }
    }
}

impl WorkerTrait for Application {
    fn intialize(&mut self, ctx: Context) -> Result<()> {
        let c = format!("{}::{}", std::any::type_name::<Self>(), "initialize");

        ctx.logger().info(
            format!("[{}] Initializing application", c).as_str(),
        );
        for worker in &mut self.workers {
            match worker.intialize(ctx.clone()) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger().error(&format!(
                        "[{}] Error while initializing worker: {}",
                        c, e
                    ));
                }
            }
        }

        Ok(())
    }

    fn do_work(&mut self, ctx: Context) -> Result<()> {
        let c = format!("{}::{}", std::any::type_name::<Self>(), "do_work");

        ctx.logger().info(
            format!("[{}] Application has started", c).as_str(),
        );

        while {
            let start = Instant::now();

            for i in 0..self.workers.len() {
                let iter_start = Instant::now();

                let worker = &mut self.workers[i];
                match worker.do_work(ctx.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        ctx.logger().error(&format!(
                            "[{}] Error while executing worker: {}",
                            c, e
                        ));
                    }
                }

                let elapsed_ms = iter_start.elapsed().as_millis();
                ctx.logger().trace(
                    format!("[{}] Worker '{}' took {} ms to complete tick",
                        c, worker.name(), elapsed_ms).as_str());

                match self.process_events() {
                    Ok(_) => {}
                    Err(e) => {
                        ctx.logger().error(&format!(
                            "[{}] Error while processing events: {}",
                            c, e
                        ));
                    }
                }
            }

            if !ctx.quit().get() {
                let loop_time = std::time::Duration::from_millis(self.loop_interval_ms);
                let elapsed_time = start.elapsed();

                if loop_time > elapsed_time {
                    let sleep_time = loop_time - elapsed_time;
                    ctx.logger().trace(&format!(
                        "[{}] Idle for {:?} ms",
                        c, sleep_time.as_millis()
                    ));
                    std::thread::sleep(sleep_time);
                }
            }

            !ctx.quit().get()
        } {}

        Ok(())
    }

    fn deinitialize(&mut self, ctx: Context) -> Result<()> {
        let c = format!("{}::{}", std::any::type_name::<Self>(), "deinitialize");

        ctx.logger().info(
            format!("[{}] Deinitializing application", c).as_str(),
        );

        for worker in &mut self.workers {
            match worker.deinitialize(ctx.clone()) {
                Ok(_) => {}
                Err(e) => {
                    ctx.logger().error(&format!(
                        "[{}] Error while deinitializing worker: {}",
                        c, e
                    ));
                }
            }
        }

        ctx.logger().info(
            format!("[{}] Shutting down now", c).as_str(),
        );
        Ok(())
    }

    fn process_events(&mut self) -> Result<()> {
        let c = format!("{}::{}", std::any::type_name::<Self>(), "process_events");

        for worker in &mut self.workers {
            match worker.process_events() {
                Ok(_) => {}
                Err(e) => {
                    self.ctx.logger().error(&format!(
                        "[{}] Error while processing events: {}",
                        c, e
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