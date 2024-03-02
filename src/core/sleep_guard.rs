#[cfg(target_os = "windows")]
extern crate winapi;

use std::process::{Child, Command};

#[cfg(target_os = "windows")]
use winapi::um::winbase::SetThreadExecutionState;
#[cfg(target_os = "windows")]
use winapi::um::winnt::{ES_CONTINUOUS, ES_SYSTEM_REQUIRED};

pub(crate) struct SleepGuard {
    should_prevent: bool,
    #[cfg(not(target_os = "windows"))]
    process: Option<Child>,
}

impl SleepGuard {
    pub(crate) fn new(should_prevent: bool) -> Self {
        let mut guard = SleepGuard {
            should_prevent,
            #[cfg(not(target_os = "windows"))]
            process: None,
        };

        if should_prevent {
            guard.prevent_sleep();
        }

        guard
    }

    #[cfg(target_os = "windows")]
    fn prevent_sleep(&mut self) {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
        }
    }

    #[cfg(target_os = "macos")]
    fn prevent_sleep(&mut self) {
        let process = Command::new("caffeinate")
            .spawn()
            .ok();
        self.process = process;
    }

    #[cfg(target_os = "linux")]
    fn prevent_sleep(&mut self) {
        let process = Command::new("systemd-inhibit")
            .arg("--what=handle-lid-switch:sleep:idle")
            .arg("--who=RustApplication")
            .arg("--why=Prevent sleep for operation")
            .arg("--mode=block")
            .spawn()
            .ok();
        self.process = process;
    }
}

impl Drop for SleepGuard {
    fn drop(&mut self) {
        if self.should_prevent {
            #[cfg(target_os = "windows")]
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS);
            }

            #[cfg(not(target_os = "windows"))]
            if let Some(mut child) = self.process.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
