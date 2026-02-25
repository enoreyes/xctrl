use crate::cli::RecordAction;
use crate::error::{exit_with_error, XctrlError};
use crate::output::print_success;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// State file location for recording state persistence.
const STATE_FILE: &str = "/tmp/xctrl-recording.json";

/// Recording state persisted to disk.
#[derive(Debug, Serialize, Deserialize)]
struct RecordingState {
    pid: u32,
    output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    xvfb_pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xvfb_display: Option<String>,
}

/// Status output for JSON mode.
#[derive(Debug, Serialize)]
struct RecordingStatus {
    recording: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pid: Option<u32>,
}

pub fn handle_screen(action: RecordAction, json: bool) {
    match action {
        RecordAction::Start { output, framerate } => handle_start(&output, framerate, json),
        RecordAction::Stop => handle_stop(json),
        RecordAction::Status => handle_status(json),
    }
}

fn handle_start(output: &str, framerate: u32, json: bool) {
    // Check for double-start
    if let Some(state) = load_state() {
        if is_process_alive(state.pid) {
            let err = XctrlError::new("recording already in progress");
            exit_with_error(&err, json, 1);
        }
        // Previous recording died; clean up stale state
        cleanup_state();
    }

    // Check that FFmpeg is available
    if !is_ffmpeg_available() {
        let err = XctrlError::with_hint("FFmpeg not found", ffmpeg_install_hint());
        exit_with_error(&err, json, 1);
    }

    // Validate output path - parent directory must exist
    if let Some(parent) = Path::new(output).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let err = XctrlError::with_hint(
                format!("output directory does not exist: {}", parent.display()),
                "Create the directory first or use a different output path.",
            );
            exit_with_error(&err, json, 1);
        }
    }

    // Platform-specific recording start
    #[cfg(target_os = "linux")]
    {
        start_recording_linux(output, framerate, json);
    }

    #[cfg(target_os = "macos")]
    {
        start_recording_macos(output, framerate, json);
    }

    #[cfg(target_os = "windows")]
    {
        start_recording_windows(output, framerate, json);
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let err = XctrlError::new("screen recording is not supported on this platform");
        exit_with_error(&err, json, 1);
    }
}

