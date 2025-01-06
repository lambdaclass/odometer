use crate::engine_api::{EngineApiRequest, EngineApiResponse, RequestType};

pub struct BenchInput {
    // name of the benchmark
    name: String,
    // TODO: Hmm we can probably just get this from the engine API
    // TODO request itself
    gas_used: String,
    // Raw engine API request that the CL
    // would send to the EL
    engine_api_requests: EngineApiRequest,
}

#[derive(Debug)]
pub struct BenchSummary {
    pub name: String,
    pub time_taken_millisecond: u128,
    pub gas_used: Option<String>,
    // TODO: This is somewhat redundant, since the ResultType
    // TODO: in response also tells you whether it was
    // TODO: a newPayload request or a forkChoiceUpdate
    pub request_type: RequestType,
    pub response: EngineApiResponse,
}
