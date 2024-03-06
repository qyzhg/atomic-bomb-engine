use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
#[derive(Clone)]
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
    pub http_errors: HashMap<(u16, String, String), u32>,
    pub timestamp: u128,
    pub assert_errors: HashMap<(String, String), u32>
}

#[derive(Debug)]
#[derive(Clone)]
pub struct BatchResult {
    pub total_duration: f64,
    pub success_rate: f64,
    pub median_response_time: u64,
    pub response_time_95: u64,
    pub response_time_99: u64,
    pub total_requests: u64,
    pub rps: f64,
    pub max_response_time: u64,
    pub min_response_time: u64,
    pub err_count: i32,
    pub total_data_kb: f64,
    pub throughput_per_second_kb: f64,
    pub http_errors: HashMap<(u16, String, String), u32>,
    pub timestamp: u128,
    pub assert_errors: HashMap<(String, String), u32>,
    pub api_results: Vec<ApiResult>
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ApiResult{
    pub name: String,
    pub url: String,
    pub success_rate: f64,
    pub median_response_time: u64,
    pub response_time_95: u64,
    pub response_time_99: u64,
    pub total_requests: u64,
    pub rps: f64,
    pub max_response_time: u64,
    pub min_response_time: u64,
    pub err_count: i32,
    pub total_data_kb: f64,
    pub throughput_per_second_kb: f64,
}
