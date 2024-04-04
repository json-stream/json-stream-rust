#[cfg(test)]
mod tests {
    mod object_one_property {
        mod valid_json_tests {
            use crate::parse_stream;
            use serde_json::json;

            #[test]
            fn test_single_key_value_pair() {
                let raw_json = r#"{"key": "value"}"#;
                let result = parse_stream(raw_json);
                assert_eq!(result.unwrap().unwrap(), json!({"key": "value"}));
            }

            #[test]
            fn test_single_key_value_pair_with_number() {
                let raw_json = r#"{"age": 1234567890}"#;
                let result = parse_stream(raw_json);
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
