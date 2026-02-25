//! End-to-end integration tests for xctrl.
//!
//! These tests spawn the actual xctrl binary and verify behavior.
//! Platform-specific tests use `#[cfg(target_os)]` gates.

mod clipboard_e2e;
mod display_e2e;
mod keyboard_e2e;
mod mouse_e2e;
mod os_actions_e2e;
mod screen_recording_e2e;
mod window_e2e;

use std::process::Command;

/// Helper to build the xctrl binary path.
pub fn xctrl_bin() -> String {
    env!("CARGO_BIN_EXE_xctrl").to_string()
}

/// Check if a display server is available (required for display-dependent tests).
pub fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok()
        || std::env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows")
        || cfg!(target_os = "macos")
}

/// Run an xctrl command and return the Output.
#[allow(dead_code)]
pub fn run_xctrl(args: &[&str]) -> std::process::Output {
    Command::new(xctrl_bin())
        .args(args)
        .output()
        .expect("failed to execute xctrl")
}

/// Run an xctrl command with display env forwarded.
#[allow(dead_code)]
pub fn run_xctrl_with_display(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(xctrl_bin());
    cmd.args(args);
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    cmd.output().expect("failed to execute xctrl")
}

/// File-based lock for serializing system-resource-dependent tests.
pub struct SystemLock {
    _file: std::fs::File,
}

impl SystemLock {
    pub fn acquire(name: &str) -> Self {
        let lock_path = std::env::temp_dir().join(format!("xctrl_e2e_{}.lock", name));
        loop {
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&lock_path)
            {
                #[cfg(unix)]
                {
                    use std::os::unix::io::AsRawFd;
                    let fd = file.as_raw_fd();
                    let ret = unsafe { libc::flock(fd, libc::LOCK_EX) };
                    if ret == 0 {
                        return Self { _file: file };
                    }
                }
                #[cfg(not(unix))]
                {
                    return Self { _file: file };
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}
