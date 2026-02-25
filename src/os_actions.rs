use crate::cli::OsAction;
use crate::error::{exit_with_error, XctrlError};
use serde::Serialize;

// ── Output structs ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FrontmostAppInfo {
    pub name: String,
    pub pid: u32,
}

#[derive(Debug, Serialize)]
pub struct AppEntry {
    pub name: String,
    pub pid: u32,
}

// ── Entrypoint ──────────────────────────────────────────────────────────────

pub fn handle_os(action: OsAction, json: bool) {
    match action {
        OsAction::OpenUrl { url } => handle_open_url(&url, json),
        OsAction::OpenApp { name } => handle_open_app(&name, json),
        OsAction::Notify { title, body } => handle_notify(&title, &body, json),
        OsAction::FrontmostApp => handle_frontmost_app(json),
        OsAction::ListApps => handle_list_apps(json),
    }
}

// ── open-url ────────────────────────────────────────────────────────────────

fn handle_open_url(url: &str, json: bool) {
    if let Err(e) = open::that(url) {
        let err = XctrlError::with_hint(
            format!("failed to open URL '{url}': {e}"),
            "Ensure a default browser is configured on this system.",
        );
        exit_with_error(&err, json, 1);
    }
    if json {
        println!("{{\"status\":\"ok\"}}");
    } else {
        println!("Opened URL: {url}");
    }
}

// ── open-app ────────────────────────────────────────────────────────────────

