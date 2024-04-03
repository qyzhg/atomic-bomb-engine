use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Clone, Serialize, Deserialize)]
pub struct JsonpathExtract{
    pub key: String,
    pub jsonpath: String
}


#[derive(Clone, Serialize, Deserialize)]
pub struct SetupApiEndpoint {
    pub name: String,
    pub url: String,
    pub method: String,
    pub timeout_secs: u64,
    pub json: Option<Value>,
    pub form_data: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
    pub cookies: Option<String>,
    pub jsonpath_extract: Option<Vec<JsonpathExtract>>
}
