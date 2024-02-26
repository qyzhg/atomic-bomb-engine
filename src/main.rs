use std::collections::{HashMap};
use reqwest;
use tokio;
use histogram::{Histogram};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use tokio::time::interval;
use indicatif::ProgressBar;
use clap::Parser;
use anyhow::{Result, Context};
use prettytable::{Table, Row, Cell, format, row};
use std::io::{self, Write};

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
        tokio::spawn(share_test_results_periodically(
            test_duration_secs,
            histogram_clone_for_printing,
            successful_requests_clone_for_printing,
            total_requests_clone_for_printing,
            max_response_time_clone_for_printing,
            min_response_time_clone_for_printing,
            err_count_clone_for_printing,
            total_response_size_clone_for_printing,
            http_errors_clone_for_printing,
        ));
    }
    // 打印进度条
    {
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
    }
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    for handle in handles {
        handle.await.unwrap();
    }
    bar.finish();
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

async fn share_test_results_periodically(
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
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

            table.add_row(row!["指标", "值"]);
            table.add_row(row!["RPS", format!("{:.3}", result.rps)]);
            table.add_row(row!["总请求数", format!("{:?}", result.total_requests)]);
            table.add_row(row!["错误数量", format!("{:?}", result.err_count)]);
            table.add_row(row!["成功率", format!("{:.2}%", result.success_rate)]);
            table.add_row(row!["最大响应时间", format!("{:.2}ms", result.max_response_time)]);
            table.add_row(row!["最小响应时间", format!("{:.2}ms", result.min_response_time)]);
            table.add_row(row!["中位响应时间", format!("{} ms", result.median_response_time)]);
            table.add_row(row!["95%响应时间", format!("{} ms", result.response_time_95)]);
            table.add_row(row!["99%响应时间", format!("{} ms", result.response_time_99)]);
            table.add_row(row!["总吞吐量", format!("{:.2}kb", result.total_data_kb)]);
            println!("压测结果:");
            table.printstd();

            if !result.http_errors.is_empty() {
                let mut errors_table = Table::new();
                errors_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

                errors_table.add_row(row!["错误代码", "错误信息", "次数"]);
                for e in result.http_errors {
                    errors_table.add_row(Row::new(vec![
                        Cell::new(format!("{:03}", e.0.0).as_str()),
                        Cell::new(&format!("{:?}", e.0.1)).style_spec("R"),
                        Cell::new(format!("{}", e.1).as_str()),
                    ]));
                }
                println!("HTTP 错误:");
                errors_table.printstd();
            }

        },
        Err(e) => println!("Error: {}", e),
    }
}

