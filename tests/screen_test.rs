use std::process::Command;
use std::sync::Mutex;

/// Global mutex to serialize screen recording tests since they share state.
static RECORDING_LOCK: Mutex<()> = Mutex::new(());

/// Get the path to the xctrl binary.
fn xctrl_bin() -> String {
    env!("CARGO_BIN_EXE_xctrl").to_string()
}

/// Check if Xvfb is running or DISPLAY is available.
fn has_display() -> bool {
    std::env::var("DISPLAY")
        .map(|d| !d.is_empty())
        .unwrap_or(false)
}

/// Check if FFmpeg is available.
fn has_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Clean up any leftover state file and stop any active recording.
fn cleanup_recording_state() {
    let bin = xctrl_bin();
    // Try to stop any active recording
    let _ = Command::new(&bin)
        .args(["screen", "record", "stop"])
        .output();
    // Give FFmpeg time to shut down
    std::thread::sleep(std::time::Duration::from_millis(500));
    // Remove state file from new location (~/.xctrl/recording.json) and legacy location
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let state_path = std::path::PathBuf::from(home)
            .join(".xctrl")
            .join("recording.json");
        let _ = std::fs::remove_file(state_path);
    }
    let _ = std::fs::remove_file("/tmp/xctrl-recording.json");
}

// -- CLI Parsing / Help tests --

#[test]
fn test_screen_help_shows_record() {
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["screen", "--help"])
        .output()
        .expect("failed to run xctrl");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.to_lowercase().contains("record"),
        "screen --help should mention record: {combined}"
    );
}

#[test]
fn test_screen_record_help_shows_actions() {
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["screen", "record", "--help"])
        .output()
        .expect("failed to run xctrl");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.to_lowercase().contains("start"),
        "record --help should mention start: {combined}"
    );
    assert!(
        combined.to_lowercase().contains("stop"),
        "record --help should mention stop: {combined}"
    );
    assert!(
        combined.to_lowercase().contains("status"),
        "record --help should mention status: {combined}"
    );
}

#[test]
fn test_screen_record_start_help_shows_options() {
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["screen", "record", "start", "--help"])
        .output()
        .expect("failed to run xctrl");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("--output"),
        "start --help should mention --output: {combined}"
    );
    assert!(
        combined.contains("--framerate"),
        "start --help should mention --framerate: {combined}"
    );
}

#[test]
fn test_screen_record_start_requires_output() {
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["screen", "record", "start"])
        .output()
        .expect("failed to run xctrl");
    assert!(
        !output.status.success(),
        "start without --output should fail"
    );
}

// -- Cold status test --

#[test]
fn test_screen_record_cold_status() {
    let _lock = RECORDING_LOCK.lock().unwrap();
    cleanup_recording_state();
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["screen", "record", "status"])
        .output()
        .expect("failed to run xctrl");
    assert!(
        output.status.success(),
        "cold status should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("not recording") || stdout.contains("recording"),
        "cold status should indicate not recording: {stdout}"
    );
}

#[test]
fn test_screen_record_cold_status_json() {
    let _lock = RECORDING_LOCK.lock().unwrap();
    cleanup_recording_state();
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["--json", "screen", "record", "status"])
        .output()
        .expect("failed to run xctrl");
    assert!(
        output.status.success(),
        "cold status --json should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("cold status should return valid JSON");
    assert_eq!(
        json["recording"], false,
        "cold status should show recording: false"
    );
}

// -- Stop when not recording --

#[test]
fn test_screen_record_stop_when_not_recording() {
    let _lock = RECORDING_LOCK.lock().unwrap();
    cleanup_recording_state();
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["screen", "record", "stop"])
        .output()
        .expect("failed to run xctrl");
    assert!(
        !output.status.success(),
        "stop when not recording should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("no active recording"),
        "stop should say 'no active recording': {stderr}"
    );
}

