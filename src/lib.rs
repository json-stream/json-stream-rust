use serde_json::{json, Value};

#[derive(Clone, Debug)]
enum ObjectStatus {
    // We are ready to start a new object.
    Ready,
    // We are in the beginning of a string, likely because we just received an opening quote.
    StringQuoteOpen,
    // We just finished a string, likely because we just received a closing quote.
    StringQuoteClose,
    // We are in the middle of a scalar value, likely because we just received a digit.
    Scalar {
        value_so_far: Vec<char>,
    },
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
        key: Vec<char>,
    },
    // We are in the beginning of a value, likely because we just received a quote.
    ValueQuoteOpen {
        key: Vec<char>,
        // We don't need to store the valueSoFar because we can add the value to the object immediately.
    },
    ValueQuoteClose,

    // We are taking any value that is not a string. For these case we just store
    // each character until we reach a comma or a closing brace and then we pare
    // and add the value to the object.
    ValueScalar {
        key: Vec<char>,
        value_so_far: Vec<char>,
    },

    // We just finished the object, likely because we just received a closing brace.
    Closed,
}

// this function takes and existing object that we are building along with a single character as we as an address
// to the current position in the object that we are in and returns the object with that character added along with
// the new address.
fn add_char_into_object(
    object: &mut Value,
    current_status: &mut ObjectStatus,
    current_char: char,
) -> Result<(), String> {
    match (object, current_status.clone(), current_char) {
        (val @ Value::Null, ObjectStatus::Ready, '"') => {
            *val = json!("");
            *current_status = ObjectStatus::StringQuoteOpen;
        }
        (val @ Value::Null, ObjectStatus::Ready, '{') => {
            *val = json!({});
            *current_status = ObjectStatus::StartProperty;
        }
        // ------ true ------
        (val @ Value::Null, ObjectStatus::Ready, 't') => {
            *val = json!(true);
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['t'],
            };
        }
        (Value::Bool(true), ObjectStatus::Scalar { ref value_so_far }, 'r')
            if *value_so_far == vec!['t'] =>
        {
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['t', 'r'],
            };
        }
        (Value::Bool(true), ObjectStatus::Scalar { ref value_so_far }, 'u')
            if *value_so_far == vec!['t', 'r'] =>
        {
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['t', 'r', 'u'],
            };
        }
        (Value::Bool(true), ObjectStatus::Scalar { ref value_so_far }, 'e')
            if *value_so_far == vec!['t', 'r', 'u'] =>
        {
            *current_status = ObjectStatus::Closed;
        }
        // ------ false ------
        (val @ Value::Null, ObjectStatus::Ready, 'f') => {
            *val = json!(false);
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['f'],
            };
        }
        (Value::Bool(false), ObjectStatus::Scalar { ref value_so_far }, 'a')
            if *value_so_far == vec!['f'] =>
        {
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['f', 'a'],
            };
        }
        (Value::Bool(false), ObjectStatus::Scalar { ref value_so_far }, 'l')
            if *value_so_far == vec!['f', 'a'] =>
        {
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['f', 'a', 'l'],
            };
        }
        (Value::Bool(false), ObjectStatus::Scalar { ref value_so_far }, 's')
            if *value_so_far == vec!['f', 'a', 'l'] =>
        {
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['f', 'a', 'l', 's'],
            };
        }
        (Value::Bool(false), ObjectStatus::Scalar { ref value_so_far }, 'e')
            if *value_so_far == vec!['f', 'a', 'l', 's'] =>
        {
            *current_status = ObjectStatus::Closed;
        }
        // ------ null ------
        (val @ Value::Null, ObjectStatus::Ready, 'n') => {
            *val = json!(null);
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['n'],
            };
        }
        (Value::Null, ObjectStatus::Scalar { ref value_so_far }, 'u')
            if *value_so_far == vec!['n'] =>
        {
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['n', 'u'],
            };
        }
        (Value::Null, ObjectStatus::Scalar { ref value_so_far }, 'l')
            if *value_so_far == vec!['n', 'u'] =>
        {
            *current_status = ObjectStatus::Scalar {
                value_so_far: vec!['n', 'u', 'l'],
            };
        }
        (Value::Null, ObjectStatus::Scalar { ref value_so_far }, 'l')
            if *value_so_far == vec!['n', 'u', 'l'] =>
        {
            *current_status = ObjectStatus::Closed;
        }
        (Value::String(_str), ObjectStatus::StringQuoteOpen, '"') => {
            *current_status = ObjectStatus::StringQuoteClose;
        }
        (Value::String(str), ObjectStatus::StringQuoteOpen, char) => {
            str.push(char);
            *current_status = ObjectStatus::StringQuoteOpen;
        }
        (Value::Object(_obj), ObjectStatus::StartProperty, '"') => {
            *current_status = ObjectStatus::KeyQuoteOpen { key_so_far: vec![] };
        }
        (Value::Object(ref mut obj), ObjectStatus::KeyQuoteOpen { key_so_far }, '"') => {
            *current_status = ObjectStatus::KeyQuoteClose {
                key: key_so_far.clone(),
            };
            // add the key to the object with null value
            obj.insert(key_so_far.iter().collect::<String>(), Value::Null);
        }
        (Value::Object(_obj), ObjectStatus::KeyQuoteOpen { mut key_so_far }, char) => {
            key_so_far.push(char);
            *current_status = ObjectStatus::KeyQuoteOpen { key_so_far };
        }
        (Value::Object(_obj), ObjectStatus::KeyQuoteClose { key }, ':') => {
            *current_status = ObjectStatus::Colon { key };
        }
        (Value::Object(_obj), ObjectStatus::Colon { .. }, ' ' | '\n') => {}
        (Value::Object(ref mut obj), ObjectStatus::Colon { key }, '"') => {
            *current_status = ObjectStatus::ValueQuoteOpen { key: key.clone() };
            // create an empty string for the value
            obj.insert(key.iter().collect::<String>().clone(), json!(""));
        }
        // ------ Add String Value ------
        (Value::Object(_obj), ObjectStatus::ValueQuoteOpen { key: _key }, '"') => {
            *current_status = ObjectStatus::ValueQuoteClose;
        }
        (Value::Object(ref mut obj), ObjectStatus::ValueQuoteOpen { key }, char) => {
            let key_string = key.iter().collect::<String>();
            let value = obj.get_mut(&key_string).unwrap();
            match value {
                Value::String(value) => {
                    value.push(char);
                }
                _ => {
                    return Err(format!("Invalid value type for key {}", key_string));
                }
            }
        }

        // ------ Add Scalar Value ------
        (Value::Object(_obj), ObjectStatus::Colon { key }, char) => {
            *current_status = ObjectStatus::ValueScalar {
                key,
                value_so_far: vec![char],
            };
        }
        (Value::Object(ref mut obj), ObjectStatus::ValueScalar { key, value_so_far }, ',') => {
            // parse the value and add it to the object
            let key_string = key.iter().collect::<String>();
            let value_string = value_so_far.iter().collect::<String>();
            let value = match value_string.parse::<Value>() {
                Ok(value) => value,
                Err(e) => {
                    return Err(format!("Invalid value for key {}: {}", key_string, e));
                }
            };
            obj.insert(key_string, value);
            *current_status = ObjectStatus::StartProperty;
        }
        (Value::Object(ref mut obj), ObjectStatus::ValueScalar { key, value_so_far }, '}') => {
            // parse the value and add it to the object
            let key_string = key.iter().collect::<String>();
            let value_string = value_so_far.iter().collect::<String>();
            let value = match value_string.parse::<Value>() {
                Ok(value) => value,
                Err(e) => {
                    return Err(format!("Invalid value for key {}: {}", key_string, e));
                }
            };
            obj.insert(key_string, value);
            *current_status = ObjectStatus::Closed;
        }
        (
            Value::Object(_obj),
            ObjectStatus::ValueScalar {
                key: _key,
                mut value_so_far,
            },
            char,
        ) => {
            // push the character into the value so far
            value_so_far.push(char);
            *current_status = ObjectStatus::ValueScalar {
                key: _key,
                value_so_far,
            };
        }

        // ------ Finished taking value ------
        (Value::Object(_obj), ObjectStatus::ValueQuoteClose, ',') => {
            *current_status = ObjectStatus::StartProperty;
        }
        (Value::Object(_obj), ObjectStatus::ValueQuoteClose, '}') => {
            *current_status = ObjectStatus::Closed;
        }

        // ------ white spaces ------
        (_, _, ' ' | '\n') => {}
        (val, st, c) => {
            return Err(format!("Invalid character {} status: {:?}", c, st));
        }
    }

    Ok(())
}

