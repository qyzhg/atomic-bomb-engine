use std::collections::VecDeque;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref MESSAGES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}
