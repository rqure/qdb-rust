use qdb::{ApplicationTrait, SignalTrait};

mod qdb;

fn on_connected(args: &()) {
    println!("Connected");
}

fn on_disconnected(args: &()) {
    println!("Disconnected");
}

fn main() {
    let mut app = qdb::Application::new(500);
    let mut ctx = qdb::ApplicationContext{
        database: Box::new(
            qdb::Database::new(
            Box::new(qdb::rest::Client::new("http://localhost:20000"))
        )),
        logger: Box::new(qdb::ConsoleLogger::new(qdb::LogLevel::Debug)),
        quit: false,
    };

    let mut db_worker = qdb::DatabaseWorker::new();
    db_worker.signals.connected.connect(qdb::Slot::new(on_connected));
    db_worker.signals.disconnected.connect(qdb::Slot::new(on_disconnected));

    app.add_worker(Box::new(db_worker));

    app.execute(&mut ctx);
}