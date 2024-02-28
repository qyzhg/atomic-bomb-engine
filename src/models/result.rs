use std::collections::HashMap;
#[allow(dead_code)]
pub struct TestResult {
    pub total_duration: f64,
    pub success_rate: f64,
    pub median_response_time: u64,
    pub response_time_95: u64,
    pub response_time_99: u64,
    pub total_requests: i32,
    pub rps: f64,
    pub max_response_time: u64,
    pub min_response_time: u64,
    pub err_count: i32,
    pub total_data_kb: f64,
    pub throughput_per_second_kb: f64,
    pub http_errors: HashMap<(u16, String), u32>
}
