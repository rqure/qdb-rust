use qdb::ClientTrait;

mod qdb;

fn main() {
    let mut client = qdb::rest::Client::new("http://localhost:20000");
    match client.get_entities("Root") {
        Ok(entities) => {
            for entity in entities {
                let e = client.get_entity("3d66362b-cd4d-4c54-8393-a3b20f3067b8").unwrap();
                println!("Entity: {}", e.entity_name);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}