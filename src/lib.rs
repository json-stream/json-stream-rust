use serde_json::{json, Value};

fn ends_with_odd_backslashes(s: &str) -> bool {
    s.chars().rev().take_while(|&c| c == '\\').count() % 2 == 1
}

fn decode_json_string(raw: &str) -> Result<String, String> {
    serde_json::from_str::<String>(&format!("\"{raw}\"")).map_err(|e| e.to_string())
}

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
    ArrayValueQuoteOpen {
        index: usize,
    },
    // We just closed an array string value.
    ArrayValueQuoteClose,
    // We are taking an array scalar value (numbers, booleans, etc.).
    ArrayValueScalar {
        index: usize,
        value_so_far: Vec<char>,
    },
    // A nested object/array being parsed inside an array
    ArrayValueNested,
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
    // A nested object/array being parsed as a property value
    ValueNested {
        key: Vec<char>,
    },

    // We just finished the object, likely because we just received a closing brace.
    Closed,
}

// this function takes and existing object that we are building along with a single character as we as an address
// to the current position in the object that we are in and returns the object with that character added along with
// the new address.
fn process_char(
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
            let digit = c.to_digit(10).ok_or_else(|| "invalid digit".to_string())?;
            *val = Value::Number(digit.into());
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
            // if the number contains a decimal point or an exponent, parse as f64
            if value_so_far.contains(&'.')
                || value_so_far.contains(&'e')
                || value_so_far.contains(&'E')
            {
                let parsed_number = value_so_far
                    .iter()
                    .collect::<String>()
                    .parse::<f64>()
                    .map_err(|e| e.to_string())?;

                if let Some(json_number) = serde_json::Number::from_f64(parsed_number) {
                    *num = json_number;
                }
            } else {
                let parsed_number = value_so_far
                    .iter()
                    .collect::<String>()
                    .parse::<i64>()
                    .map_err(|e| e.to_string())?;
                *num = parsed_number.into();
            }
        }
        (
            Value::Number(_),
            ObjectStatus::ScalarNumber {
                ref mut value_so_far,
            },
            'e' | 'E',
        ) => {
            value_so_far.push(current_char);
        }
        (
            Value::Number(_),
            ObjectStatus::ScalarNumber {
                ref mut value_so_far,
            },
            '+' | '-',
        ) if matches!(value_so_far.last(), Some('e') | Some('E')) => {
            value_so_far.push(current_char);
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
        (Value::Array(_), sts @ ObjectStatus::StartArray, ']') => {
            *sts = ObjectStatus::Closed;
        }
        (Value::Array(_), ObjectStatus::StartArray, ' ' | '\n') => {}
        (Value::Array(ref mut arr), sts @ ObjectStatus::StartArray, '"') => {
            arr.push(json!(""));
            *sts = ObjectStatus::ArrayValueQuoteOpen {
                index: arr.len() - 1,
            };
        }
        (Value::Array(ref mut arr), sts @ ObjectStatus::StartArray, char) => {
            arr.push(Value::Null);
            *sts = ObjectStatus::ArrayValueScalar {
                index: arr.len() - 1,
                value_so_far: vec![char],
            };
        }
        (Value::Array(ref mut arr), sts @ ObjectStatus::ArrayValueQuoteOpen { .. }, '"') => {
            let index = match *sts {
                ObjectStatus::ArrayValueQuoteOpen { index } => index,
                _ => unreachable!(),
            };
            if let Some(Value::String(s)) = arr.get_mut(index) {
                if ends_with_odd_backslashes(s) {
                    s.push('"');
                    return Ok(());
                }
                *s = decode_json_string(s)?;
            }
            *sts = ObjectStatus::ArrayValueQuoteClose;
        }
        (Value::Array(ref mut arr), ObjectStatus::ArrayValueQuoteOpen { index }, char) => {
            if let Some(Value::String(s)) = arr.get_mut(*index) {
                s.push(char);
            } else {
                return Err("Invalid string value in array".to_string());
            }
        }
        (Value::Array(_), ObjectStatus::ArrayValueQuoteClose, ' ' | '\n') => {}
        (Value::Array(_), sts @ ObjectStatus::ArrayValueQuoteClose, ',') => {
            *sts = ObjectStatus::StartArray;
        }
        (Value::Array(_), sts @ ObjectStatus::ArrayValueQuoteClose, ']') => {
            *sts = ObjectStatus::Closed;
        }
        (Value::Array(ref mut arr), sts @ ObjectStatus::ArrayValueScalar { .. }, ',') => {
            if let ObjectStatus::ArrayValueScalar {
                index,
                ref mut value_so_far,
            } = sts
            {
                let value_string = value_so_far.iter().collect::<String>();
                let idx = *index;
                let value: Value = match value_string.parse::<Value>() {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Invalid array value: {e}")),
                };
                arr[idx] = value;
            }
            *sts = ObjectStatus::StartArray;
        }
        (Value::Array(ref mut arr), sts @ ObjectStatus::ArrayValueScalar { .. }, ']') => {
            if let ObjectStatus::ArrayValueScalar {
                index,
                ref mut value_so_far,
            } = sts
            {
                let value_string = value_so_far.iter().collect::<String>();
                let idx = *index;
                let value: Value = match value_string.parse::<Value>() {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Invalid array value: {e}")),
                };
                arr[idx] = value;
            }
            *sts = ObjectStatus::Closed;
        }
        (
            Value::Array(_),
            ObjectStatus::ArrayValueScalar {
                ref mut value_so_far,
                ..
            },
            char,
        ) => {
            value_so_far.push(char);
        }
        // ------ string ------
        (Value::String(ref mut s), sts @ ObjectStatus::StringQuoteOpen, '"') => {
            if ends_with_odd_backslashes(s) {
                s.push('"');
            } else {
                *s = decode_json_string(s)?;
                *sts = ObjectStatus::StringQuoteClose;
            }
        }
        (Value::String(str), sts @ ObjectStatus::StringQuoteOpen, char) => {
            str.push(char);
            *sts = ObjectStatus::StringQuoteOpen;
        }
        (Value::Object(_obj), sts @ ObjectStatus::StartProperty, '"') => {
            *sts = ObjectStatus::KeyQuoteOpen { key_so_far: vec![] };
        }
        (Value::Object(_obj), sts @ ObjectStatus::StartProperty, '}') => {
            *sts = ObjectStatus::Closed;
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
        (Value::Object(_obj), ObjectStatus::Colon { .. }, ' ' | '\n' | '\t' | '\r') => {}
        (Value::Object(ref mut obj), sts @ ObjectStatus::Colon { .. }, '"') => {
            if let ObjectStatus::Colon { key } = sts.clone() {
                *sts = ObjectStatus::ValueQuoteOpen { key: key.clone() };
                // create an empty string for the value
                obj.insert(key.iter().collect::<String>().clone(), json!(""));
            }
        }
        // ------ Add String Value ------
        (Value::Object(ref mut obj), sts @ ObjectStatus::ValueQuoteOpen { .. }, '"') => {
            let key_vec = match sts {
                ObjectStatus::ValueQuoteOpen { key } => key.clone(),
                _ => unreachable!(),
            };
            let key_string = key_vec.iter().collect::<String>();
            if let Some(Value::String(value)) = obj.get_mut(&key_string) {
                if ends_with_odd_backslashes(value) {
                    value.push('"');
                    return Ok(());
                }
                *value = decode_json_string(value)?;
            }
            *sts = ObjectStatus::ValueQuoteClose;
        }
        (Value::Object(ref mut obj), ObjectStatus::ValueQuoteOpen { key }, char) => {
            let key_string = key.iter().collect::<String>();
            let value = obj
                .get_mut(&key_string)
                .ok_or_else(|| format!("missing key {key_string}"))?;
            match value {
                Value::String(value) => {
                    value.push(char);
                }
                _ => {
                    return Err(format!("Invalid value type for key {key_string}"));
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
                        return Err(format!("Invalid value for key {key_string}: {e}"));
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
                        return Err(format!("Invalid value for key {key_string}: {e}"));
                    }
                };
                obj.insert(key_string, value);
                *sts = ObjectStatus::Closed;
            }
        }
        (Value::Object(_obj), ObjectStatus::ValueScalar { .. }, ' ' | '\n' | '\t' | '\r') => {}
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
        (_, _, ' ' | '\n' | '\t' | '\r') => {}
        (_val, st, c) => {
            return Err(format!("Invalid character {c} status: {st:?}"));
        }
    }

    Ok(())
}

