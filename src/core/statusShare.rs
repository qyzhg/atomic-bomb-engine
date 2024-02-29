use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::collections::VecDeque;
use crate::models;

// 定义一个全局的队列
lazy_static! {
    pub static ref QUEUE: Mutex<VecDeque<models::result::TestResult>> = Mutex::new(VecDeque::new());
}
