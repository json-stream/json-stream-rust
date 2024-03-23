extern crate serde_json;
use serde_json::json;

mod tests;

fn parse_stream(_json_string: &str) -> Option<serde_json::Value> { None }

fn main() { println!("Hello, world!"); }
