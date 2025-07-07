use json_stream_parser::{parse_stream, JsonStreamParser};
use proptest::prelude::*;
use proptest::string::string_regex;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

fn json_number() -> impl Strategy<Value = serde_json::Number> {
    any::<i64>().prop_map(|n| n.into())
}

fn json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        json_number().prop_map(Value::Number),
        string_regex("[a-zA-Z0-9]{0,20}")
            .unwrap()
            .prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 8, 3, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..3).prop_map(Value::Array),
            prop::collection::btree_map(string_regex("[a-zA-Z0-9]{0,10}").unwrap(), inner, 0..3)
                .prop_map(|m: BTreeMap<String, Value>| {
                    let map: Map<String, Value> = m.into_iter().collect();
                    Value::Object(map)
                })
        ]
    })
}

proptest! {
    #[test]
    fn roundtrip_json(value in json_value()) {
        let json = serde_json::to_string(&value).unwrap();
        let parsed = parse_stream(&json).unwrap();
        let expected = parsed.clone();
        prop_assert_eq!(parsed, value);

        let mut parser = JsonStreamParser::new();
        for c in json.chars() {
            parser.add_char(c).unwrap();
        }
        prop_assert_eq!(parser.get_result(), &expected);
    }
}
