use chrono::Local;
use qdb::ApplicationTrait;

mod qdb;

fn main() {
    let app = qdb::Application::new(500);
    let mut ctx = qdb::ApplicationContext{
        database: Box::new(
            qdb::Database::new(
            Box::new(qdb::rest::Client::new("http://localhost:8080"))
        )),
        logger: Box::new(qdb::ConsoleLogger::new(qdb::LogLevel::Debug)),
        quit: false,
    };

    app.execute(&mut ctx);
}