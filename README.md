![crates.io](https://img.shields.io/crates/v/json-stream-parser.svg)

# JSON Stream Parser

Parse JSON as it streams in, character by character. Perfect for real-time applications like LLM responses, WebSocket streams, or large file processing.

## Why Use This?

- **Real-time parsing**: Get results before the stream completes
- **Memory efficient**: No need to buffer entire JSON strings
- **Handles incomplete data**: Gracefully parses partial JSON
- **Zero-copy when possible**: Built for performance

## Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
json-stream-parser = "0.1.4"
```

### Simple Usage

```rust
use json_stream_parser::parse_stream;

let incomplete = r#"{"name": "John", "age": 3"#;
let result = parse_stream(incomplete).unwrap();
println!("{:?}", result); // Object {"name": String("John"), "age": Number(3)}
```

### Streaming Usage

```rust
use json_stream_parser::JsonStreamParser;

let mut parser = JsonStreamParser::new();
let stream = r#"{"items": [1, 2, 3]}"#;

for char in stream.chars() {
    parser.add_char(char);
    // Get partial results as parsing progresses
    let current_result = parser.get_result();
}
```

## Use Cases

- **LLM streaming responses**: Parse JSON as AI models generate it
- **WebSocket data**: Handle real-time JSON streams
- **Large file processing**: Parse without loading entire files into memory
- **Network protocols**: Handle JSON over TCP/UDP streams
- **Log processing**: Parse JSON logs in real-time

## Performance

Designed for streaming scenarios where traditional parsers fall short:
- Parses incomplete JSON strings
- Updates results incrementally
- Minimal memory overhead
- No blocking on incomplete data

## Testing

```bash
cargo test                           # Run all tests
cargo test --test property_tests     # Run property-based tests
```

## License

MIT
