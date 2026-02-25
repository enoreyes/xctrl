use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "xctrl",
    version,
    about = "Cross-platform computer control CLI",
    long_about = "xctrl is a lightweight, OS-agnostic CLI for fine-grained computer control.\n\nUsage: xctrl <primitive> <action> [options]"
)]
pub struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Primitive,
}

#[derive(Subcommand, Debug)]
pub enum Primitive {
    /// Control the mouse cursor (move, click, scroll, drag, position)
    Mouse {
        #[command(subcommand)]
        action: MouseAction,
    },
    /// Simulate keyboard input (type, press, hotkey, key-down, key-up)
    Keyboard {
        #[command(subcommand)]
        action: KeyboardAction,
    },
    /// Read and write the system clipboard (get, set, clear)
    Clipboard {
        #[command(subcommand)]
        action: ClipboardAction,
    },
    /// Capture screenshots and query display information (screenshot, info, list)
    Display {
        #[command(subcommand)]
        action: DisplayAction,
    },
    /// Record the screen (record start, record stop, record status)
    Screen {
        #[command(subcommand)]
        action: ScreenAction,
    },
    /// Manage application windows (list, focus, resize, move, minimize, maximize, fullscreen)
    Window {
        #[command(subcommand)]
        action: WindowAction,
    },
    /// OS-level actions (open-url, open-app, notify, frontmost-app, list-apps)
    Os {
        #[command(subcommand)]
        action: OsAction,
    },
}

// -- Mouse actions --

#[derive(Subcommand, Debug)]
pub enum MouseAction {
    /// Move the cursor to an absolute position
    Move {
        /// X coordinate
        #[arg(long, allow_hyphen_values = true)]
        x: i32,
        /// Y coordinate
        #[arg(long, allow_hyphen_values = true)]
        y: i32,
    },
    /// Perform a left click at the current or specified position
    Click {
        /// X coordinate (optional, moves before clicking)
        #[arg(long, allow_hyphen_values = true)]
        x: Option<i32>,
        /// Y coordinate (optional, moves before clicking)
        #[arg(long, allow_hyphen_values = true)]
        y: Option<i32>,
    },
    /// Perform a double-click at the current or specified position
    DoubleClick {
        /// X coordinate (optional)
        #[arg(long, allow_hyphen_values = true)]
        x: Option<i32>,
        /// Y coordinate (optional)
        #[arg(long, allow_hyphen_values = true)]
        y: Option<i32>,
    },
    /// Perform a right-click at the current or specified position
    RightClick {
        /// X coordinate (optional)
        #[arg(long, allow_hyphen_values = true)]
        x: Option<i32>,
        /// Y coordinate (optional)
        #[arg(long, allow_hyphen_values = true)]
        y: Option<i32>,
    },
    /// Scroll by a given amount (positive = up, negative = down)
    Scroll {
        /// Scroll amount (positive = up, negative = down)
        #[arg(long, allow_hyphen_values = true)]
        amount: i32,
    },
    /// Drag from one position to another
    Drag {
        /// Starting X coordinate
        #[arg(long, allow_hyphen_values = true)]
        from_x: i32,
        /// Starting Y coordinate
        #[arg(long, allow_hyphen_values = true)]
        from_y: i32,
        /// Ending X coordinate
        #[arg(long, allow_hyphen_values = true)]
        to_x: i32,
        /// Ending Y coordinate
        #[arg(long, allow_hyphen_values = true)]
        to_y: i32,
    },
    /// Get the current cursor position
    Position,
}

// -- Keyboard actions --

#[derive(Subcommand, Debug)]
pub enum KeyboardAction {
    /// Type a text string as keystrokes
    Type {
        /// The text to type
        text: String,
    },
    /// Press and release a named key (e.g., enter, tab, escape)
    Press {
        /// Key name (e.g., enter, tab, escape, backspace, space, up, down, left, right, f1-f12)
        key: String,
    },
    /// Execute a key combination (e.g., ctrl c, ctrl shift s)
    Hotkey {
        /// Keys in the combination (e.g., ctrl c, ctrl shift s)
        keys: Vec<String>,
    },
    /// Hold a key down without releasing
    KeyDown {
        /// Key name to hold down
        key: String,
    },
    /// Release a held key
    KeyUp {
        /// Key name to release
        key: String,
    },
}

