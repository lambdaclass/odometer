mod bench_summary;
mod docker;
mod engine_api;
mod kute;

use kute::JwtClient;
use std::fs;

#[tokio::main]
async fn main() {
    let dc = docker::DockerCompose::new("clients/geth", "test");
    dc.up().unwrap();

    let secret = hex::decode(fs::read_to_string("jwt.hex").unwrap().trim()).unwrap();
    let client = JwtClient::new(secret);

    let mut summaries = Vec::new();

    for request in fs::read_to_string("tests/requests.txt").unwrap().lines() {
        let parsed: engine_api::EngineApiRequest = serde_json::from_str(request).unwrap();
        match client.send_request(&parsed).await {
            Ok(summary) => summaries.push(summary),
            Err(err) => println!("fail {}", err),
        };
    }

    for sum in summaries {
        dbg!(sum.time_taken_millisecond);
    }

    dc.down().unwrap();
}
