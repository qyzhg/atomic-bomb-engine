use std::process::{Command, Child};
#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
use winapi::um::winbase::SetThreadExecutionState;
#[cfg(target_os = "windows")]
use winapi::um::winnt::{ES_CONTINUOUS, ES_SYSTEM_REQUIRED};

pub(crate) struct SleepGuard {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    process: Option<Child>,
}

impl SleepGuard {
    pub(crate) fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                winapi::um::winbase::SetThreadExecutionState(
                    winapi::um::winnt::ES_CONTINUOUS | winapi::um::winnt::ES_SYSTEM_REQUIRED,
                );
            }
            SleepGuard {}
        }

        #[cfg(target_os = "macos")]
        {
            let process = Command::new("caffeinate").spawn().ok();
            SleepGuard { process }
        }

        #[cfg(target_os = "linux")]
        {
            let process = Command::new("systemd-inhibit")
                .arg("--what=handle-lid-switch:sleep:idle")
                .arg("--who=RustApplication")
                .arg("--why=Prevent sleep for operation")
                .arg("--mode=block")
                .spawn().ok();
            SleepGuard { process }
        }
    }
}

impl Drop for SleepGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            winapi::um::winbase::SetThreadExecutionState(winapi::um::winnt::ES_CONTINUOUS);
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Some(child) = &mut self.process {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
