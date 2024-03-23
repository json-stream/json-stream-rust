
#[cfg(test)]
mod tests {
    mod valid_json_tests {
        use crate::parse_stream;
        use serde_json::json;

        #[test]
        fn test_single_key_value_pair() {
            let raw_json = r#"{"key": "value"}"#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": "value"})));
        }

        #[test]
        fn test_single_key_value_pair_with_number() {
            let raw_json = r#"{"age": 30}"#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"age": 30})));
        }
    }

    mod partial_json_tests {
        use crate::parse_stream;
        use serde_json::json;

        #[test]
        fn test_without_closing_brace_for_value() {
            let raw_json = r#"{"key": "value""#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": "value"})));
        }

        #[test]
        fn test_without_closing_quote_for_value() {
            let raw_json = r#"{"key": "value"#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": "value"})));
        }

        #[test]
        fn test_with_opening_quote_without_text_for_value() {
            let raw_json = r#"{"key": ""#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": ""})));
        }

        #[test]
        fn test_without_value() {
            let raw_json = r#"{"key": "#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": null})));
        }

        #[test]
        fn test_without_colon() {
            let raw_json = r#"{"key""#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({"key": null})));
        }

        #[test]
        fn test_without_closing_quote_for_key() {
            let raw_json = r#"{"ke"#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({})));
        }

        #[test]
        fn test_with_just_opening_quote_for_key() {
            let raw_json = r#"{""#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({})));
        }

        #[test]
        fn test_with_just_opening_brace() {
            let raw_json = r#"{"#;
            let result = parse_stream(raw_json);
            assert_eq!(result, Some(json!({})));
        }
    }
}

