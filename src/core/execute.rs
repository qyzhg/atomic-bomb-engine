use std::str::FromStr;
use std::sync::{Arc};
use histogram::Histogram;
use std::time::{Duration, Instant};
use indicatif::ProgressBar;
use tokio::time::interval;
use anyhow::{Context};
use reqwest::{Method, StatusCode};
use tokio::sync::Mutex;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, HeaderName};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use jsonpath_lib::select;

use crate::core::parse_form_data;
use crate::core::sleep_guard::SleepGuard;
use crate::core::status_share::{RESULT_QUEUE, SHOULD_STOP};
use crate::models::http_error_stats::HttpErrorStats;
use crate::models::result::TestResult;
use crate::models::assert_option::AssertOption;

pub async fn run(
    url: &str,
    test_duration_secs: u64,
    concurrent_requests: i32,
    timeout_secs:u64,
    verbose: bool,
    method: &str,
    json_str: Option<String>,
    form_data_str: Option<String>,
    headers: Option<Vec<String>>,
    cookie: Option<String>,
    should_prevent: bool,
    assert_options: Option<Vec<AssertOption>>
) -> anyhow::Result<TestResult> {
    // 阻止电脑休眠
    let _guard = SleepGuard::new(should_prevent);
    // 请求方法
    let method = method.to_owned();
    // 做数据统计
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
    // 统计错误
    let http_errors = Arc::new(Mutex::new(HttpErrorStats::new()));
    // 校验如果json和form同时发送，直接报错
    if json_str.is_some() && form_data_str.is_some(){
        return Err(anyhow::Error::msg("json和form不允许同时发送"));
    }
    // 如果传入了json，就从这里解析
    let mut json_obj: Arc<Option<Value>> = Arc::new(None);
    if let Some(ref json_str) = json_str {
        let json: Value = serde_json::from_str(json_str).expect("解析json失败");
        json_obj = Arc::new(Some(json));
    }
    // 如果传入了header，就从这里做解析
    let header_map = match headers {
        None => Arc::new(None),
        Some(headers) =>{
            let mut temp_headers_map = HeaderMap::new();
            for header in headers {
                let parts: Vec<&str> = header.splitn(2, ':').collect();
                if parts.len() == 2 {
                    match parts[0].trim().parse::<HeaderName>() {
                        Ok(header_name) =>{
                            match HeaderValue::from_str(parts[1].trim()) {
                                Ok(header_value)=>{
                                    temp_headers_map.insert(header_name, header_value);
                                }
                                Err(err) => {
                                    return Err(anyhow::Error::msg(format!("无法解析header的值{:?}", err)));
                                }
                            }
                        }
                        Err(err) => {
                            return Err(anyhow::Error::msg(format!("无法解析header名称:{:?}", err)));
                        }
                    }
                }
            }
            Arc::new(Some(temp_headers_map))
        }
    };
    // 如果传入了form，就从这里处理
    let form_map = match form_data_str {
        None => Arc::new(None),
        Some(form_str) => {
            let form_data = parse_form_data::parse_form_data(&form_str);
            Arc::new(Some(form_data))
        }
    };
    let assert_options:Arc<Option<Vec<AssertOption>>> = match assert_options{
        None => Arc::new(None),
        Some(v) => {
            Arc::new(Some(v))
        }
    };
    // 开始测试时间
    let test_start = Instant::now();
    // 测试结束时间
    let test_end = test_start + Duration::from_secs(test_duration_secs);
    // 固定并发数
    for _ in 0..concurrent_requests {
        // 构建http客户端
        let client_builder = reqwest::Client::builder();
        // 如果传入了超时时间，客户端添加超时时间
        let client = if timeout_secs > 0 {
            client_builder.timeout(Duration::from_secs(timeout_secs)).build().context("构建带超时的http客户端失败")?
        } else {
            client_builder.build().context("构建http客户端失败")?
        };
        // cookie副本
        let cookie_clone = cookie.clone();
        // 请求方法副本
        let method_clone = method.clone();
        // json副本
        let json_obj_clone = json_obj.clone();
        // form副本
        let form_map_clone = form_map.clone();
        // url转为String
        let url = url.to_string();
        // 统计器副本
        let histogram_clone = histogram.clone();
        // 成功数量统计副本
        let successful_requests_clone = successful_requests.clone();
        // 最大响应时间副本
        let max_response_time_clone = max_response_time.clone();
        // 最小响应时间副本
        let min_response_time_clone = min_response_time.clone();
        // 错误次数副本
        let err_count_clone = err_count.clone();
        // 响应大小副本
        let total_response_size_clone = total_response_size.clone();
        // 请求次数副本
        let total_requests_clone = total_requests.clone();
        // http错误副本
        let http_errors_clone = http_errors.clone();
        // headers副本
        let header_map_clone = header_map.clone();
        // 断言(支持多个)
        let assert_options_clone = assert_options.clone();
        // 开启异步
        let handle = tokio::spawn(async move {
            // 计时
            while Instant::now() < test_end {
                // 总请求数+1
                *total_requests_clone.lock().await += 1;
                // 记录当前接口开始时间
                let start = Instant::now();
                // 构建请求方法
                let method = Method::from_str(&method_clone.to_uppercase()).expect("无效的方法");
                // 构建request
                let mut request = client.request(method, &url);
                /*
                                构建请求头
                    ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓
                */
                // 构建请求头
                let mut headers = HeaderMap::new();
                // 判断是否传入了请求头，如果传入，就一次性加入
                if let Some(header_map) = &*header_map_clone {
                    headers.extend(header_map.iter().map(|(k, v)| (k.clone(), v.clone())));
                }
                // 判断是否传入了cookie，如果传入了，就塞进去
                if let Some(ref cookie_clone) = cookie_clone {
                    match HeaderValue::from_str(cookie_clone) {
                        Ok(h) => {
                            headers.insert(COOKIE, h);
                        },
                        Err(e) => {
                            eprintln!("无法添加cookie:{:?}", e);
                        }
                    }
                }
                // 塞请求头进request
                request = request.headers(headers);
                /*
                                构建请求体
                    ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓
                */
                // 如果有json，就用json的方式发送请求
                if let Some(value) = &*json_obj_clone {
                    request = request.json(value);
                }
                // 判断是否传入了form，如果传入了，就用form形式发送请求
                if let Some(form_map) = &*form_map_clone{
                    request = request.form(form_map);
                }
                // 开始发送请求
                match request.send().await {
                    // 请求成功
                    Ok(response) => {
                        match response.status(){
                            // 正确的状态码
                            StatusCode::OK |
                            StatusCode::CREATED |
                            StatusCode::ACCEPTED |
                            StatusCode::NON_AUTHORITATIVE_INFORMATION |
                            StatusCode::NO_CONTENT |
                            StatusCode::RESET_CONTENT |
                            StatusCode::PARTIAL_CONTENT |
                            StatusCode::MULTI_STATUS |
                            StatusCode::ALREADY_REPORTED |
                            StatusCode::IM_USED |
                            StatusCode::MULTIPLE_CHOICES |
                            StatusCode::MOVED_PERMANENTLY |
                            StatusCode::FOUND |
                            StatusCode::SEE_OTHER |
                            StatusCode::NOT_MODIFIED |
                            StatusCode::USE_PROXY |
                            StatusCode::TEMPORARY_REDIRECT |
                            StatusCode::PERMANENT_REDIRECT => {
                                // 数据统计
                                let duration = start.elapsed().as_millis() as u64;
                                let mut max_rt = max_response_time_clone.lock().await;
                                *max_rt = (*max_rt).max(duration);
                                let mut min_rt = min_response_time_clone.lock().await;
                                *min_rt = (*min_rt).min(duration);
                                match histogram_clone.lock().await.increment(duration) {
                                    Ok(_) => {},
                                    Err(err) => eprintln!("错误:{}", err),
                                }
                                if let Some(content_length) = response.content_length() {
                                    let mut total_size = total_response_size_clone.lock().await;
                                    *total_size += content_length;
                                }

                                let body_bytes = match response.bytes().await {
                                        Ok(bytes) => {
                                            Some(bytes)
                                        },
                                        Err(e) => {
                                            eprintln!("读取响应失败:{:?}", e.to_string());
                                            None
                                        }
                                    };


                                if verbose {
                                    let body_bytes_clone = body_bytes.clone();
                                    let buffer = String::from_utf8(body_bytes_clone.expect("none").to_vec()).expect("无法转换响应体为字符串");
                                    println!("{:+?}", buffer);
                                }
                                // 如果需要断言
                                if let Some(assert_options) = &*assert_options_clone{
                                    // 将响应体解析成字节码
                                    let body_bytes = match body_bytes{
                                        None => {
                                            eprintln!("响应body为空，无法使用jsonpath获取到数据");
                                            continue
                                        }
                                        Some(bytes) =>{
                                            bytes
                                        }
                                    };
                                    // 多断言
                                    for assert_option in assert_options {
                                        let json_value: Value = match serde_json::from_slice(&*body_bytes) {
                                            Err(e) =>{
                                                eprintln!("JSONPath 查询失败: {}", e);
                                                break;
                                            }
                                            Ok(val) => {
                                                val
                                            }
                                        };
                                        // 通过jsonpath提取数据
                                        match select(&json_value, &*assert_option.jsonpath) {
                                            Ok(results) => {
                                                if results.is_empty(){
                                                    eprintln!("没有匹配到任何结果");
                                                    break;
                                                }
                                                if results.len() >1{
                                                    eprintln!("匹配到多个值，无法进行断言");
                                                    break;
                                                }
                                                // 取出匹配到的唯一值
                                                if let Some(result) = results.get(0).map(|&v|v) {
                                                    if *result != assert_option.reference_object{
                                                        // 断言失败， 失败次数+1
                                                        *err_count_clone.lock().await += 1;
                                                        // todo: 将失败情况加入到一个容器中
                                                        eprintln!("断言失败，预期结果：{:?}, 实际结果：{:?}", assert_option.reference_object, result);
                                                        // 退出断言
                                                        break;
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                eprintln!("JSONPath 查询失败: {}", e);
                                                break;
                                            },
                                        }
                                    }

                                }

                                // 正确统计+1
                                *successful_requests_clone.lock().await += 1;
                            }
                            // 状态码错误
                            _ => {
                                *err_count_clone.lock().await += 1;
                                let status_code = u16::from(response.status());
                                let err_msg = format!("HTTP 错误: 状态码 {}", status_code);
                                http_errors_clone.lock().await.increment(status_code, err_msg);
                            }
                        }
                    },
                    // 请求失败，如果有状态码，就记录
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
                }
            }
        });
        // 进池子等待完成
        handles.push(handle);
    }
    // 共享任务状态
    {
        let total_requests_clone = Arc::clone(&total_requests);
        let successful_requests_clone = Arc::clone(&successful_requests);
        let histogram_clone = Arc::clone(&histogram);
        let total_response_size_clone = Arc::clone(&total_response_size);
        let http_errors_clone = Arc::clone(&http_errors);
        let err_count_clone = Arc::clone(&err_count);
        let max_resp_time_clone = Arc::clone(&max_response_time);
        let min_resp_time_clone = Arc::clone(&min_response_time);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            let should_stop = *SHOULD_STOP.lock();
            while !should_stop {
                interval.tick().await;
                let err_count = *err_count_clone.lock().await;
                let max_response_time_c = *max_resp_time_clone.lock().await;
                let min_response_time_c = *min_resp_time_clone.lock().await;
                let total_duration = (Instant::now() - test_start).as_secs_f64();
                let total_requests = *total_requests_clone.lock().await as f64;
                let successful_requests = *successful_requests_clone.lock().await as f64;
                let success_rate = (total_requests - err_count as f64) / total_requests * 100.0;
                let histogram = histogram_clone.lock().await;
                let total_response_size_kb = *total_response_size_clone.lock().await as f64 / 1024.0;
                let throughput_kb_s = total_response_size_kb / total_duration;
                let http_errors = http_errors_clone.lock().await.errors.clone();
                let rps = successful_requests / total_duration;
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
                let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(n) => n.as_millis(),
                    Err(_) => 0,
                };

                let mut queue = RESULT_QUEUE.lock();
                // 如果队列中有了一个数据了，就移除旧数据
                if queue.len() == 1 {
                    queue.pop_front();
                }
                // 添加新结果
                queue.push_back(TestResult{
                    total_duration,
                    success_rate,
                    median_response_time: resp_median_line,
                    response_time_95: resp_95_line,
                    response_time_99: resp_99_line,
                    total_requests: total_requests as i32,
                    rps,
                    max_response_time: max_response_time_c,
                    min_response_time:min_response_time_c,
                    err_count,
                    total_data_kb:total_response_size_kb,
                    throughput_per_second_kb: throughput_kb_s,
                    http_errors: http_errors.lock().unwrap().clone(),
                    timestamp
                });
            }
        });
    }


    // 根据条件判断是否打印进度条，和等待所有任务完成
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
    // 计算返回数据
    let total_duration = (Instant::now() - test_start).as_secs_f64();
    let total_requests = *total_requests.lock().await as f64;
    let successful_requests = *successful_requests.lock().await as f64;
    let success_rate = successful_requests / total_requests * 100.0;
    let histogram = histogram.lock().await;
    let total_response_size_kb = *total_response_size.lock().await as f64 / 1024.0;
    let throughput_kb_s = total_response_size_kb / test_duration_secs as f64;
    let http_errors = http_errors.lock().await.errors.clone();
    let err_count_clone = Arc::clone(&err_count);
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => 0,
    };
    // 返回值
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
        err_count:*err_count_clone.lock().await,
        total_data_kb:total_response_size_kb,
        throughput_per_second_kb: throughput_kb_s,
        http_errors: http_errors.lock().unwrap().clone(),
        timestamp
    };
    let mut should_stop = SHOULD_STOP.lock();
    *should_stop = true;
    eprintln!("压测结束");
    Ok(test_result)
}
