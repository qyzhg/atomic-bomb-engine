use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use std::time::Duration;
use std::cmp::min;
use crate::models::step_option::InnerStepOption;

pub struct ConcurrencyController {
    semaphore: Arc<Semaphore>,
    total_permits: usize,
    step_option: Option<InnerStepOption>,
    // 余数累加
    fractional_accumulator: Mutex<f64>,
}

impl ConcurrencyController {
    pub fn new(total_permits: usize, step_option: Option<InnerStepOption>) -> Self {
        ConcurrencyController {
            semaphore: Arc::new(Semaphore::new(0)),
            total_permits,
            step_option,
            fractional_accumulator: Mutex::new(0.0),
        }
    }

    // 分发许可证
    pub async fn distribute_permits(&self) {
        if let Some(step_option) = &self.step_option {
            let mut permits_added = 0usize;
            // 锁定并立即尝试增加许可
            {
                let mut fractional_accumulator = self.fractional_accumulator.lock().unwrap();
                *fractional_accumulator += step_option.increase_step;
                if *fractional_accumulator >= 1.0 {
                    let initial_permits_to_add = fractional_accumulator.floor() as usize;
                    self.semaphore.add_permits(initial_permits_to_add);
                    permits_added += initial_permits_to_add;
                    *fractional_accumulator -= initial_permits_to_add as f64;
                }
            }
            // 继续分发剩余的许可证
            while permits_added < self.total_permits {
                tokio::time::sleep(Duration::from_secs(step_option.increase_interval)).await;
                let mut fractional_accumulator = self.fractional_accumulator.lock().unwrap();
                *fractional_accumulator += step_option.increase_step;
                let permits_to_add = min(fractional_accumulator.floor() as usize, self.total_permits - permits_added);
                if permits_to_add > 0 {
                    self.semaphore.add_permits(permits_to_add);
                    permits_added += permits_to_add;
                    // 更新累加器
                    *fractional_accumulator -= permits_to_add as f64;
                }
            }
        } else {
            // 一次性分发所有许可
            self.semaphore.add_permits(self.total_permits);
        }
    }

    // 获取信号量
    pub fn get_semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }
}