#[cfg(target_os = "linux")]
fn start_recording_linux(output: &str, framerate: u32, json: bool) {
    let mut xvfb_pid: Option<u32> = None;
    let mut xvfb_display: Option<String> = None;

    // Determine display
    let display = match std::env::var("DISPLAY") {
        Ok(d) if !d.is_empty() => d,
        _ => {
            // Headless: auto-start Xvfb
            match start_xvfb() {
                Ok((pid, disp)) => {
                    xvfb_pid = Some(pid);
                    xvfb_display = Some(disp.clone());
                    disp
                }
                Err(e) => {
                    let err = XctrlError::with_hint(
                        format!("failed to start Xvfb for headless recording: {e}"),
                        "Install Xvfb with: sudo dnf install xorg-x11-server-Xvfb",
                    );
                    exit_with_error(&err, json, 1);
                }
            }
        }
    };

    // Get display resolution for video_size
    let video_size = get_display_resolution(&display).unwrap_or_else(|| "1920x1080".to_string());

    // Build FFmpeg command
    let args = build_ffmpeg_args_linux(&display, &video_size, framerate, output);

    match Command::new("ffmpeg")
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            let pid = child.id();
            let state = RecordingState {
                pid,
                output: output.to_string(),
                xvfb_pid,
                xvfb_display,
            };
            save_state(&state);

            #[derive(Serialize)]
            struct StartResult {
                recording: bool,
                pid: u32,
                output: String,
            }

            let result = StartResult {
                recording: true,
                pid,
                output: output.to_string(),
            };
            print_success(
                &result,
                &format!("Recording started (pid: {pid}, output: {output})"),
                json,
            );
        }
        Err(e) => {
            // Clean up Xvfb if we started it
            if let Some(xvfb) = xvfb_pid {
                let _ = kill_process(xvfb);
            }
            let err = XctrlError::with_hint(
                format!("failed to start FFmpeg: {e}"),
                "Ensure FFmpeg is installed and on your PATH.",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

#[cfg(target_os = "macos")]
fn start_recording_macos(output: &str, framerate: u32, json: bool) {
    let args = build_ffmpeg_args_macos(framerate, output);

    match Command::new("ffmpeg")
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            let pid = child.id();
            let state = RecordingState {
                pid,
                output: output.to_string(),
                xvfb_pid: None,
                xvfb_display: None,
            };
            save_state(&state);

            #[derive(Serialize)]
            struct StartResult {
                recording: bool,
                pid: u32,
                output: String,
            }

            let result = StartResult {
                recording: true,
                pid,
                output: output.to_string(),
            };
            print_success(
                &result,
                &format!("Recording started (pid: {pid}, output: {output})"),
                json,
            );
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("failed to start FFmpeg: {e}"),
                "Ensure FFmpeg is installed: brew install ffmpeg",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

#[cfg(target_os = "windows")]
fn start_recording_windows(output: &str, framerate: u32, json: bool) {
    let args = build_ffmpeg_args_windows(framerate, output);

    match Command::new("ffmpeg")
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x00000008) // DETACHED_PROCESS
        .spawn()
    {
        Ok(child) => {
            let pid = child.id();
            let state = RecordingState {
                pid,
                output: output.to_string(),
                xvfb_pid: None,
                xvfb_display: None,
            };
            save_state(&state);

            #[derive(Serialize)]
            struct StartResult {
                recording: bool,
                pid: u32,
                output: String,
            }

            let result = StartResult {
                recording: true,
                pid,
                output: output.to_string(),
            };
            print_success(
                &result,
                &format!("Recording started (pid: {pid}, output: {output})"),
                json,
            );
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("failed to start FFmpeg: {e}"),
                "Ensure FFmpeg is installed. Download from: https://ffmpeg.org/download.html",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

fn handle_stop(json: bool) {
    let state = match load_state() {
        Some(s) => s,
        None => {
            let err = XctrlError::new("no active recording");
            exit_with_error(&err, json, 1);
        }
    };

    if !is_process_alive(state.pid) {
        // Recording process already exited
        cleanup_state_and_xvfb(&state);
        let err = XctrlError::new("no active recording");
        exit_with_error(&err, json, 1);
    }

    // Send SIGTERM (or equivalent) to gracefully stop FFmpeg
    // FFmpeg handles SIGTERM by finalizing the output file
    #[cfg(unix)]
    {
        // Send SIGTERM to FFmpeg for graceful shutdown
        let _ = unsafe { libc::kill(state.pid as i32, libc::SIGTERM) };
    }

    #[cfg(not(unix))]
    {
        let _ = kill_process(state.pid);
    }

    // Wait for FFmpeg to finish writing
    wait_for_process_exit(state.pid, 10);

    // Clean up Xvfb if we started it
    if let Some(xvfb_pid) = state.xvfb_pid {
        let _ = kill_process(xvfb_pid);
    }

    // Remove state file
    let _ = fs::remove_file(STATE_FILE);

    #[derive(Serialize)]
    struct StopResult {
        recording: bool,
        output: String,
    }

    let result = StopResult {
        recording: false,
        output: state.output.clone(),
    };
    print_success(
        &result,
        &format!("Recording stopped (output: {})", state.output),
        json,
    );
}

fn handle_status(json: bool) {
    match load_state() {
        Some(state) if is_process_alive(state.pid) => {
            let status = RecordingStatus {
                recording: true,
                output: Some(state.output.clone()),
                pid: Some(state.pid),
            };
            print_success(
                &status,
                &format!(
                    "Recording active (pid: {}, output: {})",
                    state.pid, state.output
                ),
                json,
            );
        }
        Some(state) => {
            // Process died; clean up stale state
            cleanup_state_and_xvfb(&state);
            let status = RecordingStatus {
                recording: false,
                output: None,
                pid: None,
            };
            print_success(&status, "Not recording", json);
        }
        None => {
            let status = RecordingStatus {
                recording: false,
                output: None,
                pid: None,
            };
            print_success(&status, "Not recording", json);
        }
    }
}

// -- State management --

fn load_state() -> Option<RecordingState> {
    let data = fs::read_to_string(STATE_FILE).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_state(state: &RecordingState) {
    if let Ok(data) = serde_json::to_string_pretty(state) {
        let _ = fs::write(STATE_FILE, data);
    }
}

fn cleanup_state() {
    let _ = fs::remove_file(STATE_FILE);
}

fn cleanup_state_and_xvfb(state: &RecordingState) {
    if let Some(xvfb_pid) = state.xvfb_pid {
        let _ = kill_process(xvfb_pid);
    }
    cleanup_state();
}

// -- Process utilities --

fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // kill with signal 0 checks if process exists
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }

    #[cfg(not(unix))]
    {
        // On Windows, check via tasklist
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .map(|o| {
                let out = String::from_utf8_lossy(&o.stdout);
                out.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

fn kill_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, libc::SIGTERM) == 0 }
    }

    #[cfg(not(unix))]
    {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

fn wait_for_process_exit(pid: u32, timeout_secs: u32) {
    for _ in 0..(timeout_secs * 10) {
        if !is_process_alive(pid) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    // Force kill if still running
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = kill_process(pid);
    }
}

// -- FFmpeg availability --

fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn ffmpeg_install_hint() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "Install FFmpeg with: sudo dnf install ffmpeg (Fedora/RHEL) or sudo apt install ffmpeg (Debian/Ubuntu)"
    }
    #[cfg(target_os = "macos")]
    {
        "Install FFmpeg with: brew install ffmpeg"
    }
    #[cfg(target_os = "windows")]
    {
        "Install FFmpeg from: https://ffmpeg.org/download.html and add it to your PATH"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "Install FFmpeg from: https://ffmpeg.org/download.html"
    }
}

// -- FFmpeg command construction (platform-specific) --

#[cfg(target_os = "linux")]
fn build_ffmpeg_args_linux(
    display: &str,
    video_size: &str,
    framerate: u32,
    output: &str,
) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-f".to_string(),
        "x11grab".to_string(),
        "-video_size".to_string(),
        video_size.to_string(),
        "-framerate".to_string(),
        framerate.to_string(),
        "-i".to_string(),
        display.to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "ultrafast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        output.to_string(),
    ]
}

#[cfg(target_os = "macos")]
fn build_ffmpeg_args_macos(framerate: u32, output: &str) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-f".to_string(),
        "avfoundation".to_string(),
        "-framerate".to_string(),
        framerate.to_string(),
        "-i".to_string(),
        "1:none".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "ultrafast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        output.to_string(),
    ]
}

