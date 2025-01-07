mod bench_summary;
mod docker;
mod engine_api;
mod kute;

use bench_summary::{BenchEngineAPIRequestSummary, BenchInput};
use kute::JwtClient;
use std::fs;

#[tokio::main]
async fn main() {
    // Launch client
    let dc = docker::DockerCompose::new("clients/geth", "test");
    dc.up().unwrap();

    // Setup Engine API credentials
    let secret = hex::decode(fs::read_to_string("jwt.hex").unwrap().trim()).unwrap();
    let client = JwtClient::new(secret, "http://localhost:8551".to_string());

    let mut summaries = Vec::new();

    // Parse Engine API requests that we want to benchmark
    let json = fs::read_to_string("tests/gaslimit_30m.json").unwrap();
    let parsed: BenchInput = serde_json::from_str(&json).unwrap();

    for sequence_element in parsed.sequence {
        let timed_response = match client.send_request(&sequence_element.request).await {
            Ok(timed_response) => timed_response,
            Err(err) => {
                println!("fail {}", err);
                continue;
            }
        };

        // Check if we should keep this measurement
        if sequence_element.expect_measurement {
            let summary = BenchEngineAPIRequestSummary {
                description: sequence_element.description,
                time_taken_microseconds: timed_response.time_taken_microseconds,
                gas_used: sequence_element
                    .request
                    .gas_used()
                    .expect("expected a gas used parameter for elements we are benchmarking"),
                response: timed_response.response,
            };

            summaries.push(summary);
        }
    }

    for sum in summaries {
        let gas_used_u128 = parse_gas_used(&sum.gas_used);
        let gas_per_second = compute_gas_per_second(gas_used_u128, sum.time_taken_microseconds);
        println!(
            "name: {}\ndesc: {}\ngas_rate: {}",
            parsed.name,
            parsed.description,
            format_gas_rate(gas_per_second)
        );
    }

    dc.down().unwrap();
}

fn parse_gas_used(gas_used_hex_str: &str) -> u128 {
    let hex_str_no_prefix = gas_used_hex_str.trim_start_matches("0x");
    u128::from_str_radix(hex_str_no_prefix, 16).expect("Failed to parse hexadecimal string")
}

fn compute_gas_per_second(gas_used: u128, time_ms: u128) -> u128 {
    if time_ms == 0 {
        return 0;
    }
    gas_used.saturating_mul(1_000_000) / time_ms
}
fn gas_per_second_to_mgas_per_second(gps: u128) -> u128 {
    gps / 1_000_000
}

fn format_gas_rate(gas_per_second: u128) -> String {
    if gas_per_second >= 1_000_000_000 {
        // Convert to Ggas/s
        let rate = (gas_per_second as f64) / 1_000_000_000_f64;
        format!("{:.2} Ggas/s", rate)
    } else if gas_per_second >= 1_000_000 {
        // Convert to Mgas/s
        let rate = (gas_per_second as f64) / 1_000_000_f64;
        format!("{:.2} Mgas/s", rate)
    } else {
        // Just "gas/s"
        format!("{} gas/s", gas_per_second)
    }
}
