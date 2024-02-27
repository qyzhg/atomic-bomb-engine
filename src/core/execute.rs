use std::str::FromStr;
use std::sync::{Arc};
use histogram::Histogram;
use std::time::{Duration, Instant};
use indicatif::ProgressBar;
use tokio::time::interval;
use anyhow::{Context};
use reqwest::Method;
use tokio::sync::Mutex;
use serde_json::Value;
use crate::core::share_test_results_periodically::share_test_results_periodically;
use crate::models::http_error_stats::HttpErrorStats;
use crate::models::result::TestResult;


pub async fn run(url: &str, test_duration_secs: u64, concurrent_requests: i32, timeout_secs:u64, verbose: bool, method: &str, json_str: &str) -> anyhow::Result<TestResult> {
    let method = method.to_owned();
    let json_str = json_str.to_owned();
    let histogram = Arc::new(Mutex::new(Histogram::new(10, 16).unwrap()));
    let successful_requests = Arc::new(Mutex::new(0));
    let total_requests = Arc::new(Mutex::new(0));
    let test_start = Instant::now();
    let test_end = test_start + Duration::from_secs(test_duration_secs);
    let max_response_time = Arc::new(Mutex::new(0u64));
    let min_response_time = Arc::new(Mutex::new(u64::MAX));
    let err_count = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    let total_response_size = Arc::new(Mutex::new(0u64));
    let http_errors = Arc::new(Mutex::new(HttpErrorStats::new()));
    for _ in 0..concurrent_requests {
        let client_builder = reqwest::Client::builder();
        let client = if timeout_secs > 0 {
            client_builder.timeout(Duration::from_secs(timeout_secs)).build().context("构建带超时的http客户端失败")?
        } else {
            client_builder.build().context("构建http客户端失败")?
        };
        let method_clone = method.clone();
        let json_str_clone = json_str.clone();
        let url = url.to_string();
        let histogram_clone = histogram.clone();
        let successful_requests_clone = successful_requests.clone();
        let test_end = test_end;
        let max_response_time_clone = max_response_time.clone();
        let min_response_time_clone = min_response_time.clone();
        let err_count_clone = err_count.clone();
        let total_response_size_clone = total_response_size.clone();
        let total_requests_clone = total_requests.clone();
        let http_errors_clone = http_errors.clone();

        let handle = tokio::spawn(async move {
            while Instant::now() < test_end {
                // 总请求数+1
                *total_requests_clone.lock().await += 1;
                let start = Instant::now();
                let method = Method::from_str(&method_clone.to_uppercase()).expect("无效的方法");
                let mut request = client.request(method, &url);
                if !json_str_clone.is_empty() {
                    let json: Value = serde_json::from_str(&json_str_clone).expect("解析json失败");
                    request = request.json(&json);
                }
                match request.send().await {
                    // 请求成功
                    Ok(response) if response.status().is_success() => {
                        let duration = start.elapsed().as_millis() as u64;
                        // 最大响应时间
                        let mut max_rt = max_response_time_clone.lock().await;
                        *max_rt = (*max_rt).max(duration);
                        // 最小响应时间
                        let mut min_rt = min_response_time_clone.lock().await;
                        *min_rt = (*min_rt).min(duration);
                        // 成功数量
                        *successful_requests_clone.lock().await += 1;
                        // 把响应时间加入统计
                        match histogram_clone.lock().await.increment(duration) {
                            Ok(_) => {},
                            Err(err) => eprintln!("错误:{}", err),
                        }
                        // 计算响应体大小并更新总大小
                        if let Some(content_length) = response.content_length() {
                            let mut total_size = total_response_size_clone.lock().await;
                            *total_size += content_length;
                        }
                        if verbose {
                            match response.bytes().await.context("读取响应体失败"){
                                Ok(bytes) => {
                                    let buffer = String::from_utf8(bytes.to_vec()).expect("无法转换响应体为字符串");
                                    println!("{:+?}", buffer);
                                }
                                Err(e) => {
                                    eprintln!("读取响应失败:{:?}", e.to_string())
                                }
                            };

                        }
                    },
                    Err(e) => {
                        *err_count_clone.lock().await += 1;
                        let status_code: u16;
                        match e.status(){
                            None => {
                                status_code = 0;
                            }
                            Some(code) => {
                                status_code = u16::from(code);
                            }
                        }
                        let err_msg = e.to_string();
                        http_errors_clone.lock().await.increment(status_code, err_msg);
                    }
                    unknown => {
                        println!("未知状态：{:?}", unknown)
                    }
                }
            }
        });

        handles.push(handle);
    }
    // 共享任务状态
    {
        let histogram_clone_for_printing = histogram.clone();
        let successful_requests_clone_for_printing = successful_requests.clone();
        let total_requests_clone_for_printing = total_requests.clone();
        let max_response_time_clone_for_printing = max_response_time.clone();
        let min_response_time_clone_for_printing = min_response_time.clone();
        let err_count_clone_for_printing = err_count.clone();
        let total_response_size_clone_for_printing = total_response_size.clone();
        let http_errors_clone_for_printing = http_errors.clone();
        tokio::spawn(async move {
            share_test_results_periodically(
                test_duration_secs,
                histogram_clone_for_printing,
                successful_requests_clone_for_printing,
                total_requests_clone_for_printing,
                max_response_time_clone_for_printing,
                min_response_time_clone_for_printing,
                err_count_clone_for_printing,
                total_response_size_clone_for_printing,
                http_errors_clone_for_printing,
            ).await;
        });
    }
    // 打印进度条
    match verbose{
        true => {
            for handle in handles {
                handle.await.unwrap();
            }
        }
        false => {
            let pb = ProgressBar::new(100);
            let progress_interval = Duration::from_millis(300);
            let mut interval = interval(progress_interval);
            tokio::spawn(async move {
                while Instant::now() < test_end {
                    interval.tick().await;
                    let elapsed = Instant::now().duration_since(test_start).as_secs_f64();
                    let progress = (elapsed / test_duration_secs as f64) * 100.0;
                    pb.set_position(progress as u64);
                }
                pb.finish_and_clear();
            }).await.unwrap();
            let bar = ProgressBar::new_spinner();
            bar.enable_steady_tick(Duration::from_millis(100));
            bar.set_message("等待所有请求响应");
            for handle in handles {
                handle.await.unwrap();
            }
            bar.finish_with_message("");
            bar.finish();
        }
    }

    let total_duration = Duration::from_secs(test_duration_secs);
    let total_requests = *total_requests.lock().await as f64;
    let successful_requests = *successful_requests.lock().await as f64;
    let success_rate = successful_requests / total_requests * 100.0;
    let histogram = histogram.lock().await;
    let total_response_size_kb = *total_response_size.lock().await as f64 / 1024.0;
    let throughput_kb_s = total_response_size_kb / test_duration_secs as f64;
    let http_errors = http_errors.lock().await.errors.clone();


    let test_result = TestResult {
        total_duration,
        success_rate,
        median_response_time: *histogram.percentile(50.0)?.range().start(),
        response_time_95: *histogram.percentile(95.0)?.range().start(),
        response_time_99: *histogram.percentile(99.0)?.range().start(),
        total_requests: total_requests as i32,
        rps: successful_requests / test_duration_secs as f64,
        max_response_time: *max_response_time.lock().await,
        min_response_time: *min_response_time.lock().await,
        err_count:*err_count.lock().await,
        total_data_kb:total_response_size_kb,
        throughput_per_second_kb: throughput_kb_s,
        http_errors: http_errors.lock().unwrap().clone(),
    };
    Ok(test_result)
}
