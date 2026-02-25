use crate::cli::WindowAction;
use crate::error::{exit_with_error, XctrlError};
use serde::Serialize;

// ── Output structs ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WindowEntry {
    pub title: String,
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub pid: u32,
}

// ── Entrypoint ──────────────────────────────────────────────────────────────

pub fn handle_window(action: WindowAction, json: bool) {
    match action {
        WindowAction::List => handle_list(json),
        WindowAction::Focus { title, id } => {
            let target = resolve_target(title, id, json);
            handle_focus(&target, json);
        }
        WindowAction::Resize {
            title,
            id,
            width,
            height,
        } => {
            let target = resolve_target(title, id, json);
            handle_resize(&target, width, height, json);
        }
        WindowAction::Move { title, id, x, y } => {
            let target = resolve_target(title, id, json);
            handle_move(&target, x, y, json);
        }
        WindowAction::Minimize { title, id } => {
            let target = resolve_target(title, id, json);
            handle_minimize(&target, json);
        }
        WindowAction::Maximize { title, id } => {
            let target = resolve_target(title, id, json);
            handle_maximize(&target, json);
        }
        WindowAction::Fullscreen { title, id } => {
            let target = resolve_target(title, id, json);
            handle_fullscreen(&target, json);
        }
    }
}

// ── Target resolution ───────────────────────────────────────────────────────

enum WindowTarget {
    Title(String),
    Id(String),
}

