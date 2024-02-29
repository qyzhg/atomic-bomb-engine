use std::collections::VecDeque;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::models;

lazy_static! {
    pub static ref MESSAGES: Mutex<VecDeque<models::result::TestResult>> = Mutex::new(VecDeque::new());
    pub static ref SHOULD_STOP: Mutex<bool> = Mutex::new(false);
}
