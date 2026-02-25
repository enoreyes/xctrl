//! Display e2e tests — Linux and macOS (Linux needs Xvfb).

#[cfg(not(target_os = "windows"))]
mod display_tests {
    use crate::{has_display, xctrl_bin};
    use std::process::Command;

    fn should_skip() -> bool {
        if cfg!(target_os = "linux") && !has_display() {
            eprintln!("SKIP: Linux requires DISPLAY for display tests");
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
    fn e2e_display_screenshot_creates_file() {
        if should_skip() {
            return;
        }

        let tmp = std::env::temp_dir().join("xctrl_e2e_screenshot.png");
        // Clean up any previous test artifact
        let _ = std::fs::remove_file(&tmp);

        let output = run_cmd(&["display", "screenshot", "--output", tmp.to_str().unwrap()]);
        assert!(
            output.status.success(),
            "display screenshot should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(tmp.exists(), "screenshot file should exist");
        assert!(
            std::fs::metadata(&tmp).unwrap().len() > 0,
            "screenshot file should not be empty"
        );

        // Verify it starts with PNG magic bytes
        let bytes = std::fs::read(&tmp).unwrap();
        assert!(
            bytes.len() >= 8 && bytes[0..4] == [0x89, 0x50, 0x4E, 0x47],
            "file should be a valid PNG (starts with PNG magic bytes)"
        );

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn e2e_display_info_returns_data() {
        if should_skip() {
            return;
        }

        let output = run_cmd(&["display", "info", "--json"]);
        assert!(
            output.status.success(),
            "display info should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("display info should return valid JSON");

        // Verify the JSON has the expected fields
        assert!(
            json.get("width").is_some(),
            "display info should contain 'width'"
        );
        assert!(
            json.get("height").is_some(),
            "display info should contain 'height'"
        );
        assert!(
            json.get("scale_factor").is_some(),
            "display info should contain 'scale_factor'"
        );

        // Width and height should be positive numbers
        assert!(
            json["width"].as_u64().unwrap_or(0) > 0,
            "width should be > 0"
        );
        assert!(
            json["height"].as_u64().unwrap_or(0) > 0,
            "height should be > 0"
        );
    }

    #[test]
    fn e2e_display_list_returns_monitors() {
        if should_skip() {
            return;
        }

        let output = run_cmd(&["display", "list", "--json"]);
        assert!(
            output.status.success(),
            "display list should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("display list should return valid JSON");

        assert!(json.is_array(), "display list should return an array");
        let arr = json.as_array().unwrap();
        assert!(
            !arr.is_empty(),
            "display list should have at least one monitor"
        );

        // Verify first monitor has expected fields
        let monitor = &arr[0];
        assert!(
            monitor.get("name").is_some(),
            "monitor should have a 'name' field"
        );
        assert!(
            monitor.get("width").is_some(),
            "monitor should have a 'width' field"
        );
        assert!(
            monitor.get("height").is_some(),
            "monitor should have a 'height' field"
        );
    }

    #[test]
    fn e2e_display_screenshot_region() {
        if should_skip() {
            return;
        }

        let tmp = std::env::temp_dir().join("xctrl_e2e_region_screenshot.png");
        let _ = std::fs::remove_file(&tmp);

        let output = run_cmd(&[
            "display",
            "screenshot",
            "--output",
            tmp.to_str().unwrap(),
            "--x",
            "0",
            "--y",
            "0",
            "--width",
            "100",
            "--height",
            "100",
        ]);
        assert!(
            output.status.success(),
            "display screenshot region should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(tmp.exists(), "region screenshot file should exist");
        assert!(
            std::fs::metadata(&tmp).unwrap().len() > 0,
            "region screenshot file should not be empty"
        );

        let _ = std::fs::remove_file(&tmp);
    }
}
