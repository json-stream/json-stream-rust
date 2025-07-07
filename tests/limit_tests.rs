use json_stream_parser::parse_stream_with_limits;

#[test]
fn depth_limit_exceeded() {
    let json = "[[[1]]]";
    let result = parse_stream_with_limits(json, Some(2), None);
    assert!(result.is_err());
}

#[test]
fn length_limit_exceeded() {
    let json = "[1,2,3]";
    let result = parse_stream_with_limits(json, None, Some(5));
    assert!(result.is_err());
}
