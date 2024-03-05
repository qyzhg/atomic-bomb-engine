use std::str::FromStr;
use std::sync::{Arc};
use histogram::Histogram;
use std::time::{self,Duration, Instant};
use tokio::time::interval;
use anyhow::{Context, Error};
use reqwest::{Client, Method, StatusCode};
use tokio::sync::{Mutex, Semaphore};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, HeaderName};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use jsonpath_lib::select;

use crate::core::sleep_guard::SleepGuard;
use crate::core::status_share::{RESULT_QUEUE, SHOULD_STOP};
use crate::models::assert_error_stats::AssertErrorStats;
use crate::models::http_error_stats::HttpErrorStats;
use crate::models::result::TestResult;
use crate::models::assert_option::AssertOption;
use crate::models::api_endpoint::ApiEndpoint;
pub async fn batch(
    test_duration_secs: u64,
    concurrent_requests: usize,
    verbose: bool,
    should_prevent: bool,
    api_endpoints: Vec<ApiEndpoint>
) -> anyhow::Result<TestResult> {
    // 阻止电脑休眠
    let _guard = SleepGuard::new(should_prevent);
    // 总响应时间统计
    let histogram = Arc::new(Mutex::new(Histogram::new(14, 20).unwrap()));
    // 成功数据统计
    let successful_requests = Arc::new(Mutex::new(0));
    // 请求总数统计
    let total_requests = Arc::new(Mutex::new(0));
    // 统计最大响应时间
    let max_response_time = Arc::new(Mutex::new(0u64));
    // 统计最小响应时间
    let min_response_time = Arc::new(Mutex::new(u64::MAX));
    // 统计错误数量
    let err_count = Arc::new(Mutex::new(0));
    // 线程池
    let mut handles = Vec::new();
    // 统计响应大小
    let total_response_size = Arc::new(Mutex::new(0u64));
    // 统计http错误
    let http_errors = Arc::new(Mutex::new(HttpErrorStats::new()));
    // 统计断言错误
    let assert_errors = Arc::new(Mutex::new(AssertErrorStats::new()));
    // 总权重
    let total_weight: u32 = api_endpoints.iter().map(|e| e.weight).sum();
    // 持续时间
    let test_duration = Duration::from_secs(test_duration_secs);
    // 并发量控制
    let semaphore = Arc::new(Mutex::new(Semaphore::new(concurrent_requests)));
    // 结束时间
    let test_end = Instant::now() + Duration::from_secs(test_duration_secs);
    // 用arc包装每一个endpoint
    let api_endpoints_arc: Vec<Arc<Mutex<ApiEndpoint>>> = api_endpoints
        .into_iter()
        .map(|endpoint| Arc::new(Mutex::new(endpoint)))
        .collect();
    // 针对每一个接口开始配置
    for endpoint_arc in api_endpoints_arc.iter() {
        let semaphore_clone = Arc::clone(&semaphore);
        let total_requests_clone = Arc::clone(&total_requests);
        let endpoint_clone = Arc::clone(endpoint_arc);
        let weight_ratio = endpoint_arc.lock().await.weight as f64 / total_weight as f64;
        let concurrency_for_endpoint = (concurrent_requests as f64 * weight_ratio).round() as usize;
        let handle = tokio::spawn(async move {
            for _ in 0..concurrency_for_endpoint {
                let semaphore_clone = Arc::clone(&semaphore_clone);
                let total_requests_clone = Arc::clone(&total_requests_clone);
                let endpoint_clone = Arc::clone(&endpoint_clone);
                let mut interval = tokio::time::interval(test_duration / concurrency_for_endpoint as u32);
                tokio::spawn(async move {
                    while Instant::now() < test_end {
                        interval.tick().await; // 这里是请求间隔，如果太慢就注释掉
                        let semaphore_guard = semaphore_clone.lock().await; // 第一步：获取锁
                        let _permit = semaphore_guard.acquire().await.expect("Failed to acquire semaphore permit"); // 第二步：获取许可
                        /*
                            构建请求体
                        */
                        
                        // 总请求数+1
                        *total_requests_clone.lock().await += 1;
                        // 记录当前接口开始时间
                        let start = Instant::now();
                        // 构建请求方法
                        let method = Method::from_str(&*endpoint_clone.lock().await.method.to_uppercase()).expect("Invalid method");
                        // 构建http客户端
                        let client_builder = reqwest::Client::builder();
                        // 构建client
                        let client = if endpoint_clone.lock().await.timeout_secs > 0 {
                            client_builder.timeout(Duration::from_secs(endpoint_clone.lock().await.timeout_secs)).build().context("构建带超时的http客户端失败")
                        } else {
                            client_builder.build().context("构建http客户端失败")
                        };
                        // 构建request
                        let mut request = match client{
                            Ok(cli) => {
                                cli.request(method, endpoint_clone.lock().await.url.clone())
                            }
                            Err(e) => {
                                eprintln!("构建请求体失败:{:?}", e);
                                panic!("{:?}", e);
                            }
                        };
                    }
                }).await.expect("xxx");
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }

    Ok(TestResult {
        total_duration: 0.0,
        success_rate: 0.0,
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
        http_errors: Default::default(),
        timestamp: 0,
        assert_errors: Default::default(),
    })
}