fn handle_open_app(name: &str, json: bool) {
    match platform::open_app(name) {
        Ok(()) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Launched application: {name}");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("failed to open application '{name}': {e}"),
                "Check that the application is installed and on your PATH.",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

// ── notify ──────────────────────────────────────────────────────────────────

fn handle_notify(title: &str, body: &str, json: bool) {
    match notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show()
    {
        Ok(_) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Notification sent.");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("failed to send notification: {e}"),
                "Ensure a notification daemon (e.g., dunst, mako) is running, or D-Bus is available.",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

// ── frontmost-app ───────────────────────────────────────────────────────────

fn handle_frontmost_app(json: bool) {
    match platform::frontmost_app() {
        Ok(info) => {
            if json {
                let json_str = serde_json::to_string_pretty(&info).unwrap_or_else(|e| {
                    format!("{{\"error\": \"JSON serialization failed: {e}\"}}")
                });
                println!("{json_str}");
            } else {
                println!("{}", info.name);
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("failed to get frontmost application: {e}"),
                "Ensure an X11 display server is running (DISPLAY must be set).",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

// ── list-apps ───────────────────────────────────────────────────────────────

fn handle_list_apps(json: bool) {
    match platform::list_apps() {
        Ok(apps) => {
            if json {
                let json_str = serde_json::to_string_pretty(&apps).unwrap_or_else(|e| {
                    format!("{{\"error\": \"JSON serialization failed: {e}\"}}")
                });
                println!("{json_str}");
            } else if apps.is_empty() {
                println!("No running applications found.");
            } else {
                for app in &apps {
                    println!("{} (pid={})", app.name, app.pid);
                }
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("failed to list applications: {e}"),
                "Ensure an X11 display server is running (DISPLAY must be set), or check /proc permissions.",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

// ── Linux implementation ────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window};
    use x11rb::rust_connection::RustConnection;

    pub fn open_app(name: &str) -> Result<(), String> {
        // Try xdg-open first (works for .desktop entries), then direct exec
        let result = Command::new("xdg-open")
            .arg(name)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                // Wait briefly to see if it fails immediately
                match child.wait() {
                    Ok(status) if status.success() => Ok(()),
                    Ok(status) => {
                        // xdg-open failed, try direct exec
                        try_direct_exec(name).map_err(|_| {
                            format!(
                                "application not found (xdg-open exited with {})",
                                status.code().unwrap_or(-1)
                            )
                        })
                    }
                    Err(e) => Err(format!("failed to wait on xdg-open: {e}")),
                }
            }
            Err(_) => {
                // xdg-open not available, try direct exec
                try_direct_exec(name)
            }
        }
    }

    fn try_direct_exec(name: &str) -> Result<(), String> {
        // Try to find the executable on PATH via `which`
        let which_result = Command::new("which").arg(name).output();

        match which_result {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // Spawn it detached (don't wait)
                Command::new(&path)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .stdin(std::process::Stdio::null())
                    .spawn()
                    .map_err(|e| format!("failed to launch '{name}': {e}"))?;
                Ok(())
            }
            _ => Err(format!("application '{name}' not found on PATH")),
        }
    }

    pub fn frontmost_app() -> Result<FrontmostAppInfo, String> {
        let (conn, screen_num) =
            RustConnection::connect(None).map_err(|e| format!("X11 connect failed: {e}"))?;
        let root = conn.setup().roots[screen_num].root;

        // Get _NET_ACTIVE_WINDOW
        let active_atom = intern_atom(&conn, "_NET_ACTIVE_WINDOW")?;
        let reply = conn
            .get_property(false, root, active_atom, AtomEnum::ANY, 0, 1)
            .map_err(|e| format!("get_property request failed: {e}"))?
            .reply()
            .map_err(|e| format!("get_property reply failed: {e}"))?;

        let active_window = if reply.format == 32 {
            reply
                .value32()
                .and_then(|mut iter| iter.next())
                .unwrap_or(0)
        } else {
            0
        };

        if active_window == 0 {
            return Err("no active window found".to_string());
        }

        // Get PID from _NET_WM_PID
        let pid = get_window_pid(&conn, active_window)?;

        // Get process name from /proc/<pid>/comm
        let name = get_process_name(pid);

        Ok(FrontmostAppInfo { name, pid })
    }

    pub fn list_apps() -> Result<Vec<AppEntry>, String> {
        let (conn, screen_num) =
            RustConnection::connect(None).map_err(|e| format!("X11 connect failed: {e}"))?;
        let root = conn.setup().roots[screen_num].root;

        // Get _NET_CLIENT_LIST
        let client_list_atom = intern_atom(&conn, "_NET_CLIENT_LIST")?;
        let reply = conn
            .get_property(false, root, client_list_atom, AtomEnum::ANY, 0, 4096)
            .map_err(|e| format!("get_property request failed: {e}"))?
            .reply()
            .map_err(|e| format!("get_property reply failed: {e}"))?;

        let window_ids: Vec<u32> = if reply.format == 32 {
            reply
                .value32()
                .map(|iter| iter.collect())
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Collect unique apps by PID
        let mut seen_pids: HashMap<u32, bool> = HashMap::new();
        let mut apps = Vec::new();

        for &wid in &window_ids {
            if let Ok(pid) = get_window_pid(&conn, wid) {
                if pid > 0 && !seen_pids.contains_key(&pid) {
                    seen_pids.insert(pid, true);
                    let name = get_process_name(pid);
                    apps.push(AppEntry { name, pid });
                }
            }
        }

        // If no X11 windows found, fall back to /proc scanning for GUI processes
        if apps.is_empty() {
            apps = list_apps_from_proc();
        }

        Ok(apps)
    }

    fn intern_atom(conn: &RustConnection, name: &str) -> Result<Atom, String> {
        conn.intern_atom(false, name.as_bytes())
            .map_err(|e| format!("intern_atom request failed: {e}"))?
            .reply()
            .map(|r| r.atom)
            .map_err(|e| format!("intern_atom reply failed: {e}"))
    }

    fn get_window_pid(conn: &RustConnection, window: Window) -> Result<u32, String> {
        let pid_atom = intern_atom(conn, "_NET_WM_PID")?;
        let reply = conn
            .get_property(false, window, pid_atom, AtomEnum::ANY, 0, 1)
            .map_err(|e| format!("get_property request failed: {e}"))?
            .reply()
            .map_err(|e| format!("get_property reply failed: {e}"))?;

        if reply.format == 32 {
            Ok(reply
                .value32()
                .and_then(|mut iter| iter.next())
                .unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    fn get_process_name(pid: u32) -> String {
        // Read /proc/<pid>/comm for the process name
        let comm_path = format!("/proc/{pid}/comm");
        if let Ok(name) = fs::read_to_string(&comm_path) {
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }

        // Fall back to /proc/<pid>/cmdline
        let cmdline_path = format!("/proc/{pid}/cmdline");
        if let Ok(data) = fs::read(&cmdline_path) {
            // cmdline is null-separated; first element is the executable
            if let Some(first) = data.split(|&b| b == 0).next() {
                let cmd = String::from_utf8_lossy(first).to_string();
                if !cmd.is_empty() {
                    // Extract basename
                    return Path::new(&cmd)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or(cmd);
                }
            }
        }

        format!("pid-{pid}")
    }

    /// Fallback: scan /proc for processes that have a DISPLAY environment variable set,
    /// indicating they are GUI applications.
    fn list_apps_from_proc() -> Vec<AppEntry> {
        let mut apps = Vec::new();
        let proc_dir = match fs::read_dir("/proc") {
            Ok(d) => d,
            Err(_) => return apps,
        };

        for entry in proc_dir.flatten() {
            let fname = entry.file_name();
            let name_str = fname.to_string_lossy();
            // Only look at numeric directories (PIDs)
            if let Ok(pid) = name_str.parse::<u32>() {
                // Check if the process has a DISPLAY env var
                let environ_path = format!("/proc/{pid}/environ");
                if let Ok(data) = fs::read(&environ_path) {
                    let has_display = data
                        .split(|&b| b == 0)
                        .any(|entry| entry.starts_with(b"DISPLAY="));
                    if has_display {
                        let name = get_process_name(pid);
                        apps.push(AppEntry { name, pid });
                    }
                }
            }
        }

        apps
    }
}

// ── macOS stub implementation ───────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::process::Command;

    pub fn open_app(name: &str) -> Result<(), String> {
        let output = Command::new("open")
            .args(["-a", name])
            .output()
            .map_err(|e| format!("failed to run 'open -a': {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("application '{name}' not found: {}", stderr.trim()))
        }
    }

    pub fn frontmost_app() -> Result<FrontmostAppInfo, String> {
        let output = Command::new("osascript")
            .args([
                "-e",
                "tell application \"System Events\" to get {name, unix id} of first application process whose frontmost is true",
            ])
            .output()
            .map_err(|e| format!("osascript failed: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("failed to get frontmost app: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        // Output format: "AppName, 12345"
        let parts: Vec<&str> = trimmed.splitn(2, ", ").collect();
        let name = parts.first().unwrap_or(&"unknown").to_string();
        let pid: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        Ok(FrontmostAppInfo { name, pid })
    }

    pub fn list_apps() -> Result<Vec<AppEntry>, String> {
        let output = Command::new("osascript")
            .args([
                "-e",
                "tell application \"System Events\" to get {name, unix id} of every application process whose background only is false",
            ])
            .output()
            .map_err(|e| format!("osascript failed: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("failed to list apps: {}", stderr.trim()));
        }

        // Output format: "{App1, App2}, {123, 456}" or similar
        // This is a simplification; real parsing would be more complex
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        // Simple heuristic: split by comma and pair names with pids
        let mut apps = Vec::new();
        for line in trimmed.lines() {
            let parts: Vec<&str> = line.splitn(2, ", ").collect();
            if parts.len() == 2 {
                let name = parts[0].to_string();
                let pid: u32 = parts[1].parse().unwrap_or(0);
                apps.push(AppEntry { name, pid });
            }
        }
        Ok(apps)
    }
}

// ── Windows stub implementation ─────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::process::Command;

    pub fn open_app(name: &str) -> Result<(), String> {
        let output = Command::new("cmd")
            .args(["/C", "start", "", name])
            .output()
            .map_err(|e| format!("failed to run 'start': {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("application '{name}' not found: {}", stderr.trim()))
        }
    }

    pub fn frontmost_app() -> Result<FrontmostAppInfo, String> {
        Err("frontmost-app is not yet implemented on Windows".to_string())
    }

    pub fn list_apps() -> Result<Vec<AppEntry>, String> {
        Err("list-apps is not yet implemented on Windows".to_string())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmost_app_info_serialization() {
        let info = FrontmostAppInfo {
            name: "firefox".to_string(),
            pid: 1234,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["name"], "firefox");
        assert_eq!(json["pid"], 1234);
    }

    #[test]
    fn test_frontmost_app_info_snake_case_keys() {
        let info = FrontmostAppInfo {
            name: "test".to_string(),
            pid: 1,
        };
        let json_str = serde_json::to_string(&info).unwrap();
        assert!(json_str.contains("\"name\""));
        assert!(json_str.contains("\"pid\""));
        // No camelCase
        assert!(!json_str.contains("\"processId\""));
        assert!(!json_str.contains("\"appName\""));
    }

    #[test]
    fn test_app_entry_serialization() {
        let entry = AppEntry {
            name: "vim".to_string(),
            pid: 5678,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["name"], "vim");
        assert_eq!(json["pid"], 5678);
    }

    #[test]
    fn test_app_entry_list_serialization() {
        let entries = vec![
            AppEntry {
                name: "firefox".to_string(),
                pid: 100,
            },
            AppEntry {
                name: "code".to_string(),
                pid: 200,
            },
        ];
        let json_str = serde_json::to_string(&entries).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["name"], "firefox");
        assert_eq!(parsed[1]["name"], "code");
    }

    #[test]
    fn test_app_entry_snake_case_keys() {
        let entry = AppEntry {
            name: "test".to_string(),
            pid: 1,
        };
        let json_str = serde_json::to_string(&entry).unwrap();
        assert!(json_str.contains("\"name\""));
        assert!(json_str.contains("\"pid\""));
    }

    #[test]
    fn test_app_entry_empty_list() {
        let entries: Vec<AppEntry> = vec![];
        let json_str = serde_json::to_string(&entries).unwrap();
        assert_eq!(json_str, "[]");
    }
}
