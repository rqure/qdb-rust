use chrono::Local;
use qdb::{ClientTrait, SignalTrait};

mod qdb;

fn main() {
    let mut signal = qdb::Signal::new();
    let mut token = signal.connect(qdb::Slot::new(|args: &(String, i16)| {
        println!("Signal emitted: {}", args.1);
    }));
    
    signal.emit(&("Hello".to_string(), 42));
    signal.emit(&("Hello".to_string(), 43));

    token.disconnect();

    let mut client = qdb::rest::Client::new("http://localhost:20000");
    match client.get_entities("Root") {
        Ok(entities) => {
            for entity in entities {
                let mut fields = vec![
                    qdb::DatabaseField::new(entity.entity_id, "SchemaUpdateTrigger")
                ];

                client.read(&mut fields).unwrap();

                for field in &fields {
                    println!("{}: {:?}: {}", field.name, field.value, field.write_time.with_timezone(&Local));
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}