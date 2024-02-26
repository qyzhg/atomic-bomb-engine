use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use histogram::Histogram;
use tokio::time::interval;
use crate::models::http_error_stats::HttpErrorStats;

pub async fn share_test_results_periodically(
    test_duration_secs: u64,
    histogram: Arc<Mutex<Histogram>>,
    successful_requests: Arc<Mutex<i32>>,
    total_requests: Arc<Mutex<i32>>,
    max_response_time: Arc<Mutex<u64>>,
    min_response_time: Arc<Mutex<u64>>,
    err_count: Arc<Mutex<i32>>,
    total_response_size: Arc<Mutex<u64>>,
    http_errors: Arc<Mutex<HttpErrorStats>>,
) {
    let mut interval = interval(Duration::from_secs(1));
    let test_start = Instant::now();
    let test_end = test_start + Duration::from_secs(test_duration_secs);

    while Instant::now() < test_end {
        interval.tick().await;
        // todo: 从这里共享
    }
}