pub fn parse_stream(json_string: &str) -> Result<Value, String> {
    let mut out: Value = Value::Null;
    let mut current_status = ObjectStatus::Ready;
    for current_char in json_string.chars() {
        println!(
            "variables: {:?} {:?} {:?}",
            out,
            current_status.clone(),
            current_char.to_string()
        );
        if let Err(e) = add_char_into_object(&mut out, &mut current_status, current_char) {
            return Err(e);
        }
    }
    return Ok(out);
}

pub struct JsonStreamParser {
    object: Value,
    current_status: ObjectStatus,
}

impl JsonStreamParser {
    pub fn new() -> JsonStreamParser {
        JsonStreamParser {
            object: Value::Null,
            current_status: ObjectStatus::Ready,
        }
    }

    pub fn add_char(&mut self, current_char: char) -> Result<(), String> {
        add_char_into_object(&mut self.object, &mut self.current_status, current_char)
    }

    pub fn get_result(&self) -> &Value {
        &self.object
    }
}

macro_rules! param_test {
    ($($name:ident: $string:expr, $value:expr)*) => {
    $(
        mod $name {
            use super::{parse_stream, JsonStreamParser};
            use serde_json::{Value, json};

            #[test]
            fn simple() {
                let string: &str = $string;
                let value: Value = $value;
                let result = parse_stream(&string);
                assert_eq!(result.unwrap(), value);
                let mut parser = JsonStreamParser::new();
                for c in string.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &value);
            }

            #[test]
            fn object_single_key_value() {
                let string = $string;
                let value = $value;
                let raw_json = format!("{{\"key\": {}}}", string);
                let expected = json!({"key": value});
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn object_multiple_key_value() {
                let string = $string;
                let value = $value;
                let raw_json = format!("{{\"key1\": {}, \"key2\": {}}}", string, string);
                let expected = json!({"key1": value, "key2": value});
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn object_multiple_key_value_with_blank_1() {
                let string = $string;
                let value = $value;
                let raw_json = format!("{{ \"key1\": {}, \"key2\": {}}}", string, string);
                let expected = json!({"key1": value, "key2": value});
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn object_multiple_key_value_with_blank_2() {
                let string = $string;
                let value = $value;
                let raw_json = format!("{{\"key1\": {}, \"key2\": {} }}", string, string);
                let expected = json!({"key1": value, "key2": value});
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn object_multiple_key_value_with_blank_3() {
                let string = $string;
                let value = $value;
                let raw_json = format!("{{ 
                    \"key1\": {} , 
                     \"key2\": {} 
                }}", string, string);
                let expected = json!({"key1": value, "key2": value});
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }
        }
    )*
    }
}

param_test! {
    null: r#"null"#, Value::Null
    true_value: r#"true"#, Value::Bool(true)
    false_value: r#"false"#, Value::Bool(false)
    empty_string: r#""""#, Value::String("".to_string())
    single_character_string: r#""a""#, Value::String("a".to_string())
    string_with_spaces: r#""a b c""#, Value::String("a b c".to_string())
    string_with_space_at_end: r#""a b c ""#, Value::String("a b c ".to_string())
    string_with_space_at_start: r#"" a b c""#, Value::String(" a b c".to_string())
    string_with_space_at_start_and_end: r#"" a b c ""#, Value::String(" a b c ".to_string())
    number: r#"1234567890"#, Value::Number(1234567890.into())
    negative_number: r#"-1234567890"#, Value::Number((-1234567890).into())
    zero: r#"0"#, Value::Number(0.into())
}
