extern crate serde_json;

mod parse_stream;

fn main() {
    use parse_stream::parse_stream;

    if let Err(e) = parse_stream(r#"{"key1": "value1", "key2": "value2"#) {
        eprintln!("Error: {}", e);
    }
}
