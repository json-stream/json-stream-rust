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
    ScalarNumber {
        value_so_far: Vec<char>,
    },
    // We just started an array, after receiving an opening bracket.
    StartArray,
    // We are in the beginning of an array string value.
    ArrayValueQuoteOpen { index: usize },
    // We just closed an array string value.
    ArrayValueQuoteClose { index: usize },
    // We are taking an array scalar value (numbers, booleans, etc.).
    ArrayValueScalar {
        index: usize,
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
    match (object, current_status, current_char) {
        (val @ Value::Null, sts @ ObjectStatus::Ready, '"') => {
            *val = json!("");
            *sts = ObjectStatus::StringQuoteOpen;
        }
        (val @ Value::Null, sts @ ObjectStatus::Ready, '{') => {
            *val = json!({});
            *sts = ObjectStatus::StartProperty;
        }
        (val @ Value::Null, sts @ ObjectStatus::Ready, '[') => {
            *val = json!([]);
            *sts = ObjectStatus::StartArray;
        }
        // ------ true ------
        (val @ Value::Null, sts @ ObjectStatus::Ready, 't') => {
            *val = json!(true);
            *sts = ObjectStatus::Scalar {
                value_so_far: vec!['t'],
            };
        }
        (
            Value::Bool(true),
            ObjectStatus::Scalar {
                ref mut value_so_far,
            },
            'r',
        ) if *value_so_far == vec!['t'] => {
            value_so_far.push('r');
        }
        (
            Value::Bool(true),
            ObjectStatus::Scalar {
                ref mut value_so_far,
            },
            'u',
        ) if *value_so_far == vec!['t', 'r'] => {
            value_so_far.push('u');
        }
        (Value::Bool(true), sts @ ObjectStatus::Scalar { .. }, 'e') => {
            *sts = ObjectStatus::Closed;
        }
        // ------ false ------
        (val @ Value::Null, sts @ ObjectStatus::Ready, 'f') => {
            *val = json!(false);
            *sts = ObjectStatus::Scalar {
                value_so_far: vec!['f'],
            };
        }
        (
            Value::Bool(false),
            ObjectStatus::Scalar {
                ref mut value_so_far,
            },
            'a',
        ) if *value_so_far == vec!['f'] => {
            value_so_far.push('a');
        }
        (
            Value::Bool(false),
            ObjectStatus::Scalar {
                ref mut value_so_far,
            },
            'l',
        ) if *value_so_far == vec!['f', 'a'] => {
            value_so_far.push('l');
        }
        (
            Value::Bool(false),
            ObjectStatus::Scalar {
                ref mut value_so_far,
            },
            's',
        ) if *value_so_far == vec!['f', 'a', 'l'] => {
            value_so_far.push('s');
        }
        (Value::Bool(false), sts @ ObjectStatus::Scalar { .. }, 'e') => {
            *sts = ObjectStatus::Closed;
        }
        // ------ null ------
        (val @ Value::Null, sts @ ObjectStatus::Ready, 'n') => {
            *val = json!(null);
            *sts = ObjectStatus::Scalar {
                value_so_far: vec!['n'],
            };
        }
        (
            Value::Null,
            ObjectStatus::Scalar {
                ref mut value_so_far,
            },
            'u',
        ) if *value_so_far == vec!['n'] => {
            value_so_far.push('u');
        }
        (
            Value::Null,
            ObjectStatus::Scalar {
                ref mut value_so_far,
            },
            'l',
        ) if *value_so_far == vec!['n', 'u'] => {
            value_so_far.push('l');
        }
        (Value::Null, sts @ ObjectStatus::Scalar { .. }, 'l') => {
            *sts = ObjectStatus::Closed;
        }
        // ------ number ------
        (val @ Value::Null, sts @ ObjectStatus::Ready, c @ '0'..='9') => {
            *val = Value::Number(c.to_digit(10).unwrap().into());
            *sts = ObjectStatus::ScalarNumber {
                value_so_far: vec![c],
            };
        }
        (val @ Value::Null, sts @ ObjectStatus::Ready, '-') => {
            *val = Value::Number(0.into());
            *sts = ObjectStatus::ScalarNumber {
                value_so_far: vec!['-'],
            };
        }
        (
            Value::Number(ref mut num),
            ObjectStatus::ScalarNumber {
                ref mut value_so_far,
            },
            c @ '0'..='9',
        ) => {
            value_so_far.push(c);
            // if there are any . in the value so far, then we need to parse the number as a float
            if value_so_far.contains(&'.') {
                let parsed_number = value_so_far
                    .iter()
                    .collect::<String>()
                    .parse::<f64>()
                    .unwrap();

                if let Some(json_number) = serde_json::Number::from_f64(parsed_number) {
                    *num = json_number;
                }
            } else {
                let parsed_number = value_so_far
                    .iter()
                    .collect::<String>()
                    .parse::<i64>()
                    .unwrap();
                *num = parsed_number.into();
            }
        }
        (
            Value::Number(_),
            ObjectStatus::ScalarNumber {
                ref mut value_so_far,
            },
            '.',
        ) => {
            value_so_far.push('.');
        }
        // ------ array ------
        (val @ Value::Array(_), sts @ ObjectStatus::StartArray, ']') => {
            *sts = ObjectStatus::Closed;
        }
        (Value::Array(_), ObjectStatus::StartArray, ' ' | '\n') => {}
        (Value::Array(ref mut arr), sts @ ObjectStatus::StartArray, '"') => {
            arr.push(json!(""));
            *sts = ObjectStatus::ArrayValueQuoteOpen { index: arr.len() - 1 };
        }
        (Value::Array(ref mut arr), sts @ ObjectStatus::StartArray, char) => {
            arr.push(Value::Null);
            *sts = ObjectStatus::ArrayValueScalar {
                index: arr.len() - 1,
                value_so_far: vec![char],
            };
        }
        (Value::Array(_), sts @ ObjectStatus::ArrayValueQuoteOpen { index: _ }, '"') => {
            if let ObjectStatus::ArrayValueQuoteOpen { index } = sts.clone() {
                *sts = ObjectStatus::ArrayValueQuoteClose { index };
            }
        }
        (
            Value::Array(ref mut arr),
            ObjectStatus::ArrayValueQuoteOpen { index },
            char,
        ) => {
            if let Some(Value::String(s)) = arr.get_mut(*index) {
                s.push(char);
            } else {
                return Err("Invalid string value in array".to_string());
            }
        }
        (Value::Array(_), ObjectStatus::ArrayValueQuoteClose { .. }, ' ' | '\n') => {}
        (Value::Array(_), sts @ ObjectStatus::ArrayValueQuoteClose { .. }, ',') => {
            *sts = ObjectStatus::StartArray;
        }
        (Value::Array(_), sts @ ObjectStatus::ArrayValueQuoteClose { .. }, ']') => {
            *sts = ObjectStatus::Closed;
        }
        (
            Value::Array(ref mut arr),
            sts @ ObjectStatus::ArrayValueScalar { .. },
            ','
        ) => {
            if let ObjectStatus::ArrayValueScalar { index, ref mut value_so_far } = sts {
                let value_string = value_so_far.iter().collect::<String>();
                let idx = *index;
                let value: Value = match value_string.parse::<Value>() {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Invalid array value: {}", e)),
                };
                arr[idx] = value;
            }
            *sts = ObjectStatus::StartArray;
        }
        (
            Value::Array(ref mut arr),
            sts @ ObjectStatus::ArrayValueScalar { .. },
            ']'
        ) => {
            if let ObjectStatus::ArrayValueScalar { index, ref mut value_so_far } = sts {
                let value_string = value_so_far.iter().collect::<String>();
                let idx = *index;
                let value: Value = match value_string.parse::<Value>() {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Invalid array value: {}", e)),
                };
                arr[idx] = value;
            }
            *sts = ObjectStatus::Closed;
        }
        (
            Value::Array(_),
            ObjectStatus::ArrayValueScalar { ref mut value_so_far, .. },
            char,
        ) => {
            value_so_far.push(char);
        }
        // ------ string ------
        (Value::String(_str), sts @ ObjectStatus::StringQuoteOpen, '"') => {
            *sts = ObjectStatus::StringQuoteClose;
        }
        (Value::String(str), sts @ ObjectStatus::StringQuoteOpen, char) => {
            str.push(char);
            *sts = ObjectStatus::StringQuoteOpen;
        }
        (Value::Object(_obj), sts @ ObjectStatus::StartProperty, '"') => {
            *sts = ObjectStatus::KeyQuoteOpen { key_so_far: vec![] };
        }
        (Value::Object(ref mut obj), sts @ ObjectStatus::KeyQuoteOpen { .. }, '"') => {
            if let ObjectStatus::KeyQuoteOpen { key_so_far } = sts.clone() {
                *sts = ObjectStatus::KeyQuoteClose {
                    key: key_so_far.clone(),
                };
                obj.insert(key_so_far.iter().collect::<String>(), Value::Null);
            }
        }
        (Value::Object(_obj), ObjectStatus::KeyQuoteOpen { ref mut key_so_far }, char) => {
            key_so_far.push(char);
        }
        (Value::Object(_obj), sts @ ObjectStatus::KeyQuoteClose { .. }, ':') => {
            if let ObjectStatus::KeyQuoteClose { key } = sts.clone() {
                *sts = ObjectStatus::Colon { key: key.clone() };
            }
        }
        (Value::Object(_obj), ObjectStatus::Colon { .. }, ' ' | '\n') => {}
        (Value::Object(ref mut obj), sts @ ObjectStatus::Colon { .. }, '"') => {
            if let ObjectStatus::Colon { key } = sts.clone() {
                *sts = ObjectStatus::ValueQuoteOpen { key: key.clone() };
                // create an empty string for the value
                obj.insert(key.iter().collect::<String>().clone(), json!(""));
            }
        }
        // ------ Add String Value ------
        (Value::Object(_obj), sts @ ObjectStatus::ValueQuoteOpen { .. }, '"') => {
            *sts = ObjectStatus::ValueQuoteClose;
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
        (Value::Object(_obj), sts @ ObjectStatus::Colon { .. }, char) => {
            if let ObjectStatus::Colon { key } = sts.clone() {
                *sts = ObjectStatus::ValueScalar {
                    key: key.clone(),
                    value_so_far: vec![char],
                };
            }
        }
        (Value::Object(ref mut obj), sts @ ObjectStatus::ValueScalar { .. }, ',') => {
            if let ObjectStatus::ValueScalar { key, value_so_far } = sts.clone() {
                let key_string = key.iter().collect::<String>();
                let value_string = value_so_far.iter().collect::<String>();
                let value = match value_string.parse::<Value>() {
                    Ok(value) => value,
                    Err(e) => {
                        return Err(format!("Invalid value for key {}: {}", key_string, e));
                    }
                };
                obj.insert(key_string, value);
                *sts = ObjectStatus::StartProperty;
            }
        }
        (Value::Object(ref mut obj), sts @ ObjectStatus::ValueScalar { .. }, '}') => {
            if let ObjectStatus::ValueScalar { key, value_so_far } = sts.clone() {
                let key_string = key.iter().collect::<String>();
                let value_string = value_so_far.iter().collect::<String>();
                let value = match value_string.parse::<Value>() {
                    Ok(value) => value,
                    Err(e) => {
                        return Err(format!("Invalid value for key {}: {}", key_string, e));
                    }
                };
                obj.insert(key_string, value);
                *sts = ObjectStatus::Closed;
            }
        }
        (
            Value::Object(_obj),
            ObjectStatus::ValueScalar {
                key: _key,
                ref mut value_so_far,
            },
            char,
        ) => {
            // push the character into the value so far
            value_so_far.push(char);
        }

        // ------ Finished taking value ------
        (Value::Object(_obj), sts @ ObjectStatus::ValueQuoteClose, ',') => {
            *sts = ObjectStatus::StartProperty;
        }
        (Value::Object(_obj), sts @ ObjectStatus::ValueQuoteClose, '}') => {
            *sts = ObjectStatus::Closed;
        }
        // ------ white spaces ------
        (_, _, ' ' | '\n') => {}
        (_val, st, c) => {
            return Err(format!("Invalid character {} status: {:?}", c, st));
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
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

#[cfg(not(debug_assertions))]
pub fn parse_stream(json_string: &str) -> Result<Value, String> {
    let mut out: Value = Value::Null;
    let mut current_status = ObjectStatus::Ready;
    for current_char in json_string.chars() {
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

// The `param_test!` macro defines a suite of parameterized tests. Each entry
// provides a JSON snippet (`$string`) and the `serde_json::Value` (`$value`)
// expected after parsing. The macro expands into a module per entry containing
// multiple tests:
//   * parsing the snippet on its own
//   * the snippet as a value in objects
//   * the snippet embedded inside arrays
// This approach ensures consistent coverage across many JSON value types.
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

            #[test]
            fn array_single_value() {
                let string = $string;
                let value = $value;
                let raw_json = format!("[{}]", string);
                let expected = json!([value]);
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn array_multiple_values() {
                let string = $string;
                let value = $value;
                let raw_json = format!("[{}, {}]", string, string);
                let expected = json!([value.clone(), value]);
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn array_multiple_values_with_blank_1() {
                let string = $string;
                let value = $value;
                let raw_json = format!("[ {}, {}]", string, string);
                let expected = json!([value.clone(), value]);
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn array_multiple_values_with_blank_2() {
                let string = $string;
                let value = $value;
                let raw_json = format!("[{}, {} ]", string, string);
                let expected = json!([value.clone(), value]);
                let result = parse_stream(&raw_json);
                assert_eq!(result.unwrap(), expected);
                let mut parser = JsonStreamParser::new();
                for c in raw_json.chars() {
                    parser.add_char(c);
                }
                assert_eq!(parser.get_result(), &expected);
            }

            #[test]
            fn array_multiple_values_with_blank_3() {
                let string = $string;
                let value = $value;
                let raw_json = format!("[\n    {},\n    {}\n]", string, string);
                let expected = json!([value.clone(), value]);
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
    single_digit_number: r#"1"#, Value::Number(1.into())
    number_with_spaces_at_start: r#" 1234567890"#, Value::Number(1234567890.into())
    number_with_spaces_at_end: r#"1234567890 "#, Value::Number(1234567890.into())
    number_with_spaces_at_start_and_end: r#" 1234567890 "#, Value::Number(1234567890.into())
    negative_number: r#"-1234567890"#, Value::Number((-1234567890).into())
    negative_single_digit_number: r#"-1"#, Value::Number((-1).into())
    zero: r#"0"#, Value::Number(0.into())
    float: r#"123.456"#, Value::Number(serde_json::Number::from_f64(123.456).unwrap())
    negative_float: r#"-123.456"#, Value::Number(serde_json::Number::from_f64(-123.456).unwrap())
}

#[cfg(test)]
mod array_tests {
    use super::{parse_stream, JsonStreamParser};
    use serde_json::json;

    #[test]
    fn empty_array() {
        let raw_json = "[]";
        let expected = json!([]);
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }

    #[test]
    fn array_as_object_value() {
        let raw_json = "{\"items\": []}";
        let expected = json!({"items": []});
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }
}
