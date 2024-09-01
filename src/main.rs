use qdb::{ApplicationContext, ApplicationTrait, DatabaseNotification, NotificationConfig, SignalTrait};

mod qdb;

fn on_current_time_changed(n: &DatabaseNotification) {
    dbg!(n);
}

fn on_connected(args: &mut ApplicationContext) {
    args.database.register_notification(&NotificationConfig{
        entity_type: "SystemClock".to_string(),
        entity_id: "".to_string(),
        field: "CurrentTime".to_string(),
        notify_on_change: true,
        context: vec![],
    }, on_current_time_changed).unwrap();
}

fn on_disconnected(args: &mut ApplicationContext) {
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