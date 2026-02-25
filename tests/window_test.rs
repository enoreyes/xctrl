use std::process::Command;

/// Helper to build the xctrl binary path
fn xctrl_bin() -> String {
    env!("CARGO_BIN_EXE_xctrl").to_string()
}

/// Check if DISPLAY is set (required for window-dependent tests).
fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok()
}

// ==== CLI argument parsing tests (no display needed) ====

#[test]
fn test_window_help_shows_actions() {
    let output = Command::new(xctrl_bin())
        .args(["window", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"), "Help should list 'list'");
    assert!(stdout.contains("focus"), "Help should list 'focus'");
    assert!(stdout.contains("resize"), "Help should list 'resize'");
    assert!(stdout.contains("move"), "Help should list 'move'");
    assert!(stdout.contains("minimize"), "Help should list 'minimize'");
    assert!(stdout.contains("maximize"), "Help should list 'maximize'");
    assert!(
        stdout.contains("fullscreen"),
        "Help should list 'fullscreen'"
    );
}

#[test]
fn test_window_focus_help_shows_title_and_id() {
    let output = Command::new(xctrl_bin())
        .args(["window", "focus", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--title"), "Help should show --title");
    assert!(stdout.contains("--id"), "Help should show --id");
}

#[test]
fn test_window_resize_help_shows_all_options() {
    let output = Command::new(xctrl_bin())
        .args(["window", "resize", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--title"), "Help should show --title");
    assert!(stdout.contains("--id"), "Help should show --id");
    assert!(stdout.contains("--width"), "Help should show --width");
    assert!(stdout.contains("--height"), "Help should show --height");
}

#[test]
fn test_window_move_help_shows_all_options() {
    let output = Command::new(xctrl_bin())
        .args(["window", "move", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--title"), "Help should show --title");
    assert!(stdout.contains("--id"), "Help should show --id");
    assert!(stdout.contains("--x"), "Help should show --x");
    assert!(stdout.contains("--y"), "Help should show --y");
}

#[test]
fn test_window_minimize_help_shows_title_and_id() {
    let output = Command::new(xctrl_bin())
        .args(["window", "minimize", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--title"), "Help should show --title");
    assert!(stdout.contains("--id"), "Help should show --id");
}

#[test]
fn test_window_maximize_help_shows_title_and_id() {
    let output = Command::new(xctrl_bin())
        .args(["window", "maximize", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--title"), "Help should show --title");
    assert!(stdout.contains("--id"), "Help should show --id");
}

#[test]
fn test_window_fullscreen_help_shows_title_and_id() {
    let output = Command::new(xctrl_bin())
        .args(["window", "fullscreen", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--title"), "Help should show --title");
    assert!(stdout.contains("--id"), "Help should show --id");
}

#[test]
fn test_window_resize_requires_width_and_height() {
    let output = Command::new(xctrl_bin())
        .args(["window", "resize", "--title", "Foo"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "resize without --width and --height should fail"
    );
}

#[test]
fn test_window_move_requires_x_and_y() {
    let output = Command::new(xctrl_bin())
        .args(["window", "move", "--title", "Foo"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "move without --x and --y should fail"
    );
}

// ==== Window-not-found error tests (require display) ====

#[test]
fn test_window_focus_nonexistent_title_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["window", "focus", "--title", "NonExistent_12345"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "focus on nonexistent window should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("window not found"),
        "stderr should contain 'window not found': {stderr}"
    );
}

#[test]
fn test_window_focus_nonexistent_title_error_json() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "window", "focus", "--title", "NonExistent_12345"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "focus on nonexistent window should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value =
        serde_json::from_str(&stderr).expect("stderr should be valid JSON");
    assert!(
        json["error"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .contains("window not found"),
        "JSON error should contain 'window not found': {json}"
    );
}

#[test]
fn test_window_focus_nonexistent_id_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["window", "focus", "--id", "999999999"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "focus on nonexistent window id should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("window not found"),
        "stderr should contain 'window not found': {stderr}"
    );
}

#[test]
fn test_window_resize_nonexistent_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args([
            "window",
            "resize",
            "--title",
            "NonExistent_12345",
            "--width",
            "800",
            "--height",
            "600",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "resize on nonexistent window should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("window not found"),
        "stderr should contain 'window not found': {stderr}"
    );
}

#[test]
fn test_window_move_nonexistent_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args([
            "window",
            "move",
            "--title",
            "NonExistent_12345",
            "--x",
            "100",
            "--y",
            "100",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "move on nonexistent window should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("window not found"),
        "stderr should contain 'window not found': {stderr}"
    );
}

#[test]
fn test_window_minimize_nonexistent_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["window", "minimize", "--title", "NonExistent_12345"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "minimize on nonexistent window should fail"
    );
}

#[test]
fn test_window_maximize_nonexistent_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["window", "maximize", "--title", "NonExistent_12345"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "maximize on nonexistent window should fail"
    );
}

#[test]
fn test_window_fullscreen_nonexistent_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["window", "fullscreen", "--title", "NonExistent_12345"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "fullscreen on nonexistent window should fail"
    );
}

// ==== Window list tests (require display) ====

#[test]
fn test_window_list_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["window", "list"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "window list should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_window_list_json_format() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "window", "list"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "window list --json should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON array
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert!(parsed.is_array(), "JSON output should be an array");

    // If there are windows, verify the structure
    if let Some(arr) = parsed.as_array() {
        for win in arr {
            assert!(win.get("title").is_some(), "Window should have 'title'");
            assert!(win.get("id").is_some(), "Window should have 'id'");
            assert!(win.get("x").is_some(), "Window should have 'x'");
            assert!(win.get("y").is_some(), "Window should have 'y'");
            assert!(win.get("width").is_some(), "Window should have 'width'");
            assert!(win.get("height").is_some(), "Window should have 'height'");
            assert!(win.get("pid").is_some(), "Window should have 'pid'");
        }
    }
}

#[test]
fn test_window_list_json_snake_case_keys() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "window", "list"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // No camelCase keys should be present
    assert!(
        !stdout.contains("windowId"),
        "Should not contain camelCase keys"
    );
    assert!(
        !stdout.contains("processId"),
        "Should not contain camelCase keys"
    );
}

// ==== Focus requires --title or --id test ====

#[test]
fn test_window_focus_requires_title_or_id() {
    // When neither --title nor --id given with focus, we still need
    // a display to get past the resolve step (it calls exit_with_error).
    // But clap won't reject it since both are optional - our code handles it.
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["window", "focus"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "focus without --title or --id should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--title") || stderr.contains("--id"),
        "Error should mention --title or --id: {stderr}"
    );
}
