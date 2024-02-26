use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct HttpErrorStats {
    pub(crate) errors: Arc<Mutex<HashMap<(u16, String), u32>>>,
}

impl HttpErrorStats {
    pub(crate) fn new() -> Self {
        HttpErrorStats {
            errors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 增加一个错误和对应的出现次数
    pub(crate) fn increment(&self, status_code: u16, error_message: String) {
        let mut errors = self.errors.lock().unwrap();
        *errors.entry((status_code, error_message)).or_insert(0) += 1;
    }
}
