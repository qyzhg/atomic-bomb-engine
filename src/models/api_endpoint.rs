use std::collections::HashMap;
use serde_json::Value;
use crate::models::assert_option::AssertOption;


#[derive(Clone)]
pub struct ApiEndpoint {
    pub name: String,
    pub url: String,
    pub method: String,
    pub timeout_secs: u64,
    pub weight: u32,
    pub json: Option<Value>,
    pub headers: Option<HashMap<String, String>>,
    pub cookies: Option<String>,
    pub assert_options: Option<Vec<AssertOption>>,
}
