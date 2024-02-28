#[cfg(feature = "python-extension")]
use pyo3::prelude::*;
#[cfg(feature = "python-extension")]
use tokio;
#[cfg(feature = "python-extension")]
use pyo3::types::PyDict;
#[cfg(feature = "python-extension")]
use tokio::runtime::Runtime;

mod models;
mod core;

#[cfg(feature = "python-extension")]
#[pyfunction]
fn run_sync(
    py: Python,
    url: String,
    test_duration_secs: u64,
    concurrent_requests: i32,
    timeout_secs: u64,
    verbose: bool,
    method: String,
    json_str: Option<String>,
    form_data_str: Option<String>,
    headers: Option<Vec<String>>,
    cookie: Option<String>,
) -> PyResult<PyObject> {
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async move {
        core::execute::run(
            &url,
            test_duration_secs,
            concurrent_requests,
            timeout_secs,
            verbose,
            &method,
            json_str,
            form_data_str,
            headers,
            cookie,
        ).await
    });

    match result {
        Ok(test_result) => {
            let dict = PyDict::new(py);
            dict.set_item("total_duration", test_result.total_duration)?;
            dict.set_item("success_rate", test_result.success_rate)?;
            dict.set_item("median_response_time", test_result.median_response_time)?;
            dict.set_item("response_time_95", test_result.response_time_95)?;
            dict.set_item("response_time_99", test_result.response_time_99)?;
            dict.set_item("total_requests", test_result.total_requests)?;
            dict.set_item("rps", test_result.rps)?;
            dict.set_item("max_response_time", test_result.max_response_time)?;
            dict.set_item("min_response_time", test_result.min_response_time)?;
            dict.set_item("err_count", test_result.err_count)?;
            dict.set_item("total_data_kb", test_result.total_data_kb)?;
            dict.set_item("throughput_per_second_kb", test_result.throughput_per_second_kb)?;
            if !test_result.http_errors.is_empty(){
                let http_error_dict = PyDict::new(py);
                for ((code, message), count) in test_result.http_errors.iter() {
                    let key = format!("{}|{}", code, message);
                    http_error_dict.set_item(key, *count).unwrap();
                }
                dict.set_item("http_errors", http_error_dict)?;
            }
            Ok(dict.into())
        },
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Error: {:?}", e))),
    }
}
#[cfg(feature = "python-extension")]
#[pymodule]
fn engine(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_sync, m)?)?;
    Ok(())
}
