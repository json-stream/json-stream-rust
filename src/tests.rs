
#[cfg(test)]
mod tests {
    mod valid_json_tests {
        use super::*;

        #[test]
        fn test_single_key_value_pair() {
            let raw_json = r#"{"key": "value"}"#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": "value"})));
        }

        // More tests for valid_json..
    }

    mod partial_json_tests {
        use super::*;

        #[test]
        fn test_without_closing_brace() {
            let raw_json = r#"{"key": "value""#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": "value"})));
        }

        // More tests for partial_json..
    }
}

