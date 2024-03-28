use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct StepOption {
    pub increase_step: usize,
    pub increase_interval: u64
}
