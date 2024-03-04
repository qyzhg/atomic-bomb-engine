use serde_json::Value;

pub struct AssertOption {
    pub jsonpath: String,
    pub reference_object: Value
}
