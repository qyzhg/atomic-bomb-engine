use serde_json::Value;
use crate::models::assert_option::AssertOption;

pub struct RequestOption {
    pub url: String,
    pub timeout_secs: u64,
    pub method: String,
    pub json: Option<Value>,
    pub form_data_str: Option<String>,
    pub headers: Option<Vec<String>>,
    pub cookie: Option<String>,
    pub assert_options: Option<Vec<AssertOption>>
}
