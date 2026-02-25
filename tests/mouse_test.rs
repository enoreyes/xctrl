use std::fs;
use std::process::Command;

/// Helper to build the xctrl binary path
fn xctrl_bin() -> String {
    env!("CARGO_BIN_EXE_xctrl").to_string()
}

/// Check if DISPLAY is set (required for display-dependent tests).
fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok()
}

/// Hold an X11 connection open for the duration of the test.
/// Xvfb resets cursor state when all clients disconnect, so we need
/// to keep at least one X11 connection alive during cursor position tests.
/// We use a Python subprocess with ctypes to hold a libX11 connection,
/// since x11rb's RustConnection may not keep the socket alive in all cases.
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
        // Wait for the connection to be established
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

/// File-based lock to serialize display-dependent tests.
/// This prevents cursor position races when tests run in parallel.
struct DisplayLock {
    _file: fs::File,
}

impl DisplayLock {
    fn acquire() -> Self {
        let lock_path = std::env::temp_dir().join("xctrl_mouse_test.lock");
        loop {
            if let Ok(file) = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&lock_path)
            {
                // Use file locking
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

impl Drop for DisplayLock {
    fn drop(&mut self) {
        // File lock is automatically released when the file is closed
    }
}

/// Helper: query cursor position via JSON and return (x, y)
fn get_cursor_position() -> (i64, i64) {
    let output = Command::new(xctrl_bin())
        .args(["mouse", "position", "--json"])
        .output()
        .expect("failed to execute position");
    assert!(output.status.success(), "position --json should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("should parse position JSON");
    (json["x"].as_i64().unwrap(), json["y"].as_i64().unwrap())
}

// ---- CLI arg parsing tests (no display needed) ----

#[test]
fn test_mouse_move_requires_x_and_y() {
    let output = Command::new(xctrl_bin())
        .args(["mouse", "move", "--x", "100"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success(), "should fail without --y");

    let output = Command::new(xctrl_bin())
        .args(["mouse", "move", "--y", "100"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success(), "should fail without --x");
}

#[test]
fn test_mouse_scroll_requires_amount() {
    let output = Command::new(xctrl_bin())
        .args(["mouse", "scroll"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success(), "should fail without --amount arg");
}

#[test]
fn test_mouse_drag_requires_all_args() {
    let output = Command::new(xctrl_bin())
        .args(["mouse", "drag", "--from-x", "0", "--from-y", "0"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "should fail without --to-x/--to-y"
    );
}

// ---- Display-dependent tests (all serialized via DisplayLock) ----

/// Combined test for all cursor-tracking operations.
/// All position-sensitive operations are grouped here and run under a file lock
/// to prevent interference from other parallel tests.
#[test]
fn test_mouse_cursor_tracking() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    // Brief pause to ensure the keepalive connection is fully registered
    // with Xvfb before we start warping
    std::thread::sleep(std::time::Duration::from_millis(50));

    // -- Move and verify round-trip --
    let move_output = Command::new(xctrl_bin())
        .args(["mouse", "move", "--x", "100", "--y", "200"])
        .output()
        .expect("failed to execute move");
    assert!(
        move_output.status.success(),
        "move should exit 0, stderr: {}",
        String::from_utf8_lossy(&move_output.stderr)
    );
    let (x, y) = get_cursor_position();
    assert_eq!(x, 100, "x should be 100 after move");
    assert_eq!(y, 200, "y should be 200 after move");

    // -- Click with position moves cursor --
    let click_output = Command::new(xctrl_bin())
        .args(["mouse", "click", "--x", "250", "--y", "250"])
        .output()
        .expect("failed to execute click");
    assert!(
        click_output.status.success(),
        "click --x --y should exit 0, stderr: {}",
        String::from_utf8_lossy(&click_output.stderr)
    );
    let (x, y) = get_cursor_position();
    assert_eq!(x, 250, "x should be 250 after click --x 250");
    assert_eq!(y, 250, "y should be 250 after click --y 250");

    // -- Drag from (100,100) to (400,300), verify destination --
    let drag_output = Command::new(xctrl_bin())
        .args([
            "mouse", "drag", "--from-x", "100", "--from-y", "100", "--to-x", "400", "--to-y", "300",
        ])
        .output()
        .expect("failed to execute drag");
    assert!(
        drag_output.status.success(),
        "drag should exit 0, stderr: {}",
        String::from_utf8_lossy(&drag_output.stderr)
    );
    let (x, y) = get_cursor_position();
    assert_eq!(x, 400, "cursor should be at to-x after drag");
    assert_eq!(y, 300, "cursor should be at to-y after drag");
}

#[test]
fn test_mouse_position_json_format() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "position", "--json"])
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "position --json should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("invalid JSON '{}': {}", stdout.trim(), e));

    assert!(json.get("x").is_some(), "JSON should have 'x' field");
    assert!(json.get("y").is_some(), "JSON should have 'y' field");
    assert!(json["x"].is_number(), "x should be a number");
    assert!(json["y"].is_number(), "y should be a number");
}

#[test]
fn test_mouse_position_text_format() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "position"])
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "position should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "position should produce output");
}

#[test]
fn test_mouse_click_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "click"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "click should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_mouse_double_click_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "double-click"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "double-click should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_mouse_right_click_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "right-click"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "right-click should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_mouse_scroll_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "scroll", "--amount=-5"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "scroll should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_mouse_drag_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args([
            "mouse", "drag", "--from-x", "100", "--from-y", "100", "--to-x", "500", "--to-y", "500",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "drag should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_mouse_move_out_of_bounds_no_panic() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "move", "--x", "99999", "--y", "99999"])
        .output()
        .expect("failed to execute");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic"),
        "should not panic on out-of-bounds: {}",
        stderr
    );
    assert!(
        output.status.success(),
        "out-of-bounds move should exit 0 (clamped), stderr: {}",
        stderr
    );
}

#[test]
fn test_mouse_move_negative_coords_no_panic() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = DisplayLock::acquire();
    let _keepalive = X11Keepalive::new();

    let output = Command::new(xctrl_bin())
        .args(["mouse", "move", "--x", "-100", "--y", "-100"])
        .output()
        .expect("failed to execute");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic"),
        "should not panic on negative coords: {}",
        stderr
    );
    assert!(
        output.status.success(),
        "negative coords should exit 0 (clamped to 0,0), stderr: {}",
        stderr
    );
}
