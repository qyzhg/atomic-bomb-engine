use serde_json::Value;
#[derive(Clone)]
pub struct AssertOption {
    pub jsonpath: String,
    pub reference_object: Value
}
