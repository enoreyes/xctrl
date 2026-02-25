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
fn test_display_screenshot_requires_output() {
    let output = Command::new(xctrl_bin())
        .args(["display", "screenshot"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "display screenshot without --output should fail"
    );
}

#[test]
fn test_display_help_shows_actions() {
    let output = Command::new(xctrl_bin())
        .args(["display", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("screenshot"), "Help should list screenshot");
    assert!(stdout.contains("info"), "Help should list info");
    assert!(stdout.contains("list"), "Help should list list");
}

#[test]
fn test_display_screenshot_help_shows_options() {
    let output = Command::new(xctrl_bin())
        .args(["display", "screenshot", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--output"), "Help should show --output");
    assert!(stdout.contains("--x"), "Help should show --x");
    assert!(stdout.contains("--y"), "Help should show --y");
    assert!(stdout.contains("--width"), "Help should show --width");
    assert!(stdout.contains("--height"), "Help should show --height");
}

// ==== Error handling tests ====

#[test]
fn test_display_screenshot_nonexistent_directory_error() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_screenshot_nonexistent_directory_error");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args([
            "display",
            "screenshot",
            "--output",
            "/nonexistent/dir/shot.png",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "screenshot to nonexistent dir should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("nonexistent")
            || stderr.contains("directory")
            || stderr.contains("path")
            || stderr.contains("No such file"),
        "Error should mention the path problem: {stderr}"
    );
}

#[test]
fn test_display_screenshot_nonexistent_directory_error_json() {
    if !has_display() {
        eprintln!(
            "DISPLAY not set, skipping test_display_screenshot_nonexistent_directory_error_json"
        );
        return;
    }
    let output = Command::new(xctrl_bin())
        .args([
            "--json",
            "display",
            "screenshot",
            "--output",
            "/nonexistent/dir/shot.png",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "screenshot to nonexistent dir should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr)
        .unwrap_or_else(|e| panic!("stderr should be valid JSON: {e}\nstderr: {stderr}"));
    assert!(
        json["error"].is_string(),
        "JSON error should have 'error' field: {json}"
    );
}

#[test]
fn test_display_screenshot_invalid_region_zero_width() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_screenshot_invalid_region_zero_width");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args([
            "display",
            "screenshot",
            "--output",
            "/tmp/xctrl_test_zero_w.png",
            "--x",
            "0",
            "--y",
            "0",
            "--width",
            "0",
            "--height",
            "100",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "screenshot with zero width should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("width")
            || stderr.contains("dimension")
            || stderr.contains("invalid")
            || stderr.contains("zero"),
        "Error should mention invalid dimensions: {stderr}"
    );
}

#[test]
fn test_display_screenshot_invalid_region_zero_height() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_screenshot_invalid_region_zero_height");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args([
            "display",
            "screenshot",
            "--output",
            "/tmp/xctrl_test_zero_h.png",
            "--x",
            "0",
            "--y",
            "0",
            "--width",
            "100",
            "--height",
            "0",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "screenshot with zero height should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("height")
            || stderr.contains("dimension")
            || stderr.contains("invalid")
            || stderr.contains("zero"),
        "Error should mention invalid dimensions: {stderr}"
    );
}

// ==== Display info tests (require DISPLAY) ====

#[test]
fn test_display_info_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_info_exits_0");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["display", "info"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "display info should exit 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "display info should produce output");
}

#[test]
fn test_display_info_json_format() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_info_json_format");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "display", "info"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "display info --json should exit 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout should be valid JSON: {e}\nstdout: {stdout}"));
    assert!(
        json["width"].is_number(),
        "JSON should have numeric 'width' field: {json}"
    );
    assert!(
        json["height"].is_number(),
        "JSON should have numeric 'height' field: {json}"
    );
    assert!(
        json["scale_factor"].is_number(),
        "JSON should have numeric 'scale_factor' field: {json}"
    );
}

