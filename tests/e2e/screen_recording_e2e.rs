//! Screen recording e2e tests — Linux only, needs Xvfb + FFmpeg.

#[cfg(target_os = "linux")]
mod linux {
    use crate::{has_display, xctrl_bin, SystemLock};
    use std::process::Command;

    fn has_ffmpeg() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn should_skip() -> bool {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return true;
        }
        if !has_ffmpeg() {
            eprintln!("SKIP: ffmpeg not available");
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

    #[test]
    fn e2e_screen_record_full_lifecycle() {
        if should_skip() {
            return;
        }
        let _lock = SystemLock::acquire("screen_record");

        // Clean up any previous state
        let _ = run_cmd(&["screen", "record", "stop"]);
        std::thread::sleep(std::time::Duration::from_millis(500));

        let tmp = std::env::temp_dir().join("xctrl_e2e_recording.mp4");
        let _ = std::fs::remove_file(&tmp);

        // Start recording
        let output = run_cmd(&[
            "screen",
            "record",
            "start",
            "--output",
            tmp.to_str().unwrap(),
        ]);
        assert!(
            output.status.success(),
            "record start should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Wait a bit for recording to begin
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Check status
        let output = run_cmd(&["screen", "record", "status", "--json"]);
        assert!(
            output.status.success(),
            "record status should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("status should be valid JSON");
        assert_eq!(
            json["recording"], true,
            "recording should be true while recording"
        );

        // Wait a bit more to ensure some frames are captured
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Stop recording
        let output = run_cmd(&["screen", "record", "stop"]);
        assert!(
            output.status.success(),
            "record stop should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Wait for file to be finalized
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Verify output file exists and has content
        assert!(tmp.exists(), "recording output file should exist");
        let file_size = std::fs::metadata(&tmp).unwrap().len();
        assert!(
            file_size > 0,
            "recording output file should be non-empty, got {} bytes",
            file_size
        );

        // Verify status shows not recording anymore
        let output = run_cmd(&["screen", "record", "status", "--json"]);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
        assert_eq!(
            json["recording"], false,
            "recording should be false after stop"
        );

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn e2e_screen_record_cold_status() {
        if should_skip() {
            return;
        }
        let _lock = SystemLock::acquire("screen_record");

        // Make sure nothing is recording
        let _ = run_cmd(&["screen", "record", "stop"]);
        std::thread::sleep(std::time::Duration::from_millis(500));

        let output = run_cmd(&["screen", "record", "status", "--json"]);
        assert!(
            output.status.success(),
            "cold status should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("cold status should be valid JSON");
        assert_eq!(
            json["recording"], false,
            "recording should be false when not recording"
        );
    }
}
