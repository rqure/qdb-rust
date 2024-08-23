use qdb::ClientTrait;

mod qdb;

fn main() {
    let mut client = qdb::rest::Client::new("http://localhost:8080");
    client.get_entities("Root").unwrap();
}