use std::process::Command;

/// Helper to build the xctrl binary path
fn xctrl_bin() -> String {
    env!("CARGO_BIN_EXE_xctrl").to_string()
}

/// Check if DISPLAY is set (required for display-dependent tests).
fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok()
}

// ==== CLI argument parsing tests (no display needed) ====

#[test]
fn test_keyboard_type_requires_text() {
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "type"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard type without text should fail"
    );
}

#[test]
fn test_keyboard_press_requires_key() {
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard press without key should fail"
    );
}

#[test]
fn test_keyboard_hotkey_requires_keys() {
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "hotkey"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard hotkey without keys should fail"
    );
}

#[test]
fn test_keyboard_key_down_requires_key() {
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "key-down"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard key-down without key should fail"
    );
}

#[test]
fn test_keyboard_key_up_requires_key() {
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "key-up"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard key-up without key should fail"
    );
}

// ==== Invalid key name tests ====

#[test]
fn test_keyboard_press_invalid_key() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press", "nonexistentkey"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard press with invalid key should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown key") || stderr.contains("nonexistentkey"),
        "Error should mention the unknown key: {stderr}"
    );
    assert!(
        stderr.contains("Valid keys") || stderr.contains("enter"),
        "Error should list valid keys: {stderr}"
    );
}

#[test]
fn test_keyboard_press_invalid_key_json_mode() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "keyboard", "press", "invalidkey123"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard press with invalid key should exit non-zero in JSON mode"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    // In JSON mode, error should be valid JSON
    let json: Result<serde_json::Value, _> = serde_json::from_str(stderr.trim());
    assert!(
        json.is_ok(),
        "JSON mode error should be valid JSON: {stderr}"
    );
    let json = json.unwrap();
    assert!(
        json.get("error").is_some(),
        "JSON error should have 'error' field: {stderr}"
    );
    let error_msg = json["error"].as_str().unwrap_or("");
    assert!(
        error_msg.contains("Unknown key") || error_msg.contains("invalidkey123"),
        "JSON error should mention the unknown key: {error_msg}"
    );
}

#[test]
fn test_keyboard_key_down_invalid_key() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "key-down", "badkey"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard key-down with invalid key should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown key"),
        "Error should mention unknown key: {stderr}"
    );
}

#[test]
fn test_keyboard_key_up_invalid_key() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "key-up", "badkey"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard key-up with invalid key should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown key"),
        "Error should mention unknown key: {stderr}"
    );
}

#[test]
fn test_keyboard_hotkey_invalid_key() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "hotkey", "ctrl", "invalidkey123"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "keyboard hotkey with invalid key should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown key"),
        "Error should mention unknown key: {stderr}"
    );
}

// ==== Display-dependent tests (keyboard actions require display) ====

#[test]
fn test_keyboard_type_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "type", "Hello, world!"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard type should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_press_enter_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press", "enter"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard press enter should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_press_tab_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press", "tab"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard press tab should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_press_escape_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press", "escape"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard press escape should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_press_backspace_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press", "backspace"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard press backspace should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_press_space_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press", "space"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard press space should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_press_arrow_keys_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    for key in &["up", "down", "left", "right"] {
        let output = Command::new(xctrl_bin())
            .args(["keyboard", "press", key])
            .output()
            .expect("failed to execute");
        assert!(
            output.status.success(),
            "keyboard press {key} should exit 0, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_keyboard_press_function_keys_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    for i in 1..=12 {
        let key = format!("f{i}");
        let output = Command::new(xctrl_bin())
            .args(["keyboard", "press", &key])
            .output()
            .expect("failed to execute");
        assert!(
            output.status.success(),
            "keyboard press {key} should exit 0, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_keyboard_hotkey_ctrl_c_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "hotkey", "ctrl", "c"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard hotkey ctrl c should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_hotkey_ctrl_shift_s_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "hotkey", "ctrl", "shift", "s"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard hotkey ctrl shift s should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_hotkey_alt_tab_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "hotkey", "alt", "tab"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard hotkey alt tab should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_key_down_shift_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "key-down", "shift"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard key-down shift should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_key_up_shift_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "key-up", "shift"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard key-up shift should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_type_empty_string_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "type", ""])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard type with empty string should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_press_single_char_key_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "press", "a"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "keyboard press 'a' should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_keyboard_help_shows_actions() {
    let output = Command::new(xctrl_bin())
        .args(["keyboard", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("type") || stdout.contains("Type"),
        "keyboard help should list type action: {stdout}"
    );
    assert!(
        stdout.contains("press") || stdout.contains("Press"),
        "keyboard help should list press action: {stdout}"
    );
    assert!(
        stdout.contains("hotkey") || stdout.contains("Hotkey"),
        "keyboard help should list hotkey action: {stdout}"
    );
    assert!(
        stdout.contains("key-down") || stdout.contains("KeyDown"),
        "keyboard help should list key-down action: {stdout}"
    );
    assert!(
        stdout.contains("key-up") || stdout.contains("KeyUp"),
        "keyboard help should list key-up action: {stdout}"
    );
}
