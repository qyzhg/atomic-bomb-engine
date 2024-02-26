use std::collections::{HashMap, HashSet};
use reqwest;
use tokio;
use histogram::{Histogram};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use tokio::time::interval;
use indicatif::ProgressBar;
use clap::Parser;
use anyhow::{Result, Context};
use reqwest::{Error, Response, StatusCode};


struct TestResult {
    total_duration: Duration,
    success_rate: f64,
    median_response_time: u64,
    response_time_95: u64,
    response_time_99: u64,
    total_requests: i32,
    rps: f64,
    max_response_time: u64,
    min_response_time: u64,
    err_count: i32,
    total_data_kb: f64,
    throughput_per_second_kb: f64,
    http_errors: HashMap<(u16, String), u32>
}

struct HttpErrorStats {
    errors: Arc<Mutex<HashMap<(u16, String), u32>>>,
}

impl HttpErrorStats {
    fn new() -> Self {
        HttpErrorStats {
            errors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 增加一个错误和对应的出现次数
    fn increment(&self, status_code: u16, error_message: String) {
        let mut errors = self.errors.lock().unwrap();
        *errors.entry((status_code, error_message)).or_insert(0) += 1;
    }
}


async fn run(url: &str, test_duration_secs: u64, concurrent_requests: i32, timeout_secs:u64) -> Result<TestResult> {
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
                *total_requests_clone.lock().unwrap() += 1;
                let start = Instant::now();
                match client.get(&url).send().await {
                    // 请求成功
                    Ok(response) if response.status().is_success() => {
                        let duration = start.elapsed().as_millis() as u64;
                        // 最大响应时间
                        let mut max_rt = max_response_time_clone.lock().unwrap();
                        *max_rt = (*max_rt).max(duration);
                        // 最小响应时间
                        let mut min_rt = min_response_time_clone.lock().unwrap();
                        *min_rt = (*min_rt).min(duration);
                        // 成功数量
                        *successful_requests_clone.lock().unwrap() += 1;
                        // 把响应时间加入统计
                        match histogram_clone.lock().unwrap().increment(duration) {
                            Ok(_) => {},
                            Err(err) => eprintln!("错误:{}", err),
                        }
                        // 计算响应体大小并更新总大小
                        if let Some(content_length) = response.content_length() {
                            let mut total_size = total_response_size_clone.lock().unwrap();
                            *total_size += content_length;
                        }
                    },
                    Err(e) => {
                        *err_count_clone.lock().unwrap() += 1;
                        let mut status_code: u16;
                        match e.status(){
                            None => {
                                status_code = 0;
                            }
                            Some(code) => {
                                status_code = u16::from(code);
                            }
                        }
                        let err_msg = e.to_string();
                        http_errors_clone.lock().unwrap().increment(status_code, err_msg);
                    }
                    unknown => {
                        println!("未知状态：{:?}", unknown)
                    }
                }
            }
        });

        handles.push(handle);
    }

    {
        // 打印进度条
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
            pb.finish_with_message("done");
        }).await.unwrap();
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_duration = Duration::from_secs(test_duration_secs);
    let total_requests = *total_requests.lock().unwrap() as f64;
    let successful_requests = *successful_requests.lock().unwrap() as f64;
    let success_rate = successful_requests / total_requests * 100.0;
    let histogram = histogram.lock().unwrap();
    let total_response_size_kb = *total_response_size.lock().unwrap() as f64 / 1024.0;
    let throughput_kb_s = total_response_size_kb / test_duration_secs as f64;
    let http_errors = http_errors.lock().unwrap().errors.clone();


    let test_result = TestResult {
        total_duration,
        success_rate,
        median_response_time: *histogram.percentile(50.0)?.range().start(),
        response_time_95: *histogram.percentile(95.0)?.range().start(),
        response_time_99: *histogram.percentile(99.0)?.range().start(),
        total_requests: total_requests as i32,
        rps: successful_requests / test_duration_secs as f64,
        max_response_time: *max_response_time.lock().unwrap(),
        min_response_time: *min_response_time.lock().unwrap(),
        err_count:*err_count.lock().unwrap(),
        total_data_kb:total_response_size_kb,
        throughput_per_second_kb: throughput_kb_s,
        http_errors: http_errors.lock().unwrap().clone(),
    };

    Ok(test_result)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 目标地址
    #[arg(short, long)]
    url: String,

    /// 持续时间（秒）
    #[arg(short, long, default_value_t = 1)]
    duration_secs: u64,

    /// 并发数
    #[arg(short, long, default_value_t = 1)]
    concurrent_requests: i32,

    /// 超时时间（秒）
    #[arg(long, default_value_t = 0)]
    timeout: u64,
}
#[tokio::main]
async fn main() {
    let args = Args::parse();

    match run(&args.url, args.duration_secs, args.concurrent_requests, args.timeout).await {
        Ok(result) => {
            println!("测试完成");
            println!("RPS:{:.3}", result.rps);
            println!("总请求数：{:?}", result.total_requests);
            println!("错误数量：{:?}", result.err_count);
            println!("成功率: {:.2}%", result.success_rate);
            println!("最大响应时间:{:.2}ms", result.max_response_time);
            println!("最小响应时间:{:.2}ms", result.min_response_time);
            println!("中位响应时间: {} ms", result.median_response_time);
            println!("95%响应时间: {} ms", result.response_time_95);
            println!("99%响应时间: {} ms", result.response_time_99);
            println!("总吞吐量:{:.2}kb", result.total_data_kb);
            for e in result.http_errors{
                println!("{:03} | 错误:{:?},次数:{}", e.0.0,e.0.1, e.1)
            }

        },
        Err(e) => println!("Error: {}", e),
    }
}