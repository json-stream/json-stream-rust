extern crate serde_json;
use serde_json::{json, Value};

mod tests;

#[derive(Clone, Debug)]
enum ObjectStatus {
    // We are ready to start a new object.
    Ready,
    // We just started a property, likely because we just received an opening brace or a comma in case of an existing object.
    StartProperty,
    // We are in the beginning of a key, likely because we just received a quote. We need to store the key_so_far because
    // unlike the value, we cannot add the key to the object until it is complete.
    KeyQuoteOpen {
        key_so_far: Vec<char>,
    },
    // We just finished a key, likely because we just received a closing quote.
    KeyQuoteClose {
        key: Vec<char>,
    },
    // We just finished a key, likely because we just received a colon.
    Colon {
        key: Vec<char>
    },
    // We are in the beginning of a value, likely because we just received a quote.
    ValueQuoteOpen {
        key: Vec<char>,
        // We don't need to store the valueSoFar because we can add the value to the object immediately.
    },
    ValueQuoteClose,
    // We just finished the object, likely because we just received a closing brace.
    Closed,
    // We are taking number value
    ValueNumber {
        key: Vec<char>,
    },
}

fn parse_stream(json_string: &str) -> Result<Option<Value>, String> { 
    let mut out: Option<Value> = None;
    let mut current_status = ObjectStatus::Ready;
    for current_char in json_string.chars() {
        println!("variables: {:?} {:?} {:?}", out, current_status.clone(), current_char.to_string());
        if let Err(e) = add_char_into_object(&mut out, &mut current_status, current_char) {
            return Err(e);
        }
    }
    return Ok(out);
}

// this function takes and existing object that we are building along with a single character as we as an address
// to the current position in the object that we are in and returns the object with that character added along with
// the new address.
fn add_char_into_object(object: &mut Option<Value>, current_status: &mut ObjectStatus, current_char: char) -> Result<(), String> {
    match (object.clone(), current_status.clone(), current_char) {
        (None, ObjectStatus::Ready, '{') => {
            *object = Some(json!({}));
            *current_status = ObjectStatus::StartProperty;
        },
        (_, _, ' ') => {
            // ignore whitespace
        },
        (Some(Value::Object(_obj)), ObjectStatus::StartProperty, '"') => {
            *current_status = ObjectStatus::KeyQuoteOpen { key_so_far: vec![] };
        },
        (Some(Value::Object(mut obj)), ObjectStatus::KeyQuoteOpen { key_so_far }, '"') => {
            *current_status = ObjectStatus::KeyQuoteClose { key: key_so_far.clone() };
            // add the key to the object with null value
            obj.insert(key_so_far.iter().collect::<String>(), Value::Null);
            *object = Some(Value::Object(obj));
        },
        (Some(Value::Object(_obj)), ObjectStatus::KeyQuoteOpen { mut key_so_far }, char) => {
            key_so_far.push(char);
            *current_status = ObjectStatus::KeyQuoteOpen { key_so_far };
        },
        (Some(Value::Object(_obj)), ObjectStatus::KeyQuoteClose{ key }, ':') => {
            *current_status = ObjectStatus::Colon { key };
        },
        (Some(Value::Object(mut obj)), ObjectStatus::Colon { key }, '"') => {
            *current_status = ObjectStatus::ValueQuoteOpen { key: key.clone() };
            // create an empty string for the value
            obj.insert(key.iter().collect::<String>().clone(), json!(""));
            *object = Some(Value::Object(obj));
        },
        // ------ Add String Value ------
        (Some(Value::Object(_obj)), ObjectStatus::ValueQuoteOpen { key: _key }, '"') => {
            *current_status = ObjectStatus::ValueQuoteClose;
        },
        (Some(Value::Object(mut obj)), ObjectStatus::ValueQuoteOpen { key }, char) => {
            let key_string = key.iter().collect::<String>();
            let value = obj.get_mut(&key_string).unwrap();
            match value {
                Value::String(value) => {
                    value.push(char);
                },
                _ => {
                    return Err(format!("Invalid value type for key {}", key_string));
                }
            }
            *object = Some(Value::Object(obj));
        },
        // ------ Add Number Value ------
        (Some(Value::Object(mut obj)), ObjectStatus::Colon { key }, '1'..='9') => {
            *current_status = ObjectStatus::ValueNumber { key: key.clone() };
            obj.insert(key.iter().collect::<String>(), json!(current_char.to_digit(10) ));
            *object = Some(Value::Object(obj));
        },
        (Some(Value::Object(mut obj)), ObjectStatus::ValueNumber { key }, '0'..='9') => {
            let key_string = key.iter().collect::<String>();
            let value = obj.get_mut(&key_string).unwrap();
            match value {
                Value::Number(value) => {
                    let new_value = value.as_i64().unwrap() * 10 + current_char.to_digit(10).unwrap() as i64;
                    *value = new_value.into();
                },
                _ => {
                    return Err(format!("Invalid value type for key {}", key_string));
                }
            }
            *object = Some(Value::Object(obj));
        },

        // ------ Finished taking value ------
        (Some(Value::Object(_obj)), ObjectStatus::ValueQuoteClose | ObjectStatus::ValueNumber { .. }, ',') => {
            *current_status = ObjectStatus::StartProperty;
        },
        (Some(Value::Object(_obj)), ObjectStatus::ValueQuoteClose | ObjectStatus::ValueNumber { .. }, '}') => {
            *current_status = ObjectStatus::Closed;
        },
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
