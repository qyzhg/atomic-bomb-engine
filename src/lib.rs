// use pyo3::prelude::*;
// use pyo3::wrap_pyfunction;
// use std::collections::HashMap;
// use pyo3::types::{PyDict};
// mod models;
// mod core;
// #[pyclass]
// pub struct PyTestResult {
//     #[pyo3(get, set)]
//     total_duration: u64,
//     #[pyo3(get, set)]
//     success_rate: f64,
//     #[pyo3(get, set)]
//     median_response_time: u64,
//     #[pyo3(get, set)]
//     response_time_95: u64,
//     #[pyo3(get, set)]
//     response_time_99: u64,
//     #[pyo3(get, set)]
//     total_requests: i32,
//     #[pyo3(get, set)]
//     rps: f64,
//     #[pyo3(get, set)]
//     max_response_time: u64,
//     #[pyo3(get, set)]
//     min_response_time: u64,
//     #[pyo3(get, set)]
//     err_count: i32,
//     #[pyo3(get, set)]
//     total_data_kb: f64,
//     #[pyo3(get, set)]
//     throughput_per_second_kb: f64,
//     #[pyo3(get, set)]
//     http_errors: HashMap<(u16, String), u32>,
// }
//
// #[pymethods]
// impl PyTestResult {
//     #[new]
//     fn new(
//         total_duration: u64,
//         success_rate: f64,
//         median_response_time: u64,
//         response_time_95: u64,
//         response_time_99: u64,
//         total_requests: i32,
//         rps: f64,
//         max_response_time: u64,
//         min_response_time: u64,
//         err_count: i32,
//         total_data_kb: f64,
//         throughput_per_second_kb: f64,
//         http_errors: HashMap<(u16, String), u32>,
//     ) -> Self {
//         PyTestResult {
//             total_duration,
//             success_rate,
//             median_response_time,
//             response_time_95,
//             response_time_99,
//             total_requests,
//             rps,
//             max_response_time,
//             min_response_time,
//             err_count,
//             total_data_kb,
//             throughput_per_second_kb,
//             http_errors,
//         }
//     }
// }
//
// #[pyfunction]
// fn run_wrapper(
//     py: Python,
//     url: &str,
//     test_duration_secs: u64,
//     concurrent_requests: i32,
//     timeout_secs: u64,
//     verbose: bool,
//     method: &str,
//     json_str: Option<String>,
//     form_data_str: Option<String>,
//     headers: Option<Vec<String>>,
//     cookie: Option<String>,
// ) -> PyResult<PyObject> {
//     pyo3_asyncio::tokio::future_into_py(py, async move {
//         let result = core::execute::run(
//             url,
//             test_duration_secs,
//             concurrent_requests,
//             timeout_secs,
//             verbose,
//             method,
//             json_str,
//             form_data_str,
//             headers,
//             cookie,
//         ).await;
//
//         Python::with_gil(|py| {
//             match result {
//                 Ok(test_result) => {
//                     let py_result = PyDict::new(py);
//                     py_result.set_item("success_rate", test_result.success_rate)?;
//                     py_result.set_item("error_count", test_result.err_count)?;
//                     Ok(py_result.into())
//                 },
//                 Err(e) => {
//                     // 转换错误为Python异常
//                     Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
//                         format!("Error: {}", e)
//                     ))
//                 }
//             }
//         })
//     })
// }
//
//
// #[pymodule]
// fn py_core(py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(run_wrapper, m)?)?;
//     Ok(())
// }
