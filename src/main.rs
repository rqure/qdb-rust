use qdb::{ApplicationContext, ApplicationTrait, Database, DatabaseNotification, NotificationCallback, NotificationConfig, SignalTrait};

mod qdb;

fn on_current_time_changed(n: &DatabaseNotification, db: Database) -> qdb::Result<()> {
    let n = n.current.value().as_str()?;

    Ok(())
}

fn on_connected(args: &mut ApplicationContext) {
    args.database.register_notification(&NotificationConfig{
        entity_type: "SystemClock".to_string(),
        entity_id: "".to_string(),
        field: "CurrentTime".to_string(),
        notify_on_change: true,
        context: vec![],
    }, NotificationCallback::new(|n| on_current_time_changed(n, args.database.clone()))).unwrap();
}

fn on_disconnected(args: &mut ApplicationContext) {
    println!("Disconnected");
}

fn main() {
    let mut app = qdb::Application::new(500);
    let mut ctx = qdb::ApplicationContext{
        database: qdb::Database::new(qdb::rest::Client::new("http://localhost:20000")),
        logger: qdb::Logger::new(qdb::ConsoleLogger::new(qdb::LogLevel::Debug)),
        quit: qdb::BoolFlag::new(),
    };

    let mut db_worker = qdb::DatabaseWorker::new();
    db_worker.signals.connected.connect(qdb::Slot::new(on_connected));
    db_worker.signals.disconnected.connect(qdb::Slot::new(on_disconnected));

    app.add_worker(Box::new(db_worker));

    app.execute(&mut ctx);
}