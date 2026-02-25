//! Mouse e2e tests — Linux only, requires Xvfb (DISPLAY must be set).

#[cfg(target_os = "linux")]
mod linux {
    use crate::{has_display, xctrl_bin, SystemLock};
    use std::process::Command;

    /// Hold an X11 connection open for the duration of the test.
    /// Xvfb resets cursor state when all clients disconnect.
    struct X11Keepalive {
        child: Option<std::process::Child>,
    }

    impl X11Keepalive {
        fn new() -> Option<Self> {
            if !has_display() {
                return None;
            }
            let display = std::env::var("DISPLAY").unwrap_or_default();
            let child = Command::new("python3")
                .arg("-c")
                .arg("import ctypes,ctypes.util,time,sys;x=ctypes.cdll.LoadLibrary(ctypes.util.find_library('X11'));x.XOpenDisplay.restype=ctypes.c_void_p;d=x.XOpenDisplay(None);sys.exit(1) if not d else time.sleep(300)")
                .env("DISPLAY", &display)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .ok();
            std::thread::sleep(std::time::Duration::from_millis(300));
            Some(Self { child })
        }
    }

    impl Drop for X11Keepalive {
        fn drop(&mut self) {
            if let Some(ref mut child) = self.child {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    #[test]
    fn e2e_mouse_move_position_roundtrip() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let _lock = SystemLock::acquire("mouse");
        let _keepalive = X11Keepalive::new();

        // Move mouse to (200, 150)
        let output = Command::new(xctrl_bin())
            .args(["mouse", "move", "--x", "200", "--y", "150"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(output.status.success(), "mouse move should exit 0");

        // Small delay for the move to take effect
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Query position
        let output = Command::new(xctrl_bin())
            .args(["mouse", "position", "--json"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(output.status.success(), "mouse position should exit 0");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("position output should be valid JSON");
        assert_eq!(json["x"], 200, "x should be 200");
        assert_eq!(json["y"], 150, "y should be 150");
    }

    #[test]
    fn e2e_mouse_click_with_position_verification() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let _lock = SystemLock::acquire("mouse");
        let _keepalive = X11Keepalive::new();

        // Click at specific position
        let output = Command::new(xctrl_bin())
            .args(["mouse", "click", "--x", "400", "--y", "300"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(output.status.success(), "mouse click should exit 0");

        std::thread::sleep(std::time::Duration::from_millis(100));

        // Verify cursor position is at the click coordinates
        let output = Command::new(xctrl_bin())
            .args(["mouse", "position", "--json"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(output.status.success(), "mouse position should exit 0");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("position output should be valid JSON");
        assert_eq!(json["x"], 400, "x should be 400 after click");
        assert_eq!(json["y"], 300, "y should be 300 after click");
    }

    #[test]
    fn e2e_mouse_drag_moves_cursor() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let _lock = SystemLock::acquire("mouse");
        let _keepalive = X11Keepalive::new();

        let output = Command::new(xctrl_bin())
            .args([
                "mouse", "drag", "--from-x", "100", "--from-y", "100", "--to-x", "500", "--to-y",
                "400",
            ])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(output.status.success(), "mouse drag should exit 0");

        std::thread::sleep(std::time::Duration::from_millis(100));

        // After drag, cursor should be at the to-position
        let output = Command::new(xctrl_bin())
            .args(["mouse", "position", "--json"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
        assert_eq!(json["x"], 500, "x should be 500 after drag");
        assert_eq!(json["y"], 400, "y should be 400 after drag");
    }
}
