use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct AssertErrorStats {
    // {(url, 错误信息): 次数}
    pub(crate) errors: Arc<Mutex<HashMap<(String, String), u32>>>,
}

impl AssertErrorStats {
    pub(crate) fn new() -> Self {
        AssertErrorStats {
            errors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 增加一个错误和对应的出现次数
    pub(crate) fn increment(&self, url: String, error_message: String) {
        let mut errors = self.errors.lock().unwrap();
        *errors.entry((url, error_message)).or_insert(0) += 1;
    }
}
