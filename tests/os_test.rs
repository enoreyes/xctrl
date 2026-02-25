use std::process::Command;

/// Helper to build the xctrl binary path
fn xctrl_bin() -> String {
    env!("CARGO_BIN_EXE_xctrl").to_string()
}

/// Check if DISPLAY is set (required for X11-dependent tests).
fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok()
}

// ==== CLI argument parsing tests (no display needed) ====

#[test]
fn test_os_help_shows_actions() {
    let output = Command::new(xctrl_bin())
        .args(["os", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("open-url"), "Help should list 'open-url'");
    assert!(stdout.contains("open-app"), "Help should list 'open-app'");
    assert!(stdout.contains("notify"), "Help should list 'notify'");
    assert!(
        stdout.contains("frontmost-app"),
        "Help should list 'frontmost-app'"
    );
    assert!(stdout.contains("list-apps"), "Help should list 'list-apps'");
}

#[test]
fn test_os_open_url_help_shows_url_arg() {
    let output = Command::new(xctrl_bin())
        .args(["os", "open-url", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("url") || stdout.contains("URL"),
        "open-url help should mention url arg: {stdout}"
    );
}

#[test]
fn test_os_open_app_help_shows_name_arg() {
    let output = Command::new(xctrl_bin())
        .args(["os", "open-app", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("name") || stdout.contains("Application"),
        "open-app help should mention name arg: {stdout}"
    );
}

#[test]
fn test_os_notify_help_shows_title_and_body() {
    let output = Command::new(xctrl_bin())
        .args(["os", "notify", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--title"),
        "notify help should show --title: {stdout}"
    );
    assert!(
        stdout.contains("--body"),
        "notify help should show --body: {stdout}"
    );
}

#[test]
fn test_os_open_url_requires_url() {
    let output = Command::new(xctrl_bin())
        .args(["os", "open-url"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success(), "open-url without url should fail");
}

#[test]
fn test_os_open_app_requires_name() {
    let output = Command::new(xctrl_bin())
        .args(["os", "open-app"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "open-app without name should fail"
    );
}

#[test]
fn test_os_notify_requires_title_and_body() {
    let output = Command::new(xctrl_bin())
        .args(["os", "notify"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "notify without --title and --body should fail"
    );
}

#[test]
fn test_os_notify_requires_body() {
    let output = Command::new(xctrl_bin())
        .args(["os", "notify", "--title", "Hello"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "notify without --body should fail"
    );
}

#[test]
fn test_os_notify_requires_title() {
    let output = Command::new(xctrl_bin())
        .args(["os", "notify", "--body", "World"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "notify without --title should fail"
    );
}

// ==== Non-existent app error handling ====

#[test]
fn test_os_open_app_nonexistent_error() {
    let output = Command::new(xctrl_bin())
        .args(["os", "open-app", "NonExistentApp_12345"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "open-app with non-existent app should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("NonExistentApp_12345") || stderr.contains("not found"),
        "Error should mention the app name or 'not found': {stderr}"
    );
}

#[test]
fn test_os_open_app_nonexistent_error_json() {
    let output = Command::new(xctrl_bin())
        .args(["--json", "os", "open-app", "NonExistentApp_12345"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "open-app with non-existent app should fail in JSON mode"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should be valid JSON with an "error" field
    let parsed: serde_json::Value = serde_json::from_str(&stderr)
        .unwrap_or_else(|e| panic!("stderr should be valid JSON: {e}\nstderr: {stderr}"));
    assert!(
        parsed["error"].is_string(),
        "JSON error should have 'error' field: {parsed}"
    );
}

// ==== Display-dependent tests ====

#[test]
fn test_os_frontmost_app_exits() {
    if !has_display() {
        eprintln!("Skipping: DISPLAY not set");
        return;
    }
    // In a headless environment (Xvfb), there may be no frontmost app,
    // so we just verify it runs without crashing. It may exit 0 or 1.
    let output = Command::new(xctrl_bin())
        .args(["os", "frontmost-app"])
        .output()
        .expect("failed to execute");
    // Should not panic (no stderr about panic)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic"),
        "frontmost-app should not panic: {stderr}"
    );
}

#[test]
fn test_os_frontmost_app_json_format() {
    if !has_display() {
        eprintln!("Skipping: DISPLAY not set");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "os", "frontmost-app"])
        .output()
        .expect("failed to execute");

    // If it succeeded, verify JSON format
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("stdout should be valid JSON: {e}\nstdout: {stdout}"));
        assert!(
            parsed["name"].is_string(),
            "JSON should have 'name' field: {parsed}"
        );
        assert!(
            parsed["pid"].is_number(),
            "JSON should have 'pid' field: {parsed}"
        );
    } else {
        // In headless without a WM, this might fail. Verify error JSON.
        let stderr = String::from_utf8_lossy(&output.stderr);
        let parsed: serde_json::Value = serde_json::from_str(&stderr)
            .unwrap_or_else(|e| panic!("stderr should be valid JSON: {e}\nstderr: {stderr}"));
        assert!(
            parsed["error"].is_string(),
            "JSON error should have 'error' field: {parsed}"
        );
    }
}

#[test]
fn test_os_list_apps_exits_0() {
    if !has_display() {
        eprintln!("Skipping: DISPLAY not set");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["os", "list-apps"])
        .output()
        .expect("failed to execute");
    // Should exit 0 even if no apps are running (empty list is fine)
    assert!(
        output.status.success(),
        "list-apps should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_os_list_apps_json_format() {
    if !has_display() {
        eprintln!("Skipping: DISPLAY not set");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "os", "list-apps"])
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "list-apps --json should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout should be valid JSON: {e}\nstdout: {stdout}"));
    assert!(
        parsed.is_array(),
        "list-apps --json should return a JSON array: {parsed}"
    );

    // If there are entries, check they have the right fields
    if let Some(arr) = parsed.as_array() {
        for entry in arr {
            assert!(
                entry["name"].is_string(),
                "Each entry should have 'name': {entry}"
            );
            assert!(
                entry["pid"].is_number(),
                "Each entry should have 'pid': {entry}"
            );
        }
    }
}

#[test]
fn test_os_list_apps_json_snake_case_keys() {
    if !has_display() {
        eprintln!("Skipping: DISPLAY not set");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "os", "list-apps"])
        .output()
        .expect("failed to execute");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Verify snake_case keys — no camelCase
        assert!(
            !stdout.contains("\"appName\""),
            "Should not have camelCase keys"
        );
        assert!(
            !stdout.contains("\"processId\""),
            "Should not have camelCase keys"
        );
    }
}

// ==== Notification test (may require D-Bus) ====

#[test]
fn test_os_notify_sends_notification() {
    // Notification may fail in headless environments without D-Bus daemon.
    // We verify the command runs and handles the error gracefully.
    let output = Command::new(xctrl_bin())
        .args([
            "os",
            "notify",
            "--title",
            "Test",
            "--body",
            "Test notification",
        ])
        .output()
        .expect("failed to execute");

    // Should not panic regardless
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic"),
        "notify should not panic: {stderr}"
    );

    // If exit 0, great. If not, it should have a clear error.
    if !output.status.success() {
        assert!(
            stderr.contains("Error") || stderr.contains("error"),
            "On failure, should print clear error: {stderr}"
        );
    }
}

#[test]
fn test_os_notify_json_mode() {
    let output = Command::new(xctrl_bin())
        .args([
            "--json",
            "os",
            "notify",
            "--title",
            "Test",
            "--body",
            "Test notification",
        ])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic"),
        "notify should not panic: {stderr}"
    );

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("\"status\"") && stdout.contains("\"ok\""),
            "Success JSON should contain status:ok: {stdout}"
        );
    } else {
        // Verify error JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stderr);
        assert!(
            parsed.is_ok(),
            "Error in JSON mode should be valid JSON: {stderr}"
        );
    }
}

// ==== open-url test ====

#[test]
fn test_os_open_url_exits() {
    // open-url may fail in headless environments (no browser), but should not panic
    let output = Command::new(xctrl_bin())
        .args(["os", "open-url", "https://example.com"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic"),
        "open-url should not panic: {stderr}"
    );
}
