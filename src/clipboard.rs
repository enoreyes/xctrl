use crate::cli::ClipboardAction;
use crate::error::{exit_with_error, XctrlError};
use crate::output::print_success;
use serde::Serialize;

/// JSON output for the `clipboard get` command.
#[derive(Serialize)]
pub struct ClipboardText {
    pub text: String,
}

/// Internal argument used to signal a daemonized clipboard-holding process.
/// This should never be passed by users directly.
#[cfg(target_os = "linux")]
const DAEMON_ARG: &str = "__clipboard_daemon";

/// Create a new arboard Clipboard instance, exiting with a clear error if it fails.
fn create_clipboard(json: bool) -> arboard::Clipboard {
    match arboard::Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("Failed to access clipboard: {e}"),
                "Ensure a display server is running (e.g., start Xvfb with: Xvfb :99 -screen 0 1920x1080x24 -ac &, then export DISPLAY=:99)",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

/// On Linux, clipboard contents are owned by the setting process.
/// When that process exits, the clipboard is cleared (unless a clipboard manager
/// is running). To work around this, we spawn a background daemon process
/// that holds the clipboard content using arboard's `SetExtLinux::wait()`,
/// which blocks until another process overwrites the clipboard.
#[cfg(target_os = "linux")]
fn set_clipboard_text(text: &str, json: bool) {
    use std::process::{Command, Stdio};

    // Spawn a daemonized child that holds the clipboard
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            let err = XctrlError::new(format!("Failed to get current executable path: {e}"));
            exit_with_error(&err, json, 1);
        }
    };

    match Command::new(exe)
        .arg(DAEMON_ARG)
        .arg(text)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_child) => {
            // Give the daemon a moment to acquire the clipboard
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Err(e) => {
            let err = XctrlError::new(format!("Failed to spawn clipboard daemon: {e}"));
            exit_with_error(&err, json, 1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn set_clipboard_text(text: &str, json: bool) {
    let mut clipboard = create_clipboard(json);
    if let Err(e) = clipboard.set_text(text) {
        let err = XctrlError::new(format!("Failed to set clipboard text: {e}"));
        exit_with_error(&err, json, 1);
    }
}

/// Entry point for the daemonized clipboard-holding process (Linux only).
/// This is called when the binary is re-invoked with `DAEMON_ARG`.
#[cfg(target_os = "linux")]
pub fn run_clipboard_daemon(text: &str) {
    use arboard::SetExtLinux;

    // This will block until another process overwrites the clipboard
    if let Err(e) =
        arboard::Clipboard::new().and_then(|mut c| c.set().wait().text(text.to_string()))
    {
        // We're a background daemon; just exit silently on error
        eprintln!("clipboard daemon error: {e}");
    }
}

/// Check if the current process was invoked as a clipboard daemon.
/// Returns Some(text) if it was, None otherwise.
#[cfg(target_os = "linux")]
pub fn check_daemon_args(args: &[String]) -> Option<String> {
    if args.len() >= 2 && args[1] == DAEMON_ARG {
        Some(args.get(2).cloned().unwrap_or_default())
    } else {
        None
    }
}

pub fn handle_clipboard(action: ClipboardAction, json: bool) {
    match action {
        ClipboardAction::Set { text } => {
            set_clipboard_text(&text, json);
        }
        ClipboardAction::Get => {
            let mut clipboard = create_clipboard(json);
            match clipboard.get_text() {
                Ok(text) => {
                    let data = ClipboardText { text: text.clone() };
                    print_success(&data, &text, json);
                }
                Err(e) => {
                    let err_str = e.to_string();
                    // If clipboard is empty, return empty string rather than an error
                    if err_str.contains("mime type")
                        || err_str.contains("empty")
                        || err_str.contains("no text")
                        || err_str.contains("ContentNotAvailable")
                    {
                        let data = ClipboardText {
                            text: String::new(),
                        };
                        print_success(&data, "", json);
                    } else {
                        let err = XctrlError::new(format!("Failed to get clipboard text: {e}"));
                        exit_with_error(&err, json, 1);
                    }
                }
            }
        }
        ClipboardAction::Clear => {
            let mut clipboard = create_clipboard(json);
            if let Err(e) = clipboard.clear() {
                let err = XctrlError::new(format!("Failed to clear clipboard: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_text_json_serialization() {
        let data = ClipboardText {
            text: "hello world".to_string(),
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["text"], "hello world");
    }

    #[test]
    fn test_clipboard_text_json_empty() {
        let data = ClipboardText {
            text: String::new(),
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["text"], "");
    }

    #[test]
    fn test_clipboard_text_json_special_chars() {
        let data = ClipboardText {
            text: "hello \"world\" & <test>".to_string(),
        };
        let json_str = serde_json::to_string(&data).unwrap();
        // Verify it's valid JSON by re-parsing
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["text"], "hello \"world\" & <test>");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_check_daemon_args_with_daemon_flag() {
        let args = vec![
            "xctrl".to_string(),
            DAEMON_ARG.to_string(),
            "hello".to_string(),
        ];
        let result = check_daemon_args(&args);
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_check_daemon_args_without_daemon_flag() {
        let args = vec![
            "xctrl".to_string(),
            "clipboard".to_string(),
            "set".to_string(),
        ];
        let result = check_daemon_args(&args);
        assert_eq!(result, None);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_check_daemon_args_empty() {
        let args = vec!["xctrl".to_string()];
        let result = check_daemon_args(&args);
        assert_eq!(result, None);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_check_daemon_args_daemon_without_text() {
        let args = vec!["xctrl".to_string(), DAEMON_ARG.to_string()];
        let result = check_daemon_args(&args);
        assert_eq!(result, Some(String::new()));
    }
}
