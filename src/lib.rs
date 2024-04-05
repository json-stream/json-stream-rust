use serde_json::{json, Value};

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
    object: &mut Option<Value>,
    current_status: &mut ObjectStatus,
    current_char: char,
) -> Result<(), String> {
    match (object.clone(), current_status.clone(), current_char) {
        (None, ObjectStatus::Ready, '{') => {
            *object = Some(json!({}));
            *current_status = ObjectStatus::StartProperty;
        }
        (Some(Value::Object(_obj)), ObjectStatus::StartProperty, '"') => {
            *current_status = ObjectStatus::KeyQuoteOpen { key_so_far: vec![] };
        }
        (Some(Value::Object(mut obj)), ObjectStatus::KeyQuoteOpen { key_so_far }, '"') => {
            *current_status = ObjectStatus::KeyQuoteClose {
                key: key_so_far.clone(),
            };
            // add the key to the object with null value
            obj.insert(key_so_far.iter().collect::<String>(), Value::Null);
            *object = Some(Value::Object(obj));
        }
        (Some(Value::Object(_obj)), ObjectStatus::KeyQuoteOpen { mut key_so_far }, char) => {
            key_so_far.push(char);
            *current_status = ObjectStatus::KeyQuoteOpen { key_so_far };
        }
        (Some(Value::Object(_obj)), ObjectStatus::KeyQuoteClose { key }, ':') => {
            *current_status = ObjectStatus::Colon { key };
        }
        (Some(Value::Object(_obj)), ObjectStatus::Colon { .. }, ' ' | '\n') => {}
        (Some(Value::Object(mut obj)), ObjectStatus::Colon { key }, '"') => {
            *current_status = ObjectStatus::ValueQuoteOpen { key: key.clone() };
            // create an empty string for the value
            obj.insert(key.iter().collect::<String>().clone(), json!(""));
            *object = Some(Value::Object(obj));
        }
        // ------ Add String Value ------
        (Some(Value::Object(_obj)), ObjectStatus::ValueQuoteOpen { key: _key }, '"') => {
            *current_status = ObjectStatus::ValueQuoteClose;
        }
        (Some(Value::Object(mut obj)), ObjectStatus::ValueQuoteOpen { key }, char) => {
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
            *object = Some(Value::Object(obj));
        }

        // ------ Add Scalar Value ------
        (Some(Value::Object(_obj)), ObjectStatus::Colon { key }, char) => {
            *current_status = ObjectStatus::ValueScalar {
                key,
                value_so_far: vec![char],
            };
        }
        (Some(Value::Object(mut obj)), ObjectStatus::ValueScalar { key, value_so_far }, ',') => {
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
            *object = Some(Value::Object(obj));
            *current_status = ObjectStatus::StartProperty;
        }
        (Some(Value::Object(mut obj)), ObjectStatus::ValueScalar { key, value_so_far }, '}') => {
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
            *object = Some(Value::Object(obj));
            *current_status = ObjectStatus::Closed;
        }
        (
            Some(Value::Object(_obj)),
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
        (Some(Value::Object(_obj)), ObjectStatus::ValueQuoteClose, ',') => {
            *current_status = ObjectStatus::StartProperty;
        }
        (Some(Value::Object(_obj)), ObjectStatus::ValueQuoteClose, '}') => {
            *current_status = ObjectStatus::Closed;
        }

        // ------ white spaces ------
        (_, _, ' ' | '\n') => {
            // ignore whitespace
        }
        _ => {
            return Err(format!("Invalid character {}", current_char));
        }
    }

    Ok(())
}

pub fn parse_stream(json_string: &str) -> Result<Option<Value>, String> {
    let mut out: Option<Value> = None;
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

#[cfg(test)]
mod tests {
    mod object_one_property {
        mod valid_json_tests {
            use crate::parse_stream;
            use serde_json::json;

            #[test]
            fn test_single_key_value_pair() {
                let raw_json = r#"{"key": "value"}"#;
                let result = parse_stream(raw_json); // Call the parse_stream function correctly
                assert_eq!(result.unwrap().unwrap(), json!({"key": "value"}));
            }

            #[test]
            fn test_single_key_value_pair_with_number() {
                let raw_json = r#"{"age": 1234567890}"#;
                let result = parse_stream(raw_json); // Call the parse_stream function correctly
                assert_eq!(result.unwrap().unwrap(), json!({"age": 1234567890}));
            }

            #[test]
            fn test_single_key_value_pair_with_negative_number() {
                let raw_json = r#"{"age": -1234567890}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": -1234567890}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_number_starting_with_zero() {
                let raw_json = r#"{"age": 01234567890}"#;
                let result = parse_stream(raw_json);
                // This is invalid JSON, so we should return an error.
                assert_eq!(result.is_err(), true);
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_float_starting_zero() {
                let raw_json = r#"{"age": 0.456}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": 0.456}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_float_starting_zero_and_zero_in_middle() {
                let raw_json = r#"{"age": 0.004056}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": 0.004056}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_negative_float_starting_zero() {
                let raw_json = r#"{"age": -0.456}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": -0.456}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_float_starting_zero_and_no_decimal() {
                let raw_json = r#"{"age": 0.}"#;
                let result = parse_stream(raw_json);
                // This is invalid JSON, so we should return an error.
                assert_eq!(result.is_err(), true);
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_float_starting_not_zero() {
                let raw_json = r#"{"age": 1.456}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": 1.456}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_negative_float_starting_not_zero() {
                let raw_json = r#"{"age": -1.456}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": -1.456}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_zero() {
                let raw_json = r#"{"age": 0}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": 0}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_false() {
                let raw_json = r#"{"age": false}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": false}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_true() {
                let raw_json = r#"{"age": true}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": true}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_null() {
                let raw_json = r#"{"age": null}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": null}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_empty_string() {
                let raw_json = r#"{"age": ""}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": ""}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_1() {
                let raw_json = r#"{"age": "  "}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": "  "}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_2() {
                let raw_json = r#"{"age": "  a  "}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": "  a  "}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_3() {
                let raw_json = r#"{"age": "a  "}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": "a  "}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_4() {
                let raw_json = r#"{"age": "  a"}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": "  a"}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_5() {
                let raw_json = r#"{"a ge": 23}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"a ge": 23}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_6() {
                let raw_json = r#"{"age ": 23}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age ": 23}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_7() {
                let raw_json = r#"{" age": 23}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({" age": 23}));
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_spaces_8() {
                let raw_json = r#"{ "age":  23  , " height ": 180 }"#;
                let result = parse_stream(raw_json);
                assert_eq!(
                    result.unwrap().unwrap(),
                    json!({"age": 23, " height ": 180})
                );
            }

            #[test]
            fn test_invalid_single_key_value_pair_with_string_with_line_breaks_1() {
                let raw_json = r#"{
                    "age": 23,
                    "name": "John"
                }"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"age": 23, "name": "John"}));
            }
        }

        mod partial_json_tests {
            use crate::parse_stream;
            use serde_json::json;

            #[test]
            fn test_without_closing_brace_for_value() {
                let raw_json = r#"{"key": "value""#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({"key": "value"})));
            }

            #[test]
            fn test_without_closing_quote_for_value() {
                let raw_json = r#"{"key": "value"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({"key": "value"})));
            }

            #[test]
            fn test_with_opening_quote_without_text_for_value() {
                let raw_json = r#"{"key": ""#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({"key": ""})));
            }

            #[test]
            fn test_without_value() {
                let raw_json = r#"{"key": "#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({"key": null})));
            }

            #[test]
            fn test_without_colon() {
                let raw_json = r#"{"key""#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({"key": null})));
            }

            #[test]
            fn test_without_closing_quote_for_key() {
                let raw_json = r#"{"ke"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({})));
            }

            #[test]
            fn test_with_just_opening_quote_for_key() {
                let raw_json = r#"{""#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({})));
            }

            #[test]
            fn test_with_just_opening_brace() {
                let raw_json = r#"{"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), (json!({})));
            }
        }
    }

    mod object_two_properties {
        mod valid_json_tests {
            use crate::parse_stream;
            use serde_json::json;

            #[test]
            fn test_two_key_value_pairs() {
                let raw_json = r#"{"key1": "value1", "key2": "value2"}"#;
                let result = parse_stream(raw_json);
                assert_eq!(
                    result.unwrap().unwrap(),
                    json!({"key1": "value1", "key2": "value2"})
                );
            }

            #[test]
            fn test_two_key_value_pairs_with_number() {
                let raw_json = r#"{"age": 1234567890, "height": 180}"#;
                let result = parse_stream(raw_json);
                assert_eq!(
                    result.unwrap().unwrap(),
                    json!({"age": 1234567890, "height": 180})
                );
            }
        }

        mod partial_json_tests {
            use crate::parse_stream;
            use serde_json::json;

            #[test]
            fn test_without_closing_brace_for_value() {
                let raw_json = r#"{"key1": "value1", "key2": "value2""#;
                let result = parse_stream(raw_json);
                assert_eq!(
                    result.unwrap().unwrap(),
                    (json!({"key1": "value1", "key2": "value2"}))
                );
            }

            #[test]
            fn test_without_closing_quote_for_value() {
                let raw_json = r#"{"key1": "value1", "key2": "value2"#;
                let result = parse_stream(raw_json);
                assert_eq!(
                    result.unwrap().unwrap(),
                    (json!({"key1": "value1", "key2": "value2"}))
                );
            }

            #[test]
            fn test_with_opening_quote_without_text_for_value() {
                let raw_json = r#"{"key1": "value1", "key2": ""#;
                let result = parse_stream(raw_json);
                assert_eq!(
                    result.unwrap().unwrap(),
                    (json!({"key1": "value1", "key2": ""}))
                );
            }

            #[test]
            fn test_without_value() {
                let raw_json = r#"{"key1": "value1", "key2": "#;
                let result = parse_stream(raw_json);
                assert_eq!(
                    result.unwrap().unwrap(),
                    (json!({"key1": "value1", "key2": null}))
                );
            }
        }
    }
}
