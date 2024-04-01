use lazy_static::lazy_static;
use std::collections::VecDeque;
use crate::models;
use tokio::sync::Mutex;
use std::sync::Arc;


lazy_static! {
    pub static ref SINGLE_RESULT_QUEUE: Arc<Mutex<VecDeque<models::result::TestResult>>> = Arc::new(Mutex::new(VecDeque::new()));
    pub static ref SINGLE_SHOULD_STOP: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    pub static ref RESULTS_QUEUE: Arc<Mutex<VecDeque<models::result::BatchResult>>> = Arc::new(Mutex::new(VecDeque::new()));
    pub static ref RESULTS_SHOULD_STOP: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}
