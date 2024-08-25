use qdb::ClientTrait;

mod qdb;

fn main() {
    let mut client = qdb::rest::Client::new("http://localhost:20000");
    match client.get_entities("Root") {
        Ok(entities) => {
            for entity in entities {
                let mut fields = vec![
                    qdb::DatabaseField::new(entity.entity_id, "SchemaUpdateTrigger")
                ];

                client.read(&mut fields).unwrap();

                for field in fields {
                    println!("{}: {:?}", field.name, field.value);
                }

                client.write(&mut fields);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}