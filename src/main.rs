use qdb::ClientTrait;

mod qdb;

fn main() {
    let client = qdb::rest::Client::new("http://localhost:8080");
}