fn add_char_into_object(
    stack: &mut Vec<(Value, ObjectStatus)>,
    current_char: char,
) -> Result<(), String> {
    if stack.is_empty() {
        return Err("empty stack".to_string());
    }

    // check if we need to start a nested context without cloning status
    {
        let (value, status) = stack.last_mut().unwrap();
        match (value, current_char) {
            (Value::Array(_), '{') if matches!(status, ObjectStatus::StartArray) => {
                *status = ObjectStatus::ArrayValueNested;
                stack.push((Value::Null, ObjectStatus::Ready));
                return add_char_into_object(stack, '{');
            }
            (Value::Array(_), '[') if matches!(status, ObjectStatus::StartArray) => {
                *status = ObjectStatus::ArrayValueNested;
                stack.push((Value::Null, ObjectStatus::Ready));
                return add_char_into_object(stack, '[');
            }
            (Value::Object(_), '{') => {
                if let ObjectStatus::Colon { key } = status {
                    let key_clone = key.clone();
                    *status = ObjectStatus::ValueNested {
                        key: key_clone.clone(),
                    };
                    stack.push((Value::Null, ObjectStatus::Ready));
                    return add_char_into_object(stack, '{');
                }
            }
            (Value::Object(_), '[') => {
                if let ObjectStatus::Colon { key } = status {
                    let key_clone = key.clone();
                    *status = ObjectStatus::ValueNested {
                        key: key_clone.clone(),
                    };
                    stack.push((Value::Null, ObjectStatus::Ready));
                    return add_char_into_object(stack, '[');
                }
            }
            _ => {}
        }
    }

    {
        let (ref mut val, ref mut status) = stack.last_mut().unwrap();
        process_char(val, status, current_char)?;
    }

    // handle closed contexts
    while stack.len() > 1 {
        let should_pop = matches!(stack.last(), Some((_, ObjectStatus::Closed)));
        if !should_pop {
            break;
        }
        let (completed_value, _) = stack.pop().unwrap();
        let (parent_value, parent_status) = stack.last_mut().unwrap();
        match parent_status {
            ObjectStatus::ArrayValueNested => {
                if let Value::Array(arr) = parent_value {
                    arr.push(completed_value);
                    *parent_status = ObjectStatus::ArrayValueQuoteClose;
                } else {
                    return Err("Parent is not array".to_string());
                }
            }
            ObjectStatus::ValueNested { key } => {
                if let Value::Object(map) = parent_value {
                    map.insert(key.iter().collect::<String>(), completed_value);
                    *parent_status = ObjectStatus::ValueQuoteClose;
                } else {
                    return Err("Parent is not object".to_string());
                }
            }
            _ => {
                return Err("Invalid parent status".to_string());
            }
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
pub fn parse_stream(json_string: &str) -> Result<Value, String> {
    let mut stack: Vec<(Value, ObjectStatus)> = vec![(Value::Null, ObjectStatus::Ready)];
    for current_char in json_string.chars() {
        let (ref val, ref st) = stack.last().unwrap();
        println!(
            "variables: {:?} {:?} {:?}",
            val,
            st.clone(),
            current_char.to_string()
        );
        add_char_into_object(&mut stack, current_char)?;
    }
    Ok(stack.pop().unwrap().0)
}

#[cfg(not(debug_assertions))]
pub fn parse_stream(json_string: &str) -> Result<Value, String> {
    let mut stack: Vec<(Value, ObjectStatus)> = vec![(Value::Null, ObjectStatus::Ready)];
    for current_char in json_string.chars() {
        add_char_into_object(&mut stack, current_char)?;
    }
    Ok(stack.pop().unwrap().0)
}

pub fn parse_stream_with_limits(
    json_string: &str,
    max_depth: Option<usize>,
    max_length: Option<usize>,
) -> Result<Value, String> {
    let mut parser = JsonStreamParser::with_limits(max_depth, max_length);
    for c in json_string.chars() {
        parser.add_char(c)?;
    }
    Ok(parser.stack.pop().unwrap().0)
}

pub struct JsonStreamParser {
    stack: Vec<(Value, ObjectStatus)>,
    processed_chars: usize,
    max_depth: Option<usize>,
    max_length: Option<usize>,
}

impl Default for JsonStreamParser {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonStreamParser {
    pub fn new() -> JsonStreamParser {
        Self::with_limits(None, None)
    }

    pub fn with_limits(max_depth: Option<usize>, max_length: Option<usize>) -> JsonStreamParser {
        JsonStreamParser {
            stack: vec![(Value::Null, ObjectStatus::Ready)],
            processed_chars: 0,
            max_depth,
            max_length,
        }
    }

    pub fn add_char(&mut self, current_char: char) -> Result<(), String> {
        if let Some(limit) = self.max_length {
            self.processed_chars += 1;
            if self.processed_chars > limit {
                return Err("input length limit exceeded".to_string());
            }
        }

        add_char_into_object(&mut self.stack, current_char)?;

        if let Some(max_depth) = self.max_depth {
            if self.stack.len() > max_depth {
                return Err("max depth exceeded".to_string());
            }
        }

        Ok(())
    }

    pub fn get_result(&self) -> &Value {
        &self.stack[0].0
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
        #[cfg(test)]
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
                    parser.add_char(c).unwrap();
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
    string_with_escaped_quote: r#""a\"b""#, Value::String("a\"b".to_string())
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
    exponent_positive: r#"1e2"#, Value::Number(serde_json::Number::from_f64(100.0).unwrap())
    exponent_negative: r#"1e-2"#, Value::Number(serde_json::Number::from_f64(0.01).unwrap())
    exponent_positive_decimal: r#"1.5e2"#, Value::Number(serde_json::Number::from_f64(150.0).unwrap())
    exponent_negative_decimal: r#"1.5e-2"#, Value::Number(serde_json::Number::from_f64(0.015).unwrap())
    tab_whitespace_number: "\t123\t", Value::Number(123.into())
    carriage_return_whitespace_number: "\r123\r", Value::Number(123.into())
    nested_array_value: "[[1]]", json!([[1]])
    deep_nested_array_value: "[[[1]]]", json!([[[1]]])
    empty_object: r#"{}"#, json!({})
    nested_object_value: r#"{"a":{"b":1}}"#, json!({"a": {"b": 1 }})
    deep_nested_object_value: r#"{"a":{"b":{"c":1}}}"#, json!({"a": {"b": {"c": 1 }}})
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

    #[test]
    fn array_with_nested_object() {
        let raw_json = "[{\"a\":1}]";
        let expected = json!([{"a":1}]);
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }

    #[test]
    fn array_with_nested_array() {
        let raw_json = "[[1,2],[3]]";
        let expected = json!([[1, 2], [3]]);
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }

    #[test]
    fn object_with_nested_object() {
        let raw_json = "{\"obj\":{\"b\":1}}";
        let expected = json!({"obj": {"b":1}});
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }

    #[test]
    fn object_with_nested_array() {
        let raw_json = "{\"arr\": [1, 2, 3]}";
        let expected = json!({"arr": [1, 2, 3]});
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }

    #[test]
    fn object_with_deep_nesting() {
        let raw_json = "{\"data\": [{\"vals\": [1]}]}";
        let expected = json!({"data": [{"vals": [1]}]});
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }

    #[test]
    fn object_with_extra_deep_nesting() {
        let raw_json = "{\"data\": [{\"vals\": [{\"deep\": [1]}]}]}";
        let expected = json!({"data": [{"vals": [{"deep": [1]}]}]});
        let result = parse_stream(raw_json);
        assert_eq!(result.unwrap(), expected);
        let mut parser = JsonStreamParser::new();
        for c in raw_json.chars() {
            parser.add_char(c).unwrap();
        }
        assert_eq!(parser.get_result(), &expected);
    }
}
