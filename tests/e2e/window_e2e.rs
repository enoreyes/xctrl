//! Window e2e tests — Linux only, needs Xvfb.

#[cfg(target_os = "linux")]
mod linux {
    use crate::{has_display, xctrl_bin};
    use std::process::Command;

    #[test]
    fn e2e_window_list_returns_data() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }

        let output = Command::new(xctrl_bin())
            .args(["window", "list", "--json"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            output.status.success(),
            "window list should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("window list should return valid JSON");
        assert!(json.is_array(), "window list should return an array");
        // In a headless Xvfb environment with no WM, the list may be empty,
        // but the command should still succeed and return a valid JSON array
    }

    #[test]
    fn e2e_window_list_text_exits_0() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }

        let output = Command::new(xctrl_bin())
            .args(["window", "list"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            output.status.success(),
            "window list (text) should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e2e_window_focus_nonexistent_errors() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }

        let output = Command::new(xctrl_bin())
            .args(["window", "focus", "--title", "NonExistentWindow_E2E_12345"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            !output.status.success(),
            "window focus on nonexistent window should fail"
        );
    }
}
