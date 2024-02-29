use std::sync::{Arc};
use std::time::{Duration, Instant};
use histogram::Histogram;
use tokio::time::interval;
use tokio::sync::Mutex;
use crate::core::share_channel::{MESSAGES, SHOULD_STOP};
use crate::models::http_error_stats::HttpErrorStats;
use crate::models::result::TestResult;


pub async fn share_test_results_periodically(
    start: Instant,
    histogram: Arc<Mutex<Histogram>>, // 使用tokio::sync::Mutex
    successful_requests: Arc<Mutex<i32>>,
    total_requests: Arc<Mutex<i32>>,
    max_response_time: Arc<Mutex<u64>>,
    min_response_time: Arc<Mutex<u64>>,
    err_count: Arc<Mutex<i32>>,
    total_response_size: Arc<Mutex<u64>>,
    http_errors: Arc<Mutex<HttpErrorStats>>,
)  {
    let mut interval = interval(Duration::from_secs(1));
    let should_stop = *SHOULD_STOP.lock().unwrap();

    while !should_stop {
        interval.tick().await;
        let total_duration = (Instant::now() - start).as_secs_f64();
        let total_requests = *total_requests.lock().await as f64;
        let successful_requests = *successful_requests.lock().await as f64;
        let success_rate = successful_requests / total_requests * 100.0;
        let histogram = histogram.lock().await;
        let total_response_size_kb = *total_response_size.lock().await as f64 / 1024.0;
        let throughput_kb_s = total_response_size_kb / total_duration as f64;
        let http_errors = http_errors.lock().await.errors.clone();
        let resp_median_line = match  histogram.percentile(50.0){
            Ok(bucket) => *bucket.range().start(),
            Err(_) =>0
        };
        let resp_95_line = match  histogram.percentile(95.0){
            Ok(bucket) => *bucket.range().start(),
            Err(_) =>0
        };
        let resp_99_line = match  histogram.percentile(99.0){
            Ok(bucket) => *bucket.range().start(),
            Err(_) =>0
        };
        let test_result = TestResult {
            total_duration,
            success_rate,
            median_response_time: resp_median_line,
            response_time_95: resp_95_line,
            response_time_99: resp_99_line,
            total_requests: total_requests as i32,
            rps: successful_requests / total_duration,
            max_response_time: *max_response_time.lock().await,
            min_response_time: *min_response_time.lock().await,
            err_count:*err_count.lock().await,
            total_data_kb:total_response_size_kb,
            throughput_per_second_kb: throughput_kb_s,
            http_errors: http_errors.lock().unwrap().clone(),
        };
        let mut messages = MESSAGES.lock().unwrap();
        messages.push_back(test_result);
    }
    eprintln!("结束生产")
}
