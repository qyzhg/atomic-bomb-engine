use std::fmt::format;
use std::str::FromStr;
use std::sync::{Arc};
use histogram::Histogram;
use std::time::{self,Duration, Instant};
use tokio::time::interval;
use anyhow::{Context, Error};
use reqwest::{Client, Method, Response, StatusCode};
use tokio::sync::{Mutex, Semaphore};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, HeaderName};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use jsonpath_lib::select;

use crate::core::sleep_guard::SleepGuard;
use crate::core::status_share::{RESULT_QUEUE, SHOULD_STOP};
use crate::models::assert_error_stats::AssertErrorStats;
use crate::models::http_error_stats::HttpErrorStats;
use crate::models::result::{ApiResult, TestResult};
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
    // 用arc包装每一个endpoint
    let api_endpoints_arc: Vec<Arc<Mutex<ApiEndpoint>>> = api_endpoints
        .into_iter()
        .map(|endpoint| Arc::new(Mutex::new(endpoint)))
        .collect();
    // 开始测试时间
    let test_start = Instant::now();
    // 测试结束时间
    let test_end = test_start + Duration::from_secs(test_duration_secs);
    // 针对每一个接口开始配置
    for endpoint_arc in api_endpoints_arc.iter() {
        let endpoint = endpoint_arc.lock().await;
        let weight = endpoint.weight.clone();
        let name = endpoint.name.clone();
        let url = endpoint.url.clone();
        drop(endpoint);
        // 计算权重比例
        let weight_ratio = weight as f64 / total_weight as f64;
        // 计算每个接口的并发量
        let mut concurrency_for_endpoint = ((concurrent_requests as f64) * weight_ratio).round() as usize;
        // 如果这个接口的并发量四舍五入成0了， 就把他定为1
        if concurrency_for_endpoint == 0{
            concurrency_for_endpoint = 1
        }
        // 接口数据的统计
        let api_histogram = Arc::new(Mutex::new(Histogram::new(14, 20).unwrap()));
        // 接口成功数据统计
        let api_successful_requests = Arc::new(Mutex::new(0));
        // 接口请求总数统计
        let api_total_requests = Arc::new(Mutex::new(0));
        // 接口统计最大响应时间
        let api_max_response_time = Arc::new(Mutex::new(0u64));
        // 接口统计最小响应时间
        let api_min_response_time = Arc::new(Mutex::new(u64::MAX));
        // 接口统计错误数量
        let api_err_count = Arc::new(Mutex::new(0));
        // 接口响应大小
        let api_total_response_size = Arc::new(Mutex::new(0u64));
        // 初始化api结果
        let mut api_result = Arc::new(Mutex::new(ApiResult{
            name: name.clone(),
            url,
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
        }));
        // 根据权重算出来每个接口的并发量
        for _ in 0..concurrency_for_endpoint {
            // 数据桶副本
            let histogram_clone = histogram.clone();
            // 任务名称
            let api_name_clone = name.clone();
            // api数据桶副本
            let api_histogram_clone = api_histogram.clone();
            // api成功数量统计副本
            let api_successful_requests_clone = api_successful_requests.clone();
            // api总统计数量统计副本
            let api_total_requests_clone = api_total_requests.clone();
            // api最大响应时间副本
            let api_max_response_time_clone = api_max_response_time.clone();
            // api最小响应时间副本
            let api_min_response_time_clone = api_min_response_time.clone();
            // api错误数量统计副本
            let api_err_count_clone = api_err_count.clone();
            // api结果副本
            let api_result_clone = api_result.clone();
            // api吞吐量副本
            let api_total_response_size_clone = api_total_response_size.clone();
            // 总请求数记录副本
            let total_requests_clone = Arc::clone(&total_requests);
            // 每个接口端点副本
            let endpoint_clone = Arc::clone(endpoint_arc);
            // 最大响应时间副本
            let max_response_time_clone = max_response_time.clone();
            // 响应大小统计副本
            let total_response_size_clone = total_response_size.clone();
            // 最小响应时间副本
            let min_response_time_clone = min_response_time.clone();
            // 错误次数副本
            let err_count_clone = err_count.clone();
            // 断言错误副本
            let assert_errors_clone = assert_errors.clone();
            // 成功次数副本
            let successful_requests_clone = successful_requests.clone();
            // http错误副本
            let http_errors_clone = http_errors.clone();
            // 构建http客户端
            let client_builder = Client::builder();
            // 如果有超时时间就将client设置
            let client = if endpoint_clone.lock().await.timeout_secs > 0 {
                client_builder.timeout(Duration::from_secs(endpoint_clone.lock().await.timeout_secs)).build().context("构建带超时的http客户端失败")?
            } else {
                client_builder.build().context("构建http客户端失败")?
            };
            // 开启并发
            let handle: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async move {
                while Instant::now() < test_end {
                    // 总请求数
                    *total_requests_clone.lock().await += 1;
                    // api请求数
                    *api_total_requests_clone.lock().await += 1;
                    // 请求方法副本
                    let method_clone = endpoint_clone.lock().await.method.clone();
                    // json副本
                    let json_obj_clone = endpoint_clone.lock().await.json.clone();
                    // headers副本
                    let headers_clone = endpoint_clone.lock().await.headers.clone();
                    // cookie副本
                    let cookie_clone = endpoint_clone.lock().await.cookies.clone();
                    // 断言副本
                    let assert_options_clone = endpoint_clone.lock().await.assert_options.clone();
                    // 构建请求方式
                    let method = Method::from_str(&method_clone.to_uppercase()).map_err(|_| Error::msg("构建请求方法失败"))?;
                    // 构建请求
                    let mut request = client.request(method, endpoint_clone.lock().await.url.clone());
                    // 构建请求头
                    let mut headers = HeaderMap::new();
                    if let Some(headers_map) = headers_clone {
                        headers.extend(headers_map.iter().map(|(k, v)| {
                            let header_name = k.parse::<HeaderName>().expect("无效的header名称");
                            let header_value = v.parse::<HeaderValue>().expect("无效的header值");
                            (header_name, header_value)
                        }));
                    }
                    // 构建cookies
                    if let Some(ref c) = cookie_clone{
                        match HeaderValue::from_str(c){
                            Ok(h) => {
                                headers.insert(COOKIE, h);
                            },
                            Err(e) =>{
                                return Err(Error::msg(format!("设置cookie失败:{:?}", e)))
                            }
                        }
                    }
                    // 构建json请求
                    if let Some(json_value) = json_obj_clone{
                        request = request.json(&json_value);
                    }
                    // 记录开始时间
                    let start = Instant::now();
                    // 发送请求
                    match request.send().await {
                        Ok(response) => {
                            let status = response.status();
                            match status{
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
                                    /*
                                    ---------------
                                        请求成功
                                    ---------------
                                    */
                                    let mut api_histogram = api_histogram_clone.lock().await;
                                    // 响应时间
                                    let duration = start.elapsed().as_millis() as u64;
                                    // 最大请求时间
                                    let mut max_rt = max_response_time_clone.lock().await;
                                    *max_rt = (*max_rt).max(duration);
                                    // api最大请求时间
                                    let mut api_max_rt = api_max_response_time_clone.lock().await;
                                    *api_max_rt = (*api_max_rt).max(duration);
                                    // 最小响应时间
                                    let mut min_rt = min_response_time_clone.lock().await;
                                    *min_rt = (*min_rt).min(duration);
                                    // api最小响应时间
                                    let mut api_min_rt = api_min_response_time_clone.lock().await;
                                    *api_min_rt = (*api_min_rt).min(duration);
                                    // 将数据放入全局统计桶
                                    if let Err(e) = histogram_clone.lock().await.increment(duration){
                                        eprintln!("histogram设置数据错误:{:?}", e)
                                    };
                                    // 将数据放入api统计桶
                                    if let Err(e) = api_histogram.increment(duration){
                                        eprintln!("api histogram设置错误:{:?}", e)
                                    }
                                    // 吞吐量统计
                                    if let Some(content_length) = response.content_length() {
                                        let mut total_size = total_response_size_clone.lock().await;
                                        *total_size += content_length;
                                        let mut api_total_size = api_total_response_size_clone.lock().await;
                                        *api_total_size += content_length;
                                    }
                                    // 获取响应
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
                                    // 断言
                                    if let Some(assert_options) = assert_options_clone{
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
                                                            // 将失败情况加入到一个容器中
                                                            assert_errors_clone.
                                                                lock().
                                                                await.
                                                                increment(
                                                                    String::from(endpoint_clone.lock().await.url.clone()),
                                                                    format!(
                                                                        "{:?}-预期结果：{:?}, 实际结果：{:?}",api_name_clone ,assert_option.reference_object, result
                                                                    )
                                                                );
                                                            if verbose{
                                                                eprintln!("{:?}-预期结果：{:?}, 实际结果：{:?}",api_name_clone ,assert_option.reference_object, result)
                                                            }
                                                            // 错误数据增加
                                                            *err_count_clone.lock().await += 1;
                                                            *api_err_count_clone.lock().await += 1;
                                                            // 退出断言
                                                            break;
                                                        } else {
                                                            // 正确统计+1
                                                            *successful_requests_clone.lock().await += 1;
                                                            // api正确统计+1
                                                            *api_successful_requests_clone.lock().await += 1;
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    eprintln!("JSONPath 查询失败: {}", e);
                                                    break;
                                                },
                                            }
                                        }

                                    } else{
                                        // 正确统计+1
                                        *successful_requests_clone.lock().await += 1;
                                        // api正确统计+1
                                        *api_successful_requests_clone.lock().await += 1;
                                    }


                                    let api_total_data_bytes = *api_total_response_size_clone.lock().await;
                                    let api_total_data_kb = api_total_data_bytes as f64 / 1024f64;
                                    let api_total_requests = api_total_requests_clone.lock().await.clone();
                                    let api_rps = api_total_requests as f64/ (Instant::now() - test_start).as_secs_f64();
                                    let api_success_rate = *api_successful_requests_clone.lock().await as f64 / api_total_requests as f64 * 100.0;
                                    let throughput_per_second_kb = api_total_data_kb / (Instant::now() - test_start).as_secs_f64();
                                    // 给结果赋值
                                    let  mut api_res = api_result_clone.lock().await;
                                    api_res.response_time_95 = *api_histogram.percentile(95.0)?.range().start();
                                    api_res.response_time_99 = *api_histogram.percentile(99.0)?.range().start();
                                    api_res.median_response_time = *api_histogram.percentile(50.0)?.range().start();
                                    api_res.max_response_time = *api_max_rt;
                                    api_res.min_response_time = *api_min_rt;
                                    api_res.total_requests = api_total_requests;
                                    api_res.total_data_kb = api_total_data_kb;
                                    api_res.rps = api_rps;
                                    api_res.success_rate = api_success_rate;
                                    api_res.err_count = *api_err_count_clone.lock().await;
                                    api_res.throughput_per_second_kb = throughput_per_second_kb;

                                    println!("{:?}",api_res);
                                }
                                // 状态码错误
                                _ =>{
                                    *err_count_clone.lock().await += 1;
                                    *api_err_count_clone.lock().await += 1;
                                    let status_code = u16::from(response.status());
                                    let err_msg = format!("HTTP 错误: 状态码 {}", status_code);
                                    let url = response.url().to_string();
                                    http_errors_clone.lock().await.increment(status_code, err_msg, url);
                                    if verbose{
                                        println!("{:?}-HTTP 错误: 状态码 {:?}",api_name_clone, status_code)
                                    }
                                }
                            }

                        },
                        Err(e) => {
                            *err_count_clone.lock().await += 1;
                            *api_err_count_clone.lock().await+=1;
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
                            http_errors_clone.lock().await.increment(status_code, err_msg, endpoint_clone.lock().await.url.clone());
                        },
                    }
                }
                Ok(())
            });

            handles.push(handle);
        }
    }
    for handle in handles {
        match handle.await {
            Ok(result) => {
                match result {
                    Ok(_) => {
                        if verbose {
                            println!("任务成功完成")
                        }
                    },
                    Err(e) => {
                        return Err(Error::msg(format!("异步任务内部错误:{:?}", e)));
                    },
                }
            },
            Err(e) => {
                return Err(Error::msg(format!("协程被取消或意外停止:{:?}", e)));
            },
        }
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


/*
    单测
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch() {
        let mut assert_vec: Vec<AssertOption> = Vec::new();
        let ref_obj = Value::from(200);
        assert_vec.push(AssertOption{ jsonpath: "$.code".to_string(), reference_object: ref_obj });
        let mut endpoints: Vec<ApiEndpoint> = Vec::new();

        endpoints.push(ApiEndpoint{
            name: "有断言".to_string(),
            url: "https://ooooo.run/api/short/v1/getJumpCount".to_string(),
            method: "GET".to_string(),
            timeout_secs: 0,
            weight: 1,
            json: None,
            headers: None,
            cookies: None,
            assert_options: Some(assert_vec.clone()),
        });

        endpoints.push(ApiEndpoint{
            name: "无断言".to_string(),
            url: "https://ooooo.run/api/short/v1/getJumpCount".to_string(),
            method: "GET".to_string(),
            timeout_secs: 0,
            weight: 3,
            json: None,
            headers: None,
            cookies: None,
            assert_options: None,
        });

        let _ = batch(5, 100, false, false, endpoints).await;
    }
}