// -- Clipboard actions --

#[derive(Subcommand, Debug)]
pub enum ClipboardAction {
    /// Write text to the system clipboard
    Set {
        /// Text to write to the clipboard
        text: String,
    },
    /// Read and print the current clipboard text
    Get,
    /// Clear the system clipboard
    Clear,
}

// -- Display actions --

#[derive(Subcommand, Debug)]
pub enum DisplayAction {
    /// Capture a screenshot to a file
    Screenshot {
        /// Output file path (PNG format)
        #[arg(long)]
        output: String,
        /// X coordinate of the capture region (optional)
        #[arg(long)]
        x: Option<u32>,
        /// Y coordinate of the capture region (optional)
        #[arg(long)]
        y: Option<u32>,
        /// Width of the capture region (optional)
        #[arg(long)]
        width: Option<u32>,
        /// Height of the capture region (optional)
        #[arg(long)]
        height: Option<u32>,
    },
    /// Show information about the primary display
    Info,
    /// List all connected monitors
    List,
}

// -- Screen recording actions --

#[derive(Subcommand, Debug)]
pub enum ScreenAction {
    /// Manage screen recording
    Record {
        #[command(subcommand)]
        action: RecordAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum RecordAction {
    /// Start recording the screen
    Start {
        /// Output file path for the recording
        #[arg(long)]
        output: String,
        /// Recording framerate (default: 30)
        #[arg(long, default_value = "30")]
        framerate: u32,
    },
    /// Stop the active screen recording
    Stop,
    /// Check the status of screen recording
    Status,
}

// -- Window actions --

#[derive(Subcommand, Debug)]
pub enum WindowAction {
    /// List all visible windows
    List,
    /// Bring a window to the foreground
    Focus {
        /// Window title to match
        #[arg(long)]
        title: Option<String>,
        /// Window ID
        #[arg(long)]
        id: Option<String>,
    },
    /// Resize a window
    Resize {
        /// Window title to match
        #[arg(long)]
        title: Option<String>,
        /// Window ID
        #[arg(long)]
        id: Option<String>,
        /// New width
        #[arg(long)]
        width: u32,
        /// New height
        #[arg(long)]
        height: u32,
    },
    /// Move a window to a new position
    Move {
        /// Window title to match
        #[arg(long)]
        title: Option<String>,
        /// Window ID
        #[arg(long)]
        id: Option<String>,
        /// New X position
        #[arg(long)]
        x: i32,
        /// New Y position
        #[arg(long)]
        y: i32,
    },
    /// Minimize a window
    Minimize {
        /// Window title to match
        #[arg(long)]
        title: Option<String>,
        /// Window ID
        #[arg(long)]
        id: Option<String>,
    },
    /// Maximize a window
    Maximize {
        /// Window title to match
        #[arg(long)]
        title: Option<String>,
        /// Window ID
        #[arg(long)]
        id: Option<String>,
    },
    /// Put a window into fullscreen mode
    Fullscreen {
        /// Window title to match
        #[arg(long)]
        title: Option<String>,
        /// Window ID
        #[arg(long)]
        id: Option<String>,
    },
}

// -- OS actions --

#[derive(Subcommand, Debug)]
pub enum OsAction {
    /// Open a URL in the default browser
    OpenUrl {
        /// URL to open
        url: String,
    },
    /// Launch an application by name
    OpenApp {
        /// Application name to launch
        name: String,
    },
    /// Send a desktop notification
    Notify {
        /// Notification title
        #[arg(long)]
        title: String,
        /// Notification body text
        #[arg(long)]
        body: String,
    },
    /// Get the name of the currently focused application
    FrontmostApp,
    /// List running applications
    ListApps,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_debug_assert() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_parse_mouse_move() {
        let cli = Cli::try_parse_from(["xctrl", "mouse", "move", "--x", "100", "--y", "200"])
            .expect("should parse mouse move");
        match cli.command {
            Primitive::Mouse {
                action: MouseAction::Move { x, y },
            } => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
            }
            _ => panic!("expected Mouse Move"),
        }
    }

    #[test]
    fn test_parse_mouse_click_without_position() {
        let cli =
            Cli::try_parse_from(["xctrl", "mouse", "click"]).expect("should parse mouse click");
        match cli.command {
            Primitive::Mouse {
                action: MouseAction::Click { x, y },
            } => {
                assert!(x.is_none());
                assert!(y.is_none());
            }
            _ => panic!("expected Mouse Click"),
        }
    }

    #[test]
    fn test_parse_mouse_click_with_position() {
        let cli = Cli::try_parse_from(["xctrl", "mouse", "click", "--x", "50", "--y", "75"])
            .expect("should parse mouse click with position");
        match cli.command {
            Primitive::Mouse {
                action: MouseAction::Click { x, y },
            } => {
                assert_eq!(x, Some(50));
                assert_eq!(y, Some(75));
            }
            _ => panic!("expected Mouse Click"),
        }
    }

    #[test]
    fn test_parse_mouse_double_click() {
        let cli = Cli::try_parse_from(["xctrl", "mouse", "double-click"])
            .expect("should parse mouse double-click");
        assert!(matches!(
            cli.command,
            Primitive::Mouse {
                action: MouseAction::DoubleClick { .. }
            }
        ));
    }

    #[test]
    fn test_parse_mouse_right_click() {
        let cli = Cli::try_parse_from(["xctrl", "mouse", "right-click"])
            .expect("should parse mouse right-click");
        assert!(matches!(
            cli.command,
            Primitive::Mouse {
                action: MouseAction::RightClick { .. }
            }
        ));
    }

    #[test]
    fn test_parse_mouse_scroll() {
        let cli = Cli::try_parse_from(["xctrl", "mouse", "scroll", "--amount=-5"])
            .expect("should parse mouse scroll");
        match cli.command {
            Primitive::Mouse {
                action: MouseAction::Scroll { amount },
            } => {
                assert_eq!(amount, -5);
            }
            _ => panic!("expected Mouse Scroll"),
        }
    }

    #[test]
    fn test_parse_mouse_drag() {
        let cli = Cli::try_parse_from([
            "xctrl", "mouse", "drag", "--from-x", "100", "--from-y", "100", "--to-x", "500",
            "--to-y", "500",
        ])
        .expect("should parse mouse drag");
        match cli.command {
            Primitive::Mouse {
                action:
                    MouseAction::Drag {
                        from_x,
                        from_y,
                        to_x,
                        to_y,
                    },
            } => {
                assert_eq!(from_x, 100);
                assert_eq!(from_y, 100);
                assert_eq!(to_x, 500);
                assert_eq!(to_y, 500);
            }
            _ => panic!("expected Mouse Drag"),
        }
    }

    #[test]
    fn test_parse_mouse_position() {
        let cli = Cli::try_parse_from(["xctrl", "mouse", "position"])
            .expect("should parse mouse position");
        assert!(matches!(
            cli.command,
            Primitive::Mouse {
                action: MouseAction::Position
            }
        ));
    }

    #[test]
    fn test_parse_keyboard_type() {
        let cli = Cli::try_parse_from(["xctrl", "keyboard", "type", "Hello, world!"])
            .expect("should parse keyboard type");
        match cli.command {
            Primitive::Keyboard {
                action: KeyboardAction::Type { text },
            } => {
                assert_eq!(text, "Hello, world!");
            }
            _ => panic!("expected Keyboard Type"),
        }
    }

    #[test]
    fn test_parse_keyboard_press() {
        let cli = Cli::try_parse_from(["xctrl", "keyboard", "press", "enter"])
            .expect("should parse keyboard press");
        match cli.command {
            Primitive::Keyboard {
                action: KeyboardAction::Press { key },
            } => {
                assert_eq!(key, "enter");
            }
            _ => panic!("expected Keyboard Press"),
        }
    }

    #[test]
    fn test_parse_keyboard_hotkey() {
        let cli = Cli::try_parse_from(["xctrl", "keyboard", "hotkey", "ctrl", "shift", "s"])
            .expect("should parse keyboard hotkey");
        match cli.command {
            Primitive::Keyboard {
                action: KeyboardAction::Hotkey { keys },
            } => {
                assert_eq!(keys, vec!["ctrl", "shift", "s"]);
            }
            _ => panic!("expected Keyboard Hotkey"),
        }
    }

    #[test]
    fn test_parse_keyboard_key_down() {
        let cli = Cli::try_parse_from(["xctrl", "keyboard", "key-down", "shift"])
            .expect("should parse keyboard key-down");
        match cli.command {
            Primitive::Keyboard {
                action: KeyboardAction::KeyDown { key },
            } => {
                assert_eq!(key, "shift");
            }
            _ => panic!("expected Keyboard KeyDown"),
        }
    }

    #[test]
    fn test_parse_keyboard_key_up() {
        let cli = Cli::try_parse_from(["xctrl", "keyboard", "key-up", "shift"])
            .expect("should parse keyboard key-up");
        match cli.command {
            Primitive::Keyboard {
                action: KeyboardAction::KeyUp { key },
            } => {
                assert_eq!(key, "shift");
            }
            _ => panic!("expected Keyboard KeyUp"),
        }
    }

    #[test]
    fn test_parse_clipboard_set() {
        let cli = Cli::try_parse_from(["xctrl", "clipboard", "set", "hello"])
            .expect("should parse clipboard set");
        match cli.command {
            Primitive::Clipboard {
                action: ClipboardAction::Set { text },
            } => {
                assert_eq!(text, "hello");
            }
            _ => panic!("expected Clipboard Set"),
        }
    }

    #[test]
    fn test_parse_clipboard_get() {
        let cli =
            Cli::try_parse_from(["xctrl", "clipboard", "get"]).expect("should parse clipboard get");
        assert!(matches!(
            cli.command,
            Primitive::Clipboard {
                action: ClipboardAction::Get
            }
        ));
    }

    #[test]
    fn test_parse_clipboard_clear() {
        let cli = Cli::try_parse_from(["xctrl", "clipboard", "clear"])
            .expect("should parse clipboard clear");
        assert!(matches!(
            cli.command,
            Primitive::Clipboard {
                action: ClipboardAction::Clear
            }
        ));
    }

    #[test]
    fn test_parse_display_screenshot() {
        let cli = Cli::try_parse_from([
            "xctrl",
            "display",
            "screenshot",
            "--output",
            "/tmp/shot.png",
        ])
        .expect("should parse display screenshot");
        match cli.command {
            Primitive::Display {
                action:
                    DisplayAction::Screenshot {
                        output,
                        x,
                        y,
                        width,
                        height,
                    },
            } => {
                assert_eq!(output, "/tmp/shot.png");
                assert!(x.is_none());
                assert!(y.is_none());
                assert!(width.is_none());
                assert!(height.is_none());
            }
            _ => panic!("expected Display Screenshot"),
        }
    }

    #[test]
    fn test_parse_display_screenshot_with_region() {
        let cli = Cli::try_parse_from([
            "xctrl",
            "display",
            "screenshot",
            "--output",
            "/tmp/region.png",
            "--x",
            "0",
            "--y",
            "0",
            "--width",
            "100",
            "--height",
            "100",
        ])
        .expect("should parse display screenshot with region");
        match cli.command {
            Primitive::Display {
                action:
                    DisplayAction::Screenshot {
                        output,
                        x,
                        y,
                        width,
                        height,
                    },
            } => {
                assert_eq!(output, "/tmp/region.png");
                assert_eq!(x, Some(0));
                assert_eq!(y, Some(0));
                assert_eq!(width, Some(100));
                assert_eq!(height, Some(100));
            }
            _ => panic!("expected Display Screenshot with region"),
        }
    }

    #[test]
    fn test_parse_display_info() {
        let cli =
            Cli::try_parse_from(["xctrl", "display", "info"]).expect("should parse display info");
        assert!(matches!(
            cli.command,
            Primitive::Display {
                action: DisplayAction::Info
            }
        ));
    }

    #[test]
    fn test_parse_display_list() {
        let cli =
            Cli::try_parse_from(["xctrl", "display", "list"]).expect("should parse display list");
        assert!(matches!(
            cli.command,
            Primitive::Display {
                action: DisplayAction::List
            }
        ));
    }

    #[test]
    fn test_parse_screen_record_start() {
        let cli = Cli::try_parse_from([
            "xctrl",
            "screen",
            "record",
            "start",
            "--output",
            "/tmp/rec.mp4",
        ])
        .expect("should parse screen record start");
        match cli.command {
            Primitive::Screen {
                action:
                    ScreenAction::Record {
                        action: RecordAction::Start { output, framerate },
                    },
            } => {
                assert_eq!(output, "/tmp/rec.mp4");
                assert_eq!(framerate, 30); // default
            }
            _ => panic!("expected Screen Record Start"),
        }
    }

    #[test]
    fn test_parse_screen_record_start_with_framerate() {
        let cli = Cli::try_parse_from([
            "xctrl",
            "screen",
            "record",
            "start",
            "--output",
            "/tmp/rec.mp4",
            "--framerate",
            "60",
        ])
        .expect("should parse screen record start with framerate");
        match cli.command {
            Primitive::Screen {
                action:
                    ScreenAction::Record {
                        action: RecordAction::Start { output, framerate },
                    },
            } => {
                assert_eq!(output, "/tmp/rec.mp4");
                assert_eq!(framerate, 60);
            }
            _ => panic!("expected Screen Record Start"),
        }
    }

    #[test]
    fn test_parse_screen_record_stop() {
        let cli = Cli::try_parse_from(["xctrl", "screen", "record", "stop"])
            .expect("should parse screen record stop");
        assert!(matches!(
            cli.command,
            Primitive::Screen {
                action: ScreenAction::Record {
                    action: RecordAction::Stop
                }
            }
        ));
    }

    #[test]
    fn test_parse_screen_record_status() {
        let cli = Cli::try_parse_from(["xctrl", "screen", "record", "status"])
            .expect("should parse screen record status");
        assert!(matches!(
            cli.command,
            Primitive::Screen {
                action: ScreenAction::Record {
                    action: RecordAction::Status
                }
            }
        ));
    }

    #[test]
    fn test_parse_window_list() {
        let cli =
            Cli::try_parse_from(["xctrl", "window", "list"]).expect("should parse window list");
        assert!(matches!(
            cli.command,
            Primitive::Window {
                action: WindowAction::List
            }
        ));
    }

    #[test]
    fn test_parse_window_focus_by_title() {
        let cli = Cli::try_parse_from(["xctrl", "window", "focus", "--title", "MyWindow"])
            .expect("should parse window focus");
        match cli.command {
            Primitive::Window {
                action: WindowAction::Focus { title, id },
            } => {
                assert_eq!(title, Some("MyWindow".to_string()));
                assert!(id.is_none());
            }
            _ => panic!("expected Window Focus"),
        }
    }

    #[test]
    fn test_parse_window_focus_by_id() {
        let cli = Cli::try_parse_from(["xctrl", "window", "focus", "--id", "12345"])
            .expect("should parse window focus by id");
        match cli.command {
            Primitive::Window {
                action: WindowAction::Focus { title, id },
            } => {
                assert!(title.is_none());
                assert_eq!(id, Some("12345".to_string()));
            }
            _ => panic!("expected Window Focus"),
        }
    }

    #[test]
    fn test_parse_window_resize() {
        let cli = Cli::try_parse_from([
            "xctrl", "window", "resize", "--title", "MyWin", "--width", "800", "--height", "600",
        ])
        .expect("should parse window resize");
        match cli.command {
            Primitive::Window {
                action:
                    WindowAction::Resize {
                        title,
                        id,
                        width,
                        height,
                    },
            } => {
                assert_eq!(title, Some("MyWin".to_string()));
                assert!(id.is_none());
                assert_eq!(width, 800);
                assert_eq!(height, 600);
            }
            _ => panic!("expected Window Resize"),
        }
    }

    #[test]
    fn test_parse_window_move() {
        let cli = Cli::try_parse_from([
            "xctrl", "window", "move", "--title", "MyWin", "--x", "100", "--y", "200",
        ])
        .expect("should parse window move");
        match cli.command {
            Primitive::Window {
                action:
                    WindowAction::Move {
                        title, id, x, y, ..
                    },
            } => {
                assert_eq!(title, Some("MyWin".to_string()));
                assert!(id.is_none());
                assert_eq!(x, 100);
                assert_eq!(y, 200);
            }
            _ => panic!("expected Window Move"),
        }
    }

    #[test]
    fn test_parse_window_minimize() {
        let cli = Cli::try_parse_from(["xctrl", "window", "minimize", "--title", "MyWin"])
            .expect("should parse window minimize");
        assert!(matches!(
            cli.command,
            Primitive::Window {
                action: WindowAction::Minimize { .. }
            }
        ));
    }

    #[test]
    fn test_parse_window_maximize() {
        let cli = Cli::try_parse_from(["xctrl", "window", "maximize", "--id", "999"])
            .expect("should parse window maximize");
        assert!(matches!(
            cli.command,
            Primitive::Window {
                action: WindowAction::Maximize { .. }
            }
        ));
    }

    #[test]
    fn test_parse_window_fullscreen() {
        let cli = Cli::try_parse_from(["xctrl", "window", "fullscreen", "--title", "MyWin"])
            .expect("should parse window fullscreen");
        assert!(matches!(
            cli.command,
            Primitive::Window {
                action: WindowAction::Fullscreen { .. }
            }
        ));
    }

    #[test]
    fn test_parse_os_open_url() {
        let cli = Cli::try_parse_from(["xctrl", "os", "open-url", "https://example.com"])
            .expect("should parse os open-url");
        match cli.command {
            Primitive::Os {
                action: OsAction::OpenUrl { url },
            } => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("expected Os OpenUrl"),
        }
    }

    #[test]
    fn test_parse_os_open_app() {
        let cli = Cli::try_parse_from(["xctrl", "os", "open-app", "firefox"])
            .expect("should parse os open-app");
        match cli.command {
            Primitive::Os {
                action: OsAction::OpenApp { name },
            } => {
                assert_eq!(name, "firefox");
            }
            _ => panic!("expected Os OpenApp"),
        }
    }

    #[test]
    fn test_parse_os_notify() {
        let cli = Cli::try_parse_from([
            "xctrl", "os", "notify", "--title", "Hello", "--body", "World",
        ])
        .expect("should parse os notify");
        match cli.command {
            Primitive::Os {
                action: OsAction::Notify { title, body },
            } => {
                assert_eq!(title, "Hello");
                assert_eq!(body, "World");
            }
            _ => panic!("expected Os Notify"),
        }
    }

    #[test]
    fn test_parse_os_frontmost_app() {
        let cli = Cli::try_parse_from(["xctrl", "os", "frontmost-app"])
            .expect("should parse os frontmost-app");
        assert!(matches!(
            cli.command,
            Primitive::Os {
                action: OsAction::FrontmostApp
            }
        ));
    }

    #[test]
    fn test_parse_os_list_apps() {
        let cli =
            Cli::try_parse_from(["xctrl", "os", "list-apps"]).expect("should parse os list-apps");
        assert!(matches!(
            cli.command,
            Primitive::Os {
                action: OsAction::ListApps
            }
        ));
    }

    #[test]
    fn test_parse_json_flag() {
        let cli =
            Cli::try_parse_from(["xctrl", "--json", "mouse", "position"]).expect("should parse");
        assert!(cli.json);
    }

    #[test]
    fn test_parse_json_flag_after_primitive() {
        let cli =
            Cli::try_parse_from(["xctrl", "mouse", "--json", "position"]).expect("should parse");
        assert!(cli.json);
    }

    #[test]
    fn test_unknown_subcommand_error() {
        let result = Cli::try_parse_from(["xctrl", "nonexistent"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("invalid value")
                || err_str.contains("unrecognized")
                || err_str.contains("found")
                || err_str.contains("valid subcommand"),
            "Error should mention invalid subcommand: {err_str}"
        );
    }

    #[test]
    fn test_version_flag() {
        let result = Cli::try_parse_from(["xctrl", "--version"]);
        // --version causes clap to return a DisplayVersion error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
        let output = err.to_string();
        assert!(
            output.contains("0.1.0"),
            "Version output should contain 0.1.0: {output}"
        );
    }

    #[test]
    fn test_help_flag() {
        let result = Cli::try_parse_from(["xctrl", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        let output = err.to_string();
        // Verify all 7 primitives are listed
        assert!(output.contains("mouse"), "Help should list mouse");
        assert!(output.contains("keyboard"), "Help should list keyboard");
        assert!(output.contains("clipboard"), "Help should list clipboard");
        assert!(output.contains("display"), "Help should list display");
        assert!(output.contains("screen"), "Help should list screen");
        assert!(output.contains("window"), "Help should list window");
        assert!(output.contains("os"), "Help should list os");
    }
}
