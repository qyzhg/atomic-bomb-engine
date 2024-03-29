use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total_duration: f64,
    pub success_rate: f64,
    pub error_rate: f64,
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
    pub total_concurrent_number: i32,
    pub api_results: Vec<ApiResult>
}

#[derive(Debug)]
#[derive(Clone, Serialize, Deserialize)]
pub struct ApiResult{
    pub name: String,
    pub url: String,
    pub method: String,
    pub success_rate: f64,
    pub error_rate: f64,
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
    pub concurrent_number: i32,
}

impl ApiResult {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            method: String::new(),
            success_rate: 0.0,
            error_rate: 0.0,
            median_response_time: 0,
            response_time_95: 0,
            response_time_99: 0,
            total_requests: 0,
            rps: 0.0,
            max_response_time: 0,
            min_response_time: 0,
            err_count: 0,
            total_data_kb: 0.0,
            throughput_per_second_kb: 0.0,
            concurrent_number: 0,
        }
    }
}