#[test]
fn test_screen_record_stop_when_not_recording_json() {
    let _lock = RECORDING_LOCK.lock().unwrap();
    cleanup_recording_state();
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args(["--json", "screen", "record", "stop"])
        .output()
        .expect("failed to run xctrl");
    assert!(
        !output.status.success(),
        "stop when not recording should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("JSON stop error should be valid JSON");
    assert!(
        json["error"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .contains("no active recording"),
        "JSON error should mention 'no active recording': {json}"
    );
}

// -- Full lifecycle test (requires DISPLAY + FFmpeg) --

#[test]
fn test_screen_record_full_lifecycle() {
    if !has_display() || !has_ffmpeg() {
        eprintln!("Skipping full lifecycle test: no DISPLAY or FFmpeg");
        return;
    }

    let _lock = RECORDING_LOCK.lock().unwrap();
    cleanup_recording_state();
    let bin = xctrl_bin();
    let output_path = "/tmp/xctrl_test_recording.mp4";

    // Clean up any previous test file
    let _ = std::fs::remove_file(output_path);

    // 1. Start recording
    let start_output = Command::new(&bin)
        .args(["screen", "record", "start", "--output", output_path])
        .output()
        .expect("failed to run start");
    assert!(
        start_output.status.success(),
        "start should exit 0, stderr: {}",
        String::from_utf8_lossy(&start_output.stderr)
    );

    // 2. Check status (should be recording)
    let status_output = Command::new(&bin)
        .args(["--json", "screen", "record", "status"])
        .output()
        .expect("failed to run status");
    assert!(status_output.status.success(), "status should exit 0");
    let status_json: serde_json::Value = serde_json::from_str(
        &String::from_utf8_lossy(&status_output.stdout)
            .trim()
            .to_string(),
    )
    .expect("status should return valid JSON");
    assert_eq!(
        status_json["recording"], true,
        "status should show recording: true"
    );
    assert_eq!(
        status_json["output"], output_path,
        "status should show correct output path"
    );
    assert!(
        status_json["pid"].as_u64().is_some(),
        "status should include a pid"
    );

    // 3. Wait for some recording
    std::thread::sleep(std::time::Duration::from_secs(2));

    // 4. Stop recording
    let stop_output = Command::new(&bin)
        .args(["screen", "record", "stop"])
        .output()
        .expect("failed to run stop");
    assert!(
        stop_output.status.success(),
        "stop should exit 0, stderr: {}",
        String::from_utf8_lossy(&stop_output.stderr)
    );

    // 5. Verify output file exists and has non-zero size
    let metadata = std::fs::metadata(output_path).expect("output file should exist after stop");
    assert!(
        metadata.len() > 0,
        "output file should have non-zero size, got {} bytes",
        metadata.len()
    );

    // 6. Verify status is now not recording
    let final_status = Command::new(&bin)
        .args(["--json", "screen", "record", "status"])
        .output()
        .expect("failed to run final status");
    let final_json: serde_json::Value = serde_json::from_str(
        &String::from_utf8_lossy(&final_status.stdout)
            .trim()
            .to_string(),
    )
    .expect("final status should return valid JSON");
    assert_eq!(
        final_json["recording"], false,
        "final status should show recording: false"
    );

    // Clean up
    let _ = std::fs::remove_file(output_path);
}

// -- Double-start test (requires DISPLAY + FFmpeg) --

#[test]
fn test_screen_record_double_start() {
    if !has_display() || !has_ffmpeg() {
        eprintln!("Skipping double-start test: no DISPLAY or FFmpeg");
        return;
    }

    let _lock = RECORDING_LOCK.lock().unwrap();
    cleanup_recording_state();
    let bin = xctrl_bin();
    let output_path = "/tmp/xctrl_test_double_start.mp4";
    let output_path2 = "/tmp/xctrl_test_double_start2.mp4";
    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(output_path2);

    // Start first recording
    let start1 = Command::new(&bin)
        .args(["screen", "record", "start", "--output", output_path])
        .output()
        .expect("failed to run start 1");
    assert!(
        start1.status.success(),
        "first start should succeed, stderr: {}",
        String::from_utf8_lossy(&start1.stderr)
    );

    // Try to start a second recording
    let start2 = Command::new(&bin)
        .args(["screen", "record", "start", "--output", output_path2])
        .output()
        .expect("failed to run start 2");
    assert!(!start2.status.success(), "second start should fail");
    let stderr = String::from_utf8_lossy(&start2.stderr);
    assert!(
        stderr
            .to_lowercase()
            .contains("recording already in progress"),
        "double-start should say 'recording already in progress': {stderr}"
    );

    // Clean up: stop the first recording
    let _ = Command::new(&bin)
        .args(["screen", "record", "stop"])
        .output();
    std::thread::sleep(std::time::Duration::from_secs(1));
    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(output_path2);
}

// -- Output directory validation test --

#[test]
fn test_screen_record_start_nonexistent_directory() {
    if !has_ffmpeg() {
        eprintln!("Skipping nonexistent directory test: no FFmpeg");
        return;
    }

    let _lock = RECORDING_LOCK.lock().unwrap();
    cleanup_recording_state();
    let bin = xctrl_bin();
    let output = Command::new(&bin)
        .args([
            "screen",
            "record",
            "start",
            "--output",
            "/nonexistent/dir/rec.mp4",
        ])
        .output()
        .expect("failed to run xctrl");
    assert!(
        !output.status.success(),
        "start to nonexistent dir should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("directory")
            || stderr.to_lowercase().contains("does not exist"),
        "should mention missing directory: {stderr}"
    );
}
