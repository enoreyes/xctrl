use std::fs;
use std::process::Command;

/// Helper to build the xctrl binary path
fn xctrl_bin() -> String {
    env!("CARGO_BIN_EXE_xctrl").to_string()
}

/// Check if DISPLAY is set (required for clipboard tests on Linux).
fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok()
}

/// File-based lock to serialize clipboard-dependent tests.
/// Clipboard is a shared system resource, so tests must not run in parallel.
struct ClipboardLock {
    _file: fs::File,
}

impl ClipboardLock {
    fn acquire() -> Self {
        let lock_path = std::env::temp_dir().join("xctrl_clipboard_test.lock");
        loop {
            if let Ok(file) = fs::OpenOptions::new()
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

// ==== CLI argument parsing tests (no display needed) ====

#[test]
fn test_clipboard_set_requires_text() {
    let output = Command::new(xctrl_bin())
        .args(["clipboard", "set"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "clipboard set without text should fail"
    );
}

#[test]
fn test_clipboard_help_shows_actions() {
    let output = Command::new(xctrl_bin())
        .args(["clipboard", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("set") || stdout.contains("Set"),
        "clipboard help should list set action: {stdout}"
    );
    assert!(
        stdout.contains("get") || stdout.contains("Get"),
        "clipboard help should list get action: {stdout}"
    );
    assert!(
        stdout.contains("clear") || stdout.contains("Clear"),
        "clipboard help should list clear action: {stdout}"
    );
}

#[test]
fn test_clipboard_set_help_shows_text_arg() {
    let output = Command::new(xctrl_bin())
        .args(["clipboard", "set", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("text") || stdout.contains("TEXT"),
        "clipboard set help should mention text argument: {stdout}"
    );
}

// ==== Display-dependent tests (clipboard requires X11 on Linux) ====

#[test]
fn test_clipboard_set_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    let output = Command::new(xctrl_bin())
        .args(["clipboard", "set", "hello"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "clipboard set should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_clipboard_get_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    // First set something so get has content
    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "set", "test_get"])
        .output()
        .expect("failed to execute set");

    let output = Command::new(xctrl_bin())
        .args(["clipboard", "get"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "clipboard get should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_clipboard_clear_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    let output = Command::new(xctrl_bin())
        .args(["clipboard", "clear"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "clipboard clear should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_clipboard_round_trip() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    // Set clipboard text
    let set_output = Command::new(xctrl_bin())
        .args(["clipboard", "set", "test123"])
        .output()
        .expect("failed to execute set");
    assert!(
        set_output.status.success(),
        "clipboard set should exit 0, stderr: {}",
        String::from_utf8_lossy(&set_output.stderr)
    );

    // Get clipboard text
    let get_output = Command::new(xctrl_bin())
        .args(["clipboard", "get"])
        .output()
        .expect("failed to execute get");
    assert!(
        get_output.status.success(),
        "clipboard get should exit 0, stderr: {}",
        String::from_utf8_lossy(&get_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&get_output.stdout);
    assert_eq!(
        stdout.trim(),
        "test123",
        "clipboard get should return exactly what was set"
    );
}

#[test]
fn test_clipboard_round_trip_special_chars() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    let special_text = "Hello, world! 🎉 <>&\"'";

    let set_output = Command::new(xctrl_bin())
        .args(["clipboard", "set", special_text])
        .output()
        .expect("failed to execute set");
    assert!(
        set_output.status.success(),
        "clipboard set with special chars should exit 0, stderr: {}",
        String::from_utf8_lossy(&set_output.stderr)
    );

    let get_output = Command::new(xctrl_bin())
        .args(["clipboard", "get"])
        .output()
        .expect("failed to execute get");
    assert!(
        get_output.status.success(),
        "clipboard get should exit 0, stderr: {}",
        String::from_utf8_lossy(&get_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&get_output.stdout);
    assert_eq!(
        stdout.trim(),
        special_text,
        "clipboard should preserve special characters"
    );
}

#[test]
fn test_clipboard_get_json_format() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    // Set clipboard text first
    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "set", "json_test"])
        .output()
        .expect("failed to execute set");

    let output = Command::new(xctrl_bin())
        .args(["clipboard", "get", "--json"])
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "clipboard get --json should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("invalid JSON '{}': {}", stdout.trim(), e));

    assert!(
        json.get("text").is_some(),
        "JSON should have 'text' field: {stdout}"
    );
    assert_eq!(
        json["text"].as_str().unwrap(),
        "json_test",
        "JSON text field should match what was set"
    );
}

#[test]
fn test_clipboard_get_json_snake_case_keys() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "set", "key_test"])
        .output()
        .expect("failed to execute set");

    let output = Command::new(xctrl_bin())
        .args(["--json", "clipboard", "get"])
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "clipboard get --json should exit 0"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("should parse JSON");

    // Verify all keys are snake_case (no camelCase or PascalCase)
    if let Some(obj) = json.as_object() {
        for key in obj.keys() {
            assert!(
                !key.chars().any(|c| c.is_uppercase()),
                "JSON key '{key}' should be snake_case"
            );
        }
    }
}

#[test]
fn test_clipboard_clear_then_get() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    // Set something first
    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "set", "to_be_cleared"])
        .output()
        .expect("failed to execute set");

    // Clear
    let clear_output = Command::new(xctrl_bin())
        .args(["clipboard", "clear"])
        .output()
        .expect("failed to execute clear");
    assert!(
        clear_output.status.success(),
        "clipboard clear should exit 0, stderr: {}",
        String::from_utf8_lossy(&clear_output.stderr)
    );

    // Get after clear — should return empty or exit 0 with empty content
    let get_output = Command::new(xctrl_bin())
        .args(["clipboard", "get"])
        .output()
        .expect("failed to execute get");

    // After clear, get should either:
    // 1. Exit 0 with empty output, or
    // 2. Exit 0 with empty string
    // Both are acceptable
    if get_output.status.success() {
        let stdout = String::from_utf8_lossy(&get_output.stdout);
        assert!(
            stdout.trim().is_empty(),
            "clipboard get after clear should return empty, got: '{}'",
            stdout.trim()
        );
    }
    // If it exits non-zero, that's also acceptable (empty clipboard error)
}

#[test]
fn test_clipboard_clear_then_get_json() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    // Set, then clear
    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "set", "to_be_cleared"])
        .output()
        .expect("failed to execute set");
    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "clear"])
        .output()
        .expect("failed to execute clear");

    // Get in JSON mode
    let get_output = Command::new(xctrl_bin())
        .args(["clipboard", "get", "--json"])
        .output()
        .expect("failed to execute get");

    if get_output.status.success() {
        let stdout = String::from_utf8_lossy(&get_output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("should be valid JSON");
        assert_eq!(
            json["text"].as_str().unwrap_or(""),
            "",
            "clipboard get --json after clear should have empty text"
        );
    }
}

#[test]
fn test_clipboard_overwrite() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let _lock = ClipboardLock::acquire();

    // Set first value
    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "set", "first"])
        .output()
        .expect("failed to execute set");

    // Overwrite with second value
    let _ = Command::new(xctrl_bin())
        .args(["clipboard", "set", "second"])
        .output()
        .expect("failed to execute set");

    // Get should return second value
    let get_output = Command::new(xctrl_bin())
        .args(["clipboard", "get"])
        .output()
        .expect("failed to execute get");

    assert!(get_output.status.success());
    let stdout = String::from_utf8_lossy(&get_output.stdout);
    assert_eq!(
        stdout.trim(),
        "second",
        "clipboard should contain the most recently set value"
    );
}
