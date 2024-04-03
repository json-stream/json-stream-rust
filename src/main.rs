extern crate serde_json;
use serde_json::{json, Value};

mod tests;

#[derive(Clone, Debug)]
enum ObjectStatus {
    // We are ready to start a new object.
    Ready,
    // We just started the object, likely because we just received an opening brace or a comma in case of an existing object.
    StartObject,
    // We are in the beginning of a key, likely because we just received a quote. We need to store the keySoFar because
    // unlike the value, we cannot add the key to the object until it is complete.
    KeyQuoteOpen {
        KeySoFar: [char; 50],
    },
    // We just finished a key, likely because we just received a closing quote.
    KeyQuoteClose {
        Key: [char; 50],
    },
    // We just finished a key, likely because we just received a colon.
    Colon {
        Key: [char; 50]
    },
    // We are in the beginning of a value, likely because we just received a quote.
    ValueQuoteOpen {
        Key: [char; 50],
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
fn add_char_into_object(object: &mut Option<Value>, current_status: &mut ObjectStatus, current_char: char) -> Result<(), String> {
    match (object.take(), current_status.clone(), current_char) {
        (None, ObjectStatus::Ready, '{') => {
            *object = Some(json!({}));
            *current_status = ObjectStatus::StartObject;
        },
        // implement the rest of the cases
        _ => {
            return Err(format!("Invalid character {}", current_char));
        }
    }

    Ok(())
}

fn main() { 

    let mut object: Option<Value> = None;
    let mut current_status = ObjectStatus::Ready;
    let current_char = '{';
    
    if let Err(e) = add_char_into_object(&mut object, &mut current_status, current_char) {
        eprintln!("error: {}", e);
    } else {
        println!("success {:?}", object);
    }
}
