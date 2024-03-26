extern crate serde_json;
use serde_json::{json, Value};

mod tests;

enum ObjectStatus {
    // We are ready to start a new object.
    Ready,
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

fn parse_stream(_json_string: &str) -> Result<Value, String> { 
    // TODO: reverse the string and feed the characters to the add_char_into_object
    let out = json!({}) ; 
    return Ok(out);
}

// this function takes and existing object that we are building along with a single character as we as an address
// to the current position in the object that we are in and returns the object with that character added along with
// the new address.
fn add_char_into_object(object: Option< &mut Value>, current_status: &mut ObjectStatus, current_char: char) -> Result<(), String> {
    match (object, current_status, current_char) {
        (None, ObjectStatus::Ready, '{') => {
            *object = Some(Value::Object(serde_json::Map::new()));
            *current_status = ObjectStatus::StartObject;
        },
        (Some(Value::Object(_)), ObjectStatus::StartObject, '"') => {
            *current_status = ObjectStatus::KeyQuoteOpen { keySoFar: String::new() };
        },
        (Some(Value::Object(_)), ObjectStatus::StartObject, '}') => {
            // We are done with the object.
            return Ok(());
        },
        (Some(Value::Object(_)), ObjectStatus::StartObject, _) => {
            // We are in the middle of a key.
            return Err("Expected a quote to start the key".into());
        },
        (Some(Value::Object(_)), ObjectStatus::KeyQuoteOpen { keySoFar }, '"') => {
            *current_status = ObjectStatus::KeyQuoteClose { key: keySoFar.clone() };
        },
        (Some(Value::Object(_)), ObjectStatus::KeyQuoteOpen { keySoFar }, _) => {
            // We are in the middle of a key.
            let mut keySoFar = keySoFar.clone();
            keySoFar.push(current_char);
            *current_status = ObjectStatus::KeyQuoteOpen { keySoFar };
        },
        // (Some(Value::Object(_)), ObjectStatus::KeyQuoteClose { key }, ':') => {
        //     *current_status = ObjectStatus::Colon { key };
        // },
        // (Some(Value::Object(_)), ObjectStatus::KeyQuoteClose { key }, _) => {
        //     // We are in the middle of a key.
        //     return Err("Expected a colon after the key".into());
        // },
        // (Some(Value::Object(_)), ObjectStatus::Colon { key }, '"') => {
        //     *current_status = ObjectStatus::ValueQuoteOpen { key, };
        // },
        (Some(Value::Object(_)), ObjectStatus::Colon { key }, _) => {
            // We are in the middle of a key.
            return Err("Expected a quote to start the value".into());
        },
        _ => {
            return Err("Unexpected character".into());
        },
    }
    Ok(())
}

fn main() { 

    let mut object: Option<&mut Value> = None;
    let mut current_status = ObjectStatus::Ready;
    let current_char = '{';
    
    if let Err(e) = add_char_into_object(object, &mut current_status, current_char) {
        eprintln!("error: {}", e);
    } else {
        println!("success {:?}", object);
    }
}
