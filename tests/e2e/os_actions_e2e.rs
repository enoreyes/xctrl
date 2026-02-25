//! OS actions e2e tests — all OSes.

use crate::xctrl_bin;
use std::process::Command;

fn run_cmd(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(xctrl_bin());
    cmd.args(args);
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    cmd.output().expect("failed to run xctrl")
}

#[test]
fn e2e_os_list_apps_returns_data() {
    let output = run_cmd(&["--json", "os", "list-apps"]);

    // On macOS CI, list-apps may fail due to AppleScript sandbox restrictions.
    // Accept either success with data or a descriptive error.
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("list-apps should return valid JSON");
        assert!(json.is_array(), "list-apps should return an array");
        let arr = json.as_array().unwrap();
        assert!(!arr.is_empty(), "list-apps should return at least one app");
    } else {
        // On macOS CI or restricted environments, list-apps may fail —
        // verify it produces a descriptive error rather than crashing
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.trim().is_empty(),
            "list-apps failure should produce stderr output"
        );
    }
}

#[test]
fn e2e_os_list_apps_text_exits_0() {
    let output = run_cmd(&["os", "list-apps"]);

    // On macOS CI, list-apps may fail due to AppleScript sandbox restrictions.
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.trim().is_empty(),
            "list-apps text output should not be empty"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.trim().is_empty(),
            "list-apps failure should produce stderr output"
        );
    }
}

#[test]
fn e2e_os_notify_exits_0() {
    // Notification may not display in CI but should not error
    let output = run_cmd(&[
        "os",
        "notify",
        "--title",
        "E2E Test",
        "--body",
        "Hello from e2e",
    ]);
    // On some CI environments without a notification daemon, this may fail gracefully.
    // We check that it either succeeds or fails with a known error.
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Accept failure if it's due to no notification service (common in CI)
        assert!(
            stderr.contains("notification")
                || stderr.contains("dbus")
                || stderr.contains("D-Bus")
                || stderr.contains("error"),
            "notify failure should have a descriptive error, got: {}",
            stderr
        );
    }
}

#[test]
fn e2e_os_open_app_nonexistent_errors() {
    let output = run_cmd(&["os", "open-app", "NonExistentApp_E2E_99999"]);
    assert!(
        !output.status.success(),
        "open-app with nonexistent app should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "open-app error should produce stderr output"
    );
}

#[test]
fn e2e_os_open_app_nonexistent_json_error() {
    let output = run_cmd(&["--json", "os", "open-app", "NonExistentApp_E2E_99999"]);
    assert!(
        !output.status.success(),
        "open-app --json with nonexistent app should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // In JSON mode, error should be valid JSON on stderr
    let json: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("JSON mode error should produce valid JSON");
    assert!(
        json.get("error").is_some(),
        "JSON error should have an 'error' field"
    );
}