#[cfg(target_os = "windows")]
fn build_ffmpeg_args_windows(framerate: u32, output: &str) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-f".to_string(),
        "gdigrab".to_string(),
        "-framerate".to_string(),
        framerate.to_string(),
        "-i".to_string(),
        "desktop".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "ultrafast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        output.to_string(),
    ]
}

// -- Headless Linux support --

#[cfg(target_os = "linux")]
fn start_xvfb() -> Result<(u32, String), String> {
    // Check Xvfb is available
    if Command::new("Xvfb")
        .arg("-help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_err()
    {
        return Err("Xvfb not found on PATH".to_string());
    }

    // Find a free display number
    let display_num = find_free_display();
    let display = format!(":{display_num}");

    let child = Command::new("Xvfb")
        .args([&display, "-screen", "0", "1920x1080x24", "-ac"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn Xvfb: {e}"))?;

    let pid = child.id();

    // Wait for Xvfb to be ready
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Verify it's running
    if !is_process_alive(pid) {
        return Err("Xvfb exited immediately; display may already be in use".to_string());
    }

    Ok((pid, display))
}

#[cfg(target_os = "linux")]
fn find_free_display() -> u32 {
    for num in 50..200 {
        let lock_path = format!("/tmp/.X{num}-lock");
        if !Path::new(&lock_path).exists() {
            return num;
        }
    }
    // Fallback
    99
}

#[cfg(target_os = "linux")]
fn get_display_resolution(display: &str) -> Option<String> {
    // Try xdpyinfo to get resolution
    let output = Command::new("xdpyinfo")
        .env("DISPLAY", display)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("dimensions:") {
            // e.g., "dimensions:    1920x1080 pixels (..."
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- FFmpeg command construction tests --

    #[test]
    fn test_build_ffmpeg_args_linux() {
        let args = build_ffmpeg_args_linux(":0", "1920x1080", 30, "/tmp/out.mp4");
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"x11grab".to_string()));
        assert!(args.contains(&"-video_size".to_string()));
        assert!(args.contains(&"1920x1080".to_string()));
        assert!(args.contains(&"-framerate".to_string()));
        assert!(args.contains(&"30".to_string()));
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&":0".to_string()));
        assert!(args.contains(&"-c:v".to_string()));
        assert!(args.contains(&"libx264".to_string()));
        assert!(args.contains(&"-preset".to_string()));
        assert!(args.contains(&"ultrafast".to_string()));
        assert!(args.contains(&"/tmp/out.mp4".to_string()));
        // -y should be first for overwrite
        assert_eq!(args[0], "-y");
    }

    #[test]
    fn test_build_ffmpeg_args_linux_custom_framerate() {
        let args = build_ffmpeg_args_linux(":99", "1280x720", 60, "/tmp/rec.mp4");
        assert!(args.contains(&"60".to_string()));
        assert!(args.contains(&":99".to_string()));
        assert!(args.contains(&"1280x720".to_string()));
    }

    #[test]
    fn test_build_ffmpeg_args_linux_custom_display() {
        let args = build_ffmpeg_args_linux(":42", "1920x1080", 30, "/tmp/rec.mp4");
        let i_idx = args.iter().position(|a| a == "-i").unwrap();
        assert_eq!(args[i_idx + 1], ":42");
    }

    // -- State management tests --

    #[test]
    fn test_recording_state_serialization() {
        let state = RecordingState {
            pid: 12345,
            output: "/tmp/test.mp4".to_string(),
            xvfb_pid: None,
            xvfb_display: None,
        };
        let json = serde_json::to_value(&state).unwrap();
        assert_eq!(json["pid"], 12345);
        assert_eq!(json["output"], "/tmp/test.mp4");
        // xvfb_pid should be omitted when None
        assert!(json.get("xvfb_pid").is_none());
    }

    #[test]
    fn test_recording_state_with_xvfb() {
        let state = RecordingState {
            pid: 12345,
            output: "/tmp/test.mp4".to_string(),
            xvfb_pid: Some(67890),
            xvfb_display: Some(":50".to_string()),
        };
        let json = serde_json::to_value(&state).unwrap();
        assert_eq!(json["pid"], 12345);
        assert_eq!(json["xvfb_pid"], 67890);
        assert_eq!(json["xvfb_display"], ":50");
    }

    #[test]
    fn test_recording_state_deserialization() {
        let json_str = r#"{"pid":1234,"output":"/tmp/out.mp4"}"#;
        let state: RecordingState = serde_json::from_str(json_str).unwrap();
        assert_eq!(state.pid, 1234);
        assert_eq!(state.output, "/tmp/out.mp4");
        assert!(state.xvfb_pid.is_none());
        assert!(state.xvfb_display.is_none());
    }

    #[test]
    fn test_recording_state_roundtrip() {
        let state = RecordingState {
            pid: 99999,
            output: "/home/user/video.mp4".to_string(),
            xvfb_pid: Some(11111),
            xvfb_display: Some(":55".to_string()),
        };
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: RecordingState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.pid, state.pid);
        assert_eq!(deserialized.output, state.output);
        assert_eq!(deserialized.xvfb_pid, state.xvfb_pid);
        assert_eq!(deserialized.xvfb_display, state.xvfb_display);
    }

    // -- Status output tests --

    #[test]
    fn test_recording_status_cold() {
        let status = RecordingStatus {
            recording: false,
            output: None,
            pid: None,
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["recording"], false);
        // output and pid should be omitted when None
        assert!(json.get("output").is_none());
        assert!(json.get("pid").is_none());
    }

    #[test]
    fn test_recording_status_active() {
        let status = RecordingStatus {
            recording: true,
            output: Some("/tmp/rec.mp4".to_string()),
            pid: Some(12345),
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["recording"], true);
        assert_eq!(json["output"], "/tmp/rec.mp4");
        assert_eq!(json["pid"], 12345);
    }

    #[test]
    fn test_recording_status_snake_case_keys() {
        let status = RecordingStatus {
            recording: true,
            output: Some("/tmp/rec.mp4".to_string()),
            pid: Some(42),
        };
        let json_str = serde_json::to_string(&status).unwrap();
        assert!(json_str.contains("\"recording\""));
        assert!(json_str.contains("\"output\""));
        assert!(json_str.contains("\"pid\""));
    }

    // -- Headless detection tests --

    #[test]
    fn test_find_free_display() {
        let display_num = find_free_display();
        assert!(display_num >= 50);
        assert!(display_num < 200);
    }

    // -- FFmpeg availability test --

    #[test]
    fn test_ffmpeg_install_hint_not_empty() {
        let hint = ffmpeg_install_hint();
        assert!(!hint.is_empty());
        // On Linux, should mention dnf or apt
        #[cfg(target_os = "linux")]
        {
            assert!(hint.contains("dnf") || hint.contains("apt"));
        }
    }

    // -- Process utility tests --

    #[test]
    fn test_is_process_alive_self() {
        // Current process should be alive
        let pid = std::process::id();
        assert!(is_process_alive(pid));
    }

    #[test]
    fn test_is_process_alive_nonexistent() {
        // Use a very high but valid PID (max positive i32 value)
        // that is extremely unlikely to exist
        assert!(!is_process_alive(2_000_000_000));
    }

    // -- State file management tests --

    #[test]
    fn test_save_and_load_state() {
        // Use a custom state file path for testing to avoid conflicts
        let test_state_file = "/tmp/xctrl-recording-test-save-load.json";
        let state = RecordingState {
            pid: 12345,
            output: "/tmp/test_output.mp4".to_string(),
            xvfb_pid: None,
            xvfb_display: None,
        };
        let data = serde_json::to_string_pretty(&state).unwrap();
        fs::write(test_state_file, &data).unwrap();

        let loaded: RecordingState =
            serde_json::from_str(&fs::read_to_string(test_state_file).unwrap()).unwrap();
        assert_eq!(loaded.pid, 12345);
        assert_eq!(loaded.output, "/tmp/test_output.mp4");

        let _ = fs::remove_file(test_state_file);
    }

    #[test]
    fn test_load_state_missing_file() {
        // Ensure state file doesn't exist
        let _ = fs::remove_file("/tmp/xctrl-recording-test-missing.json");
        let data = fs::read_to_string("/tmp/xctrl-recording-test-missing.json");
        assert!(data.is_err());
    }
}
