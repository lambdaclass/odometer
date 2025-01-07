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
    let client = JwtClient::new(secret);

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
                time_taken_milliseconds: timed_response.time_taken_milliseconds,
                gas_used: sequence_element.request.gas_used(),
                response: timed_response.response,
            };

            summaries.push(summary);
        }
    }

    for sum in summaries {
        dbg!(sum);
    }

    dc.down().unwrap();
}