fn resolve_target(title: Option<String>, id: Option<String>, json: bool) -> WindowTarget {
    match (title, id) {
        (Some(t), _) => WindowTarget::Title(t),
        (_, Some(i)) => WindowTarget::Id(i),
        (None, None) => {
            let err = XctrlError::with_hint(
                "either --title or --id is required",
                "Specify a window with --title <name> or --id <window_id>.",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

// ── Linux implementation ────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{self, Atom, AtomEnum, ConnectionExt, Window};
    use x11rb::rust_connection::RustConnection;

    struct X11Conn {
        conn: RustConnection,
        root: Window,
    }

    impl X11Conn {
        fn new() -> Result<Self, String> {
            let (conn, screen_num) =
                RustConnection::connect(None).map_err(|e| format!("X11 connect failed: {e}"))?;
            let root = conn.setup().roots[screen_num].root;
            Ok(Self { conn, root })
        }

        fn intern_atom(&self, name: &str) -> Result<Atom, String> {
            self.conn
                .intern_atom(false, name.as_bytes())
                .map_err(|e| format!("intern_atom request failed: {e}"))?
                .reply()
                .map(|r| r.atom)
                .map_err(|e| format!("intern_atom reply failed: {e}"))
        }

        fn get_property_u32(&self, window: Window, atom: Atom) -> Result<Vec<u32>, String> {
            let reply = self
                .conn
                .get_property(false, window, atom, AtomEnum::ANY, 0, 1024)
                .map_err(|e| format!("get_property request failed: {e}"))?
                .reply()
                .map_err(|e| format!("get_property reply failed: {e}"))?;
            // Value is returned as bytes in native endianness; convert to u32
            if reply.format == 32 {
                Ok(reply
                    .value32()
                    .map(|iter| iter.collect())
                    .unwrap_or_default())
            } else {
                Ok(vec![])
            }
        }

        fn get_window_title(&self, window: Window) -> String {
            // Try _NET_WM_NAME first (UTF-8), fall back to WM_NAME
            if let Ok(title) = self.get_net_wm_name(window) {
                if !title.is_empty() {
                    return title;
                }
            }
            if let Ok(title) = self.get_wm_name(window) {
                if !title.is_empty() {
                    return title;
                }
            }
            String::new()
        }

        fn get_net_wm_name(&self, window: Window) -> Result<String, String> {
            let atom = self.intern_atom("_NET_WM_NAME")?;
            let utf8_atom = self.intern_atom("UTF8_STRING")?;
            let reply = self
                .conn
                .get_property(false, window, atom, utf8_atom, 0, 1024)
                .map_err(|e| format!("get_property failed: {e}"))?
                .reply()
                .map_err(|e| format!("get_property reply failed: {e}"))?;
            Ok(String::from_utf8_lossy(&reply.value).to_string())
        }

        fn get_wm_name(&self, window: Window) -> Result<String, String> {
            let reply = self
                .conn
                .get_property(
                    false,
                    window,
                    Atom::from(AtomEnum::WM_NAME),
                    Atom::from(AtomEnum::STRING),
                    0,
                    1024,
                )
                .map_err(|e| format!("get_property failed: {e}"))?
                .reply()
                .map_err(|e| format!("get_property reply failed: {e}"))?;
            Ok(String::from_utf8_lossy(&reply.value).to_string())
        }

        fn get_window_pid(&self, window: Window) -> u32 {
            if let Ok(atom) = self.intern_atom("_NET_WM_PID") {
                if let Ok(vals) = self.get_property_u32(window, atom) {
                    if let Some(&pid) = vals.first() {
                        return pid;
                    }
                }
            }
            0
        }

        fn get_geometry(&self, window: Window) -> (i32, i32, i32, i32) {
            // Use translate_coordinates to get absolute position, then get_geometry for size.
            let geom = match self
                .conn
                .get_geometry(window)
                .ok()
                .and_then(|cookie| cookie.reply().ok())
            {
                Some(g) => g,
                None => return (0, 0, 0, 0),
            };

            let w = geom.width as i32;
            let h = geom.height as i32;

            // translate to root coordinates
            let (x, y) = self
                .conn
                .translate_coordinates(window, self.root, 0, 0)
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|trans| (trans.dst_x as i32, trans.dst_y as i32))
                .unwrap_or((0, 0));

            (x, y, w, h)
        }

        fn list_windows(&self) -> Result<Vec<WindowEntry>, String> {
            let client_list_atom = self.intern_atom("_NET_CLIENT_LIST")?;
            let window_ids = self.get_property_u32(self.root, client_list_atom)?;

            let mut entries = Vec::new();
            for &wid in &window_ids {
                let title = self.get_window_title(wid);
                let pid = self.get_window_pid(wid);
                let (x, y, w, h) = self.get_geometry(wid);
                entries.push(WindowEntry {
                    title,
                    id: wid.to_string(),
                    x,
                    y,
                    width: w,
                    height: h,
                    pid,
                });
            }
            Ok(entries)
        }

        fn find_window(&self, target: &WindowTarget) -> Result<Window, String> {
            match target {
                WindowTarget::Id(id_str) => {
                    let wid: u32 = id_str
                        .parse()
                        .map_err(|_| format!("invalid window id: {id_str}"))?;
                    // Verify window exists by trying to get its attributes
                    self.conn
                        .get_window_attributes(wid)
                        .map_err(|e| format!("window not found: {e}"))?
                        .reply()
                        .map_err(|_| "window not found".to_string())?;
                    Ok(wid)
                }
                WindowTarget::Title(name) => {
                    let client_list_atom = self.intern_atom("_NET_CLIENT_LIST")?;
                    let window_ids = self.get_property_u32(self.root, client_list_atom)?;
                    for &wid in &window_ids {
                        let title = self.get_window_title(wid);
                        if title.contains(name.as_str()) {
                            return Ok(wid);
                        }
                    }
                    Err(format!(
                        "window not found: no window with title containing '{name}'"
                    ))
                }
            }
        }

        fn send_ewmh_message(
            &self,
            window: Window,
            message_type: Atom,
            data: [u32; 5],
        ) -> Result<(), String> {
            let event = xproto::ClientMessageEvent::new(
                32,
                window,
                message_type,
                xproto::ClientMessageData::from([data[0], data[1], data[2], data[3], data[4]]),
            );
            self.conn
                .send_event(
                    false,
                    self.root,
                    xproto::EventMask::SUBSTRUCTURE_REDIRECT
                        | xproto::EventMask::SUBSTRUCTURE_NOTIFY,
                    event,
                )
                .map_err(|e| format!("send_event failed: {e}"))?;
            self.conn
                .flush()
                .map_err(|e| format!("flush failed: {e}"))?;
            Ok(())
        }

        fn focus_window(&self, window: Window) -> Result<(), String> {
            let active_atom = self.intern_atom("_NET_ACTIVE_WINDOW")?;
            // Source indication = 2 (pager), timestamp = 0, requestor = 0
            self.send_ewmh_message(window, active_atom, [2, 0, 0, 0, 0])
        }

        fn resize_window(&self, window: Window, width: u32, height: u32) -> Result<(), String> {
            let aux = xproto::ConfigureWindowAux::new()
                .width(width)
                .height(height);
            self.conn
                .configure_window(window, &aux)
                .map_err(|e| format!("configure_window failed: {e}"))?;
            self.conn
                .flush()
                .map_err(|e| format!("flush failed: {e}"))?;
            Ok(())
        }

        fn move_window(&self, window: Window, x: i32, y: i32) -> Result<(), String> {
            let aux = xproto::ConfigureWindowAux::new().x(x).y(y);
            self.conn
                .configure_window(window, &aux)
                .map_err(|e| format!("configure_window failed: {e}"))?;
            self.conn
                .flush()
                .map_err(|e| format!("flush failed: {e}"))?;
            Ok(())
        }

        fn iconify_window(&self, window: Window) -> Result<(), String> {
            // Use WM_CHANGE_STATE to request iconic (minimized) state
            let wm_change_state = self.intern_atom("WM_CHANGE_STATE")?;
            // IconicState = 3
            self.send_ewmh_message(window, wm_change_state, [3, 0, 0, 0, 0])
        }

        fn set_wm_state(
            &self,
            window: Window,
            action: u32,
            state_atom: Atom,
        ) -> Result<(), String> {
            let wm_state_atom = self.intern_atom("_NET_WM_STATE")?;
            self.send_ewmh_message(
                window,
                wm_state_atom,
                [action, state_atom, 0, 1, 0], // source = 1 (application)
            )
        }

        fn maximize_window(&self, window: Window) -> Result<(), String> {
            let horz = self.intern_atom("_NET_WM_STATE_MAXIMIZED_HORZ")?;
            let vert = self.intern_atom("_NET_WM_STATE_MAXIMIZED_VERT")?;
            let wm_state_atom = self.intern_atom("_NET_WM_STATE")?;
            // _NET_WM_STATE action: 1 = add
            self.send_ewmh_message(window, wm_state_atom, [1, horz, vert, 1, 0])
        }

        fn fullscreen_window(&self, window: Window) -> Result<(), String> {
            let fullscreen = self.intern_atom("_NET_WM_STATE_FULLSCREEN")?;
            // action = 1 (add)
            self.set_wm_state(window, 1, fullscreen)
        }
    }

    pub fn list_windows() -> Result<Vec<WindowEntry>, String> {
        let xconn = X11Conn::new()?;
        xconn.list_windows()
    }

    pub fn focus_window(target: &WindowTarget) -> Result<(), String> {
        let xconn = X11Conn::new()?;
        let window = xconn.find_window(target)?;
        xconn.focus_window(window)
    }

    pub fn resize_window(target: &WindowTarget, width: u32, height: u32) -> Result<(), String> {
        let xconn = X11Conn::new()?;
        let window = xconn.find_window(target)?;
        xconn.resize_window(window, width, height)
    }

    pub fn move_window(target: &WindowTarget, x: i32, y: i32) -> Result<(), String> {
        let xconn = X11Conn::new()?;
        let window = xconn.find_window(target)?;
        xconn.move_window(window, x, y)
    }

    pub fn minimize_window(target: &WindowTarget) -> Result<(), String> {
        let xconn = X11Conn::new()?;
        let window = xconn.find_window(target)?;
        xconn.iconify_window(window)
    }

    pub fn maximize_window(target: &WindowTarget) -> Result<(), String> {
        let xconn = X11Conn::new()?;
        let window = xconn.find_window(target)?;
        xconn.maximize_window(window)
    }

    pub fn fullscreen_window(target: &WindowTarget) -> Result<(), String> {
        let xconn = X11Conn::new()?;
        let window = xconn.find_window(target)?;
        xconn.fullscreen_window(window)
    }
}

// ── macOS stub implementation ───────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    pub fn list_windows() -> Result<Vec<WindowEntry>, String> {
        Err("window list is not yet implemented on macOS".to_string())
    }

    pub fn focus_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window focus is not yet implemented on macOS".to_string())
    }

    pub fn resize_window(_target: &WindowTarget, _width: u32, _height: u32) -> Result<(), String> {
        Err("window resize is not yet implemented on macOS".to_string())
    }

    pub fn move_window(_target: &WindowTarget, _x: i32, _y: i32) -> Result<(), String> {
        Err("window move is not yet implemented on macOS".to_string())
    }

    pub fn minimize_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window minimize is not yet implemented on macOS".to_string())
    }

    pub fn maximize_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window maximize is not yet implemented on macOS".to_string())
    }

    pub fn fullscreen_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window fullscreen is not yet implemented on macOS".to_string())
    }
}

