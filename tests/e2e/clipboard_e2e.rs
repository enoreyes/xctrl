//! Clipboard e2e tests — all OSes, Linux needs Xvfb (DISPLAY).

use crate::{has_display, xctrl_bin, SystemLock};
use std::process::Command;

/// Skip test if on Linux without display.
fn should_skip() -> bool {
    if cfg!(target_os = "linux") && !has_display() {
        eprintln!("SKIP: Linux requires DISPLAY for clipboard");
        return true;
    }
    false
}

fn run_cmd(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(xctrl_bin());
    cmd.args(args);
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    cmd.output().expect("failed to run xctrl")
}

/// All clipboard tests share one lock to avoid races on the global clipboard.
/// We run them in a single test function to guarantee serial execution.
#[test]
fn e2e_clipboard_all() {
    if should_skip() {
        return;
    }
    let _lock = SystemLock::acquire("clipboard");

    // ── set/get round-trip ──
    {
        let text = "e2e roundtrip test 12345";
        let output = run_cmd(&["clipboard", "set", text]);
        assert!(
            output.status.success(),
            "clipboard set should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = run_cmd(&["clipboard", "get"]);
        assert!(
            output.status.success(),
            "clipboard get should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            text,
            "clipboard get should return the same text that was set"
        );
    }

    // ── JSON round-trip ──
    {
        let text = "json roundtrip e2e";
        let output = run_cmd(&["clipboard", "set", text]);
        assert!(output.status.success());

        let output = run_cmd(&["clipboard", "get", "--json"]);
        assert!(
            output.status.success(),
            "clipboard get --json should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("output should be valid JSON");
        assert_eq!(json["text"], text, "JSON text field should match set value");
    }

    // ── clear ──
    {
        let output = run_cmd(&["clipboard", "set", "to be cleared"]);
        assert!(output.status.success());

        let output = run_cmd(&["clipboard", "clear"]);
        assert!(
            output.status.success(),
            "clipboard clear should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = run_cmd(&["clipboard", "get"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if output.status.success() {
            assert!(
                stdout.trim().is_empty(),
                "clipboard should be empty after clear, got: '{}'",
                stdout.trim()
            );
        }
        // Non-zero exit for empty clipboard is also acceptable
    }

    // ── special characters ──
    {
        let text = "special chars: !@#$%^&*()_+-={}[]|\\:\";<>?,./~`";
        let output = run_cmd(&["clipboard", "set", text]);
        assert!(output.status.success());

        let output = run_cmd(&["clipboard", "get"]);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            text,
            "clipboard should preserve special characters"
        );
    }
}
