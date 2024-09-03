use qdb::{ApplicationContext, ApplicationTrait, Database, DatabaseNotification, NotificationCallback, NotificationConfig, SignalTrait};

mod qdb;

fn on_current_time_changed(n: &DatabaseNotification, db: Database) -> qdb::Result<()> {
    let n = n.current.value().as_str()?;

    Ok(())
}

fn on_connected(args: &ApplicationContext) {
    let db = args.database().clone();
    args.database().register_notification(&NotificationConfig{
        entity_type: "SystemClock".to_string(),
        entity_id: "".to_string(),
        field: "CurrentTime".to_string(),
        notify_on_change: true,
        context: vec![],
    }, NotificationCallback::new(move |n| on_current_time_changed(n, db.clone()))).unwrap();
}

fn on_disconnected(args: &ApplicationContext) {
    println!("Disconnected");
}

fn main() {
    let ctx = qdb::ApplicationContext::new(
        qdb::Database::new(qdb::rest::Client::new("http://localhost:20000")),
        qdb::Logger::new(qdb::ConsoleLogger::new(qdb::LogLevel::Debug)),
    );

    let mut app = qdb::Application::new(ctx, 500);

    let mut db_worker = qdb::DatabaseWorker::new();
    db_worker.signals.connected.connect(qdb::Slot::new(on_connected));
    db_worker.signals.disconnected.connect(qdb::Slot::new(on_disconnected));

    app.add_worker(Box::new(db_worker));

    app.execute();
}