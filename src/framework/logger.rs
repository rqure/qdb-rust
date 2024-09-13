
pub type LoggerRef = Rc<RefCell<dyn LoggerTrait>>;
pub struct Logger(LoggerRef);

impl Logger {
    pub fn new(logger: impl LoggerTrait + 'static) -> Self {
        Logger(Rc::new(RefCell::new(logger)))
    }

    pub fn clone(&self) -> Self {
        Logger(self.0.clone())
    }

    pub fn log(&self, level: &LogLevel, message: &str) {
        self.0.borrow_mut().log(level, message);
    }

    pub fn trace(&self, message: &str) {
        self.0.borrow_mut().trace(message);
    }

    pub fn debug(&self, message: &str) {
        self.0.borrow_mut().debug(message);
    }

    pub fn info(&self, message: &str) {
        self.0.borrow_mut().info(message);
    }

    pub fn warning(&self, message: &str) {
        self.0.borrow_mut().warning(message);
    }

    pub fn error(&self, message: &str) {
        self.0.borrow_mut().error(message);
    }
}