// ── Windows stub implementation ─────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    pub fn list_windows() -> Result<Vec<WindowEntry>, String> {
        Err("window list is not yet implemented on Windows".to_string())
    }

    pub fn focus_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window focus is not yet implemented on Windows".to_string())
    }

    pub fn resize_window(_target: &WindowTarget, _width: u32, _height: u32) -> Result<(), String> {
        Err("window resize is not yet implemented on Windows".to_string())
    }

    pub fn move_window(_target: &WindowTarget, _x: i32, _y: i32) -> Result<(), String> {
        Err("window move is not yet implemented on Windows".to_string())
    }

    pub fn minimize_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window minimize is not yet implemented on Windows".to_string())
    }

    pub fn maximize_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window maximize is not yet implemented on Windows".to_string())
    }

    pub fn fullscreen_window(_target: &WindowTarget) -> Result<(), String> {
        Err("window fullscreen is not yet implemented on Windows".to_string())
    }
}

// ── Handlers ────────────────────────────────────────────────────────────────

fn handle_list(json: bool) {
    match platform::list_windows() {
        Ok(windows) => {
            if json {
                let json_str = serde_json::to_string_pretty(&windows).unwrap_or_else(|e| {
                    format!("{{\"error\": \"JSON serialization failed: {e}\"}}")
                });
                println!("{json_str}");
            } else if windows.is_empty() {
                println!("No windows found.");
            } else {
                for w in &windows {
                    println!(
                        "[{}] \"{}\" ({}x{} at {},{}) pid={}",
                        w.id, w.title, w.width, w.height, w.x, w.y, w.pid
                    );
                }
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("failed to list windows: {e}"),
                "Ensure an X11 display server is running (DISPLAY must be set).",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

fn handle_focus(target: &WindowTarget, json: bool) {
    match platform::focus_window(target) {
        Ok(()) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Window focused.");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                e.clone(),
                "Check that the window exists with: xctrl window list",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

fn handle_resize(target: &WindowTarget, width: u32, height: u32, json: bool) {
    match platform::resize_window(target, width, height) {
        Ok(()) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Window resized to {width}x{height}.");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                e.clone(),
                "Check that the window exists with: xctrl window list",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

fn handle_move(target: &WindowTarget, x: i32, y: i32, json: bool) {
    match platform::move_window(target, x, y) {
        Ok(()) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Window moved to ({x}, {y}).");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                e.clone(),
                "Check that the window exists with: xctrl window list",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

fn handle_minimize(target: &WindowTarget, json: bool) {
    match platform::minimize_window(target) {
        Ok(()) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Window minimized.");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                e.clone(),
                "Check that the window exists with: xctrl window list",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

fn handle_maximize(target: &WindowTarget, json: bool) {
    match platform::maximize_window(target) {
        Ok(()) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Window maximized.");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                e.clone(),
                "Check that the window exists with: xctrl window list",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

fn handle_fullscreen(target: &WindowTarget, json: bool) {
    match platform::fullscreen_window(target) {
        Ok(()) => {
            if json {
                println!("{{\"status\":\"ok\"}}");
            } else {
                println!("Window set to fullscreen.");
            }
        }
        Err(e) => {
            let err = XctrlError::with_hint(
                e.clone(),
                "Check that the window exists with: xctrl window list",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_entry_serialization() {
        let entry = WindowEntry {
            title: "Test Window".to_string(),
            id: "12345".to_string(),
            x: 100,
            y: 200,
            width: 800,
            height: 600,
            pid: 1234,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["title"], "Test Window");
        assert_eq!(json["id"], "12345");
        assert_eq!(json["x"], 100);
        assert_eq!(json["y"], 200);
        assert_eq!(json["width"], 800);
        assert_eq!(json["height"], 600);
        assert_eq!(json["pid"], 1234);
    }

    #[test]
    fn test_window_entry_list_serialization() {
        let entries = vec![
            WindowEntry {
                title: "Window A".to_string(),
                id: "1".to_string(),
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                pid: 100,
            },
            WindowEntry {
                title: "Window B".to_string(),
                id: "2".to_string(),
                x: 50,
                y: 50,
                width: 640,
                height: 480,
                pid: 200,
            },
        ];
        let json_str = serde_json::to_string(&entries).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["title"], "Window A");
        assert_eq!(parsed[1]["title"], "Window B");
    }

    #[test]
    fn test_window_entry_snake_case_keys() {
        let entry = WindowEntry {
            title: "Test".to_string(),
            id: "1".to_string(),
            x: 0,
            y: 0,
            width: 100,
            height: 100,
            pid: 1,
        };
        let json_str = serde_json::to_string(&entry).unwrap();
        // All keys should be snake_case
        assert!(json_str.contains("\"title\""));
        assert!(json_str.contains("\"id\""));
        assert!(json_str.contains("\"x\""));
        assert!(json_str.contains("\"y\""));
        assert!(json_str.contains("\"width\""));
        assert!(json_str.contains("\"height\""));
        assert!(json_str.contains("\"pid\""));
        // No camelCase keys
        assert!(!json_str.contains("\"windowId\""));
        assert!(!json_str.contains("\"processId\""));
    }
}