#[test]
fn test_display_info_json_snake_case_keys() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_info_json_snake_case_keys");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "display", "info"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify all keys are snake_case (no camelCase)
    assert!(
        !stdout.contains("scaleFactor") && !stdout.contains("ScaleFactor"),
        "JSON keys should use snake_case: {stdout}"
    );
    assert!(
        stdout.contains("scale_factor"),
        "JSON should contain 'scale_factor' key: {stdout}"
    );
}

// ==== Display list tests (require DISPLAY) ====

#[test]
fn test_display_list_exits_0() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_list_exits_0");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["display", "list"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "display list should exit 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "display list should produce output");
}

#[test]
fn test_display_list_json_format() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_list_json_format");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "display", "list"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "display list --json should exit 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout should be valid JSON: {e}\nstdout: {stdout}"));
    assert!(
        json.is_array(),
        "display list --json should return array: {json}"
    );
    let arr = json.as_array().unwrap();
    assert!(
        !arr.is_empty(),
        "display list should return at least one monitor"
    );
    // Check first monitor has expected fields
    let monitor = &arr[0];
    assert!(
        monitor["name"].is_string(),
        "monitor should have 'name' field: {monitor}"
    );
    assert!(
        monitor["width"].is_number(),
        "monitor should have 'width' field: {monitor}"
    );
    assert!(
        monitor["height"].is_number(),
        "monitor should have 'height' field: {monitor}"
    );
    assert!(
        monitor["x"].is_number(),
        "monitor should have 'x' field: {monitor}"
    );
    assert!(
        monitor["y"].is_number(),
        "monitor should have 'y' field: {monitor}"
    );
}

#[test]
fn test_display_list_json_snake_case_keys() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_list_json_snake_case_keys");
        return;
    }
    let output = Command::new(xctrl_bin())
        .args(["--json", "display", "list"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout should be valid JSON: {e}\nstdout: {stdout}"));
    let arr = json.as_array().unwrap();
    assert!(!arr.is_empty(), "should have at least one monitor");
    let monitor = &arr[0];
    assert!(
        monitor["scale_factor"].is_number(),
        "monitor should have 'scale_factor' key: {monitor}"
    );
}

// ==== Screenshot tests (require DISPLAY) ====

#[test]
fn test_display_screenshot_creates_file() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_screenshot_creates_file");
        return;
    }
    let path = "/tmp/xctrl_test_screenshot.png";
    // Remove old file if it exists
    let _ = std::fs::remove_file(path);

    let output = Command::new(xctrl_bin())
        .args(["display", "screenshot", "--output", path])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "display screenshot should exit 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Verify file was created
    assert!(
        std::path::Path::new(path).exists(),
        "screenshot file should exist at {path}"
    );
    // Verify it's a valid PNG (check header bytes)
    let data = std::fs::read(path).expect("should be able to read screenshot file");
    assert!(data.len() > 8, "screenshot file should not be empty");
    assert_eq!(
        &data[0..8],
        &[137, 80, 78, 71, 13, 10, 26, 10],
        "file should start with PNG header"
    );
    // Cleanup
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_display_screenshot_region_creates_file() {
    if !has_display() {
        eprintln!("DISPLAY not set, skipping test_display_screenshot_region_creates_file");
        return;
    }
    let path = "/tmp/xctrl_test_region_screenshot.png";
    let _ = std::fs::remove_file(path);

    let output = Command::new(xctrl_bin())
        .args([
            "display",
            "screenshot",
            "--output",
            path,
            "--x",
            "0",
            "--y",
            "0",
            "--width",
            "100",
            "--height",
            "100",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "display screenshot with region should exit 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::path::Path::new(path).exists(),
        "region screenshot file should exist at {path}"
    );
    // Verify it's a valid PNG
    let data = std::fs::read(path).expect("should be able to read screenshot file");
    assert!(data.len() > 8, "screenshot file should not be empty");
    assert_eq!(
        &data[0..8],
        &[137, 80, 78, 71, 13, 10, 26, 10],
        "file should start with PNG header"
    );
    // Cleanup
    let _ = std::fs::remove_file(path);
}
