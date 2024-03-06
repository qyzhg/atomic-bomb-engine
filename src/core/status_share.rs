use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::collections::VecDeque;
use crate::models;

// 定义一个全局的队列
lazy_static! {
    pub static ref SINGLE_RESULT_QUEUE: Mutex<VecDeque<models::result::TestResult>> = Mutex::new(VecDeque::new());
    pub static ref SINGLE_SHOULD_STOP: Mutex<bool> = Mutex::new(false);
    pub static ref RESULTS_QUEUE: Mutex<VecDeque<models::result::BatchResult>> = Mutex::new(VecDeque::new());
    pub static ref RESULTS_SHOULD_STOP: Mutex<bool> = Mutex::new(false);
}
