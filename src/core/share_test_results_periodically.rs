use std::sync::{Arc};
use std::time::{Duration, Instant};
use histogram::Histogram;
use tokio::time::interval;
use tokio::sync::Mutex;

use crate::models::http_error_stats::HttpErrorStats;

pub async fn share_test_results_periodically(
    test_duration_secs: u64,
    _histogram: Arc<Mutex<Histogram>>, // 使用tokio::sync::Mutex
    _successful_requests: Arc<Mutex<i32>>,
    _total_requests: Arc<Mutex<i32>>,
    _max_response_time: Arc<Mutex<u64>>,
    _min_response_time: Arc<Mutex<u64>>,
    _err_count: Arc<Mutex<i32>>,
    _total_response_size: Arc<Mutex<u64>>,
    _http_errors: Arc<Mutex<HttpErrorStats>>,
) {
    let mut interval = interval(Duration::from_secs(1));
    let test_start = Instant::now();
    let test_end = test_start + Duration::from_secs(test_duration_secs);

    while Instant::now() < test_end {
        interval.tick().await;
        //todo 在这里编写逻辑以共享测试结果
    }
}