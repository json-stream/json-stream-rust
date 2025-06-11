use json_stream_parser::parse_stream;
use serde_json::Value;
use std::time::Instant;

const LARGE_JSON: &str = include_str!("large_array.json");

fn memory_usage_kb() -> usize {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = value.parse::<usize>() {
                        return kb;
                    }
                }
            }
        }
    }
    0
}

#[test]
fn performance_compare() {
    let json = LARGE_JSON;

    let start_mem = memory_usage_kb();
    let start_time = Instant::now();
    let parsed_stream = parse_stream(&json).unwrap();
    let duration_stream = start_time.elapsed();
    let end_mem = memory_usage_kb();
    let mem_used_stream = end_mem.saturating_sub(start_mem);

    let start_mem = memory_usage_kb();
    let start_time = Instant::now();
    let parsed_serde: Value = serde_json::from_str(&json).unwrap();
    let duration_serde = start_time.elapsed();
    let end_mem = memory_usage_kb();
    let mem_used_serde = end_mem.saturating_sub(start_mem);

    assert_eq!(parsed_stream, parsed_serde);

    println!(
        "json_stream_parser: time: {:?}, memory: {} KB",
        duration_stream, mem_used_stream
    );
    println!(
        "serde_json: time: {:?}, memory: {} KB",
        duration_serde, mem_used_serde
    );
}
