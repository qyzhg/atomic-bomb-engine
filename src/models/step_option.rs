use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct StepOption {
    pub increase_step: usize,
    pub increase_interval: u64
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InnerStepOption {
    pub increase_step: f64,
    pub increase_interval: u64
}
