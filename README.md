# JSON Stream Parser in Rust

This project is a library that provides an incremental JSON parser, built with Rust. It's designed to parse a stream of JSON data, making it suited for situations where you might not have the entire JSON object available when parsing begins — for example, in the case of streaming a structured response from a Large Language Model.

🚨 This project is a work in progress, and is not yet ready for use in production.

## Installation

This project is built with [Rust](https://www.rust-lang.org/), and you'll need `cargo` to get started.

To include JSON Stream Parser in your project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
json-stream-parser = "0.1.0"
```

Then, run `cargo build` to build your project.

## Usage

The simplest way to use this library is to use the `parse_stream` function, which takes a string slice and returns a `Result` containing a `serde_json::Value` if successful.
Here's an example:

```rust
fn main() {
    let incomplete_json = r#"{"key": "value""#;
    let parsed_json = json_stream_parser::parse_stream(incomplete_json);
    if let Ok(json) = parsed_json {
        println!("{:?}", json);
    }
}
```

As you can see this object is incomplete, but the parser will still be able to parse it:

```rust
Some(Object {"key": String("value")})
```

## Testing

This project uses `cargo test` to run the tests.

## Contributing

Communicate intentions through the Issues for any major changes. Feel free contribute other changes directly via a pull request.

## License

This project is licensed under the MIT License.
