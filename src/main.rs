extern crate serde_json;
use serde_json::{json, Result};

mod tests;

enum ObjectStatus {
    // We just started the object, likely because we just received an opening brace or a comma in case of an existing object.
    StartObject,
    // We are in the beginning of a key, likely because we just received a quote. We need to store the keySoFar because
    // unlike the value, we cannot add the key to the object until it is complete.
    KeyQuoteOpen {
        keySoFar: String,
    },
    // We just finished a key, likely because we just received a closing quote.
    KeyQuoteClose {
        key: String,
    },
    // We just finished a key, likely because we just received a colon.
    Colon {
        key: String
    },
    // We are in the beginning of a value, likely because we just received a quote.
    ValueQuoteOpen {
        key: String,
        // We don't need to store the valueSoFar because we can add the value to the object immediately.
    },
    // We just finished a value, likely because we just received a closing quote.
    ValueQuoteClose,
}

fn parse_stream(_json_string: &str) -> Result<serde_json::Value> { 
    // TODO: reverse the string and feed the characters to the add_char_into_object
    let out = json!({}) ; 
    return Ok(out);
}

// this function takes and existing object that we are building along with a single character as we as an address
// to the current position in the object that we are in and returns the object with that character added along with
// the new address.
fn add_char_into_object(object: &mut serde_json::Value, current_status: &mut ObjectStatus, current_char: &char) -> Result<()> {
    Ok(())
}

fn main() { println!("Hello, world!"); }
