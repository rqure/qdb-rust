use chrono::Local;
use qdb::ApplicationTrait;

mod qdb;

fn main() {
    let app = qdb::Application::new(500);
    let mut ctx = qdb::ApplicationContext{
        logger: Box::new(qdb::ConsoleLogger::new(qdb::LogLevel::Debug)),
        client: Box::new(qdb::rest::Client::new("http://localhost:20000")),
        notification_manager: Box::new(qdb::NotificationManager::new()),
        quit: false,
    };

    app.execute(&mut ctx);
}