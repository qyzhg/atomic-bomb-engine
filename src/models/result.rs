use std::time::Duration;
use std::collections::HashMap;
#[allow(dead_code)]
pub struct TestResult {
    pub(crate) total_duration: Duration,
    pub(crate) success_rate: f64,
    pub(crate) median_response_time: u64,
    pub(crate) response_time_95: u64,
    pub(crate) response_time_99: u64,
    pub(crate) total_requests: i32,
    pub(crate) rps: f64,
    pub(crate) max_response_time: u64,
    pub(crate) min_response_time: u64,
    pub(crate) err_count: i32,
    pub(crate) total_data_kb: f64,
    pub(crate) throughput_per_second_kb: f64,
    pub(crate) http_errors: HashMap<(u16, String), u32>
}
