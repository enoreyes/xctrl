use crate::cli::MouseAction;
use crate::error::{exit_with_error, XctrlError};
use crate::output::print_success;
use enigo::{Axis, Button, Direction, Enigo, Mouse, Settings};
use serde::Serialize;

/// JSON output for the `mouse position` command.
#[derive(Serialize)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

/// Create a new Enigo instance, exiting with a clear error if it fails.
fn create_enigo(json: bool) -> Enigo {
    match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("Failed to initialize input controller: {e}"),
                "Ensure a display server is running (e.g., start Xvfb with: Xvfb :99 -screen 0 1920x1080x24 -ac &, then export DISPLAY=:99)",
            );
            exit_with_error(&err, json, 1);
        }
    }
}

// ---- Platform-specific mouse move / position using x11rb ----

#[cfg(target_os = "linux")]
mod platform {
    use x11rb::connection::Connection;
    use x11rb::protocol::randr::ConnectionExt as RandrExt;
    use x11rb::protocol::xproto::ConnectionExt;
    use x11rb::wrapper::ConnectionExt as WrapperExt;

    /// Get screen dimensions (width, height) from the X11 display.
    fn get_screen_size() -> Result<(i32, i32), String> {
        let (conn, screen_num) = x11rb::connect(None).map_err(|e| format!("{e}"))?;
        let screen = &conn.setup().roots[screen_num];

        // Try RandR first for accurate mode resolution
        if let Ok(cookie) = conn.randr_get_screen_resources(screen.root) {
            if let Ok(reply) = cookie.reply() {
                if let Some(mode) = reply.modes.first() {
                    return Ok((mode.width as i32, mode.height as i32));
                }
            }
        }

        // Fallback to screen dimensions
        Ok((
            screen.width_in_pixels as i32,
            screen.height_in_pixels as i32,
        ))
    }

    /// Clamp coordinates to the display bounds.
    pub fn clamp_coords(x: i32, y: i32) -> (i32, i32) {
        if let Ok((w, h)) = get_screen_size() {
            let cx = x.max(0).min(w.saturating_sub(1));
            let cy = y.max(0).min(h.saturating_sub(1));
            (cx, cy)
        } else {
            (x.max(0), y.max(0))
        }
    }

    /// Move the mouse cursor using XWarpPointer (works reliably with Xvfb).
    pub fn warp_mouse(x: i32, y: i32) -> Result<(), String> {
        let (conn, screen_num) = x11rb::connect(None).map_err(|e| format!("{e}"))?;
        let screen = &conn.setup().roots[screen_num];
        conn.warp_pointer(
            x11rb::NONE, // src_window
            screen.root, // dst_window (root = absolute)
            0,           // src_x
            0,           // src_y
            0,           // src_width
            0,           // src_height
            x as i16,    // dst_x
            y as i16,    // dst_y
        )
        .map_err(|e| format!("{e}"))?;
        // sync() ensures the X server processes the warp before we close
        // the connection. Without this, the server may not have processed
        // the request before the connection drops.
        conn.sync().map_err(|e| format!("{e}"))?;
        Ok(())
    }

    /// Query the current cursor position using XQueryPointer.
    pub fn query_position() -> Result<(i32, i32), String> {
        let (conn, screen_num) = x11rb::connect(None).map_err(|e| format!("{e}"))?;
        let screen = &conn.setup().roots[screen_num];
        let reply = conn
            .query_pointer(screen.root)
            .map_err(|e| format!("{e}"))?
            .reply()
            .map_err(|e| format!("{e}"))?;
        Ok((reply.root_x as i32, reply.root_y as i32))
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use enigo::{Coordinate, Enigo, Mouse, Settings};

    pub fn clamp_coords(x: i32, y: i32) -> (i32, i32) {
        let enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(_) => return (x.max(0), y.max(0)),
        };
        if let Ok((w, h)) = enigo.main_display() {
            (
                x.max(0).min(w.saturating_sub(1)),
                y.max(0).min(h.saturating_sub(1)),
            )
        } else {
            (x.max(0), y.max(0))
        }
    }

    pub fn warp_mouse(x: i32, y: i32) -> Result<(), String> {
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("{e}"))?;
        enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| format!("{e}"))
    }

    pub fn query_position() -> Result<(i32, i32), String> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| format!("{e}"))?;
        enigo.location().map_err(|e| format!("{e}"))
    }
}

/// Move the mouse to an absolute position, clamping to display bounds.
fn do_move(x: i32, y: i32, json: bool) {
    let (cx, cy) = platform::clamp_coords(x, y);
    if let Err(e) = platform::warp_mouse(cx, cy) {
        let err = XctrlError::new(format!("Failed to move mouse: {e}"));
        exit_with_error(&err, json, 1);
    }
}

/// Optionally move to (x, y) if both are provided, then perform a button action.
fn move_and_click(
    enigo: &mut Enigo,
    x: Option<i32>,
    y: Option<i32>,
    button: Button,
    direction: Direction,
    json: bool,
) {
    if let (Some(mx), Some(my)) = (x, y) {
        do_move(mx, my, json);
    }
    if let Err(e) = enigo.button(button, direction) {
        let err = XctrlError::new(format!("Failed to perform click: {e}"));
        exit_with_error(&err, json, 1);
    }
}

pub fn handle_mouse(action: MouseAction, json: bool) {
    match action {
        MouseAction::Move { x, y } => {
            do_move(x, y, json);
        }
        MouseAction::Click { x, y } => {
            let mut enigo = create_enigo(json);
            move_and_click(&mut enigo, x, y, Button::Left, Direction::Click, json);
        }
        MouseAction::DoubleClick { x, y } => {
            let mut enigo = create_enigo(json);
            if let (Some(mx), Some(my)) = (x, y) {
                do_move(mx, my, json);
            }
            for _ in 0..2 {
                if let Err(e) = enigo.button(Button::Left, Direction::Click) {
                    let err = XctrlError::new(format!("Failed to perform double-click: {e}"));
                    exit_with_error(&err, json, 1);
                }
            }
        }
        MouseAction::RightClick { x, y } => {
            let mut enigo = create_enigo(json);
            move_and_click(&mut enigo, x, y, Button::Right, Direction::Click, json);
        }
        MouseAction::Scroll { amount } => {
            let mut enigo = create_enigo(json);
            // Feature spec: positive = up, negative = down
            // enigo: positive = down, negative = up
            // So we negate the amount.
            let enigo_amount = -amount;
            if let Err(e) = enigo.scroll(enigo_amount, Axis::Vertical) {
                let err = XctrlError::new(format!("Failed to scroll: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
        MouseAction::Drag {
            from_x,
            from_y,
            to_x,
            to_y,
        } => {
            let mut enigo = create_enigo(json);
            do_move(from_x, from_y, json);
            if let Err(e) = enigo.button(Button::Left, Direction::Press) {
                let err = XctrlError::new(format!("Failed to start drag: {e}"));
                exit_with_error(&err, json, 1);
            }
            do_move(to_x, to_y, json);
            if let Err(e) = enigo.button(Button::Left, Direction::Release) {
                let err = XctrlError::new(format!("Failed to end drag: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
        MouseAction::Position => match platform::query_position() {
            Ok((x, y)) => {
                let pos = MousePosition { x, y };
                let text = format!("x: {x}, y: {y}");
                print_success(&pos, &text, json);
            }
            Err(e) => {
                let err = XctrlError::with_hint(
                    format!("Failed to get cursor position: {e}"),
                    "Ensure a display server is running.",
                );
                exit_with_error(&err, json, 1);
            }
        },
    }
}
