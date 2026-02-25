mod cli;
mod clipboard;
mod display;
mod error;
mod keyboard;
mod mouse;
mod output;
mod screen;

use clap::Parser;

use cli::{Cli, OsAction, Primitive, ScreenAction, WindowAction};

fn main() {
    // Check if we're running as a clipboard daemon (Linux only).
    // This must happen before clap parses args, because the daemon
    // uses a special internal argument that isn't a valid CLI command.
    #[cfg(target_os = "linux")]
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(text) = clipboard::check_daemon_args(&args) {
            clipboard::run_clipboard_daemon(&text);
            return;
        }
    }

    let cli = Cli::parse();
    let json = cli.json;

    match cli.command {
        Primitive::Mouse { action } => mouse::handle_mouse(action, json),
        Primitive::Keyboard { action } => keyboard::handle_keyboard(action, json),
        Primitive::Clipboard { action } => clipboard::handle_clipboard(action, json),
        Primitive::Display { action } => display::handle_display(action, json),
        Primitive::Screen { action } => handle_screen(action, json),
        Primitive::Window { action } => handle_window(action, json),
        Primitive::Os { action } => handle_os(action, json),
    }
}

fn handle_screen(action: ScreenAction, json: bool) {
    match action {
        ScreenAction::Record { action } => screen::handle_screen(action, json),
    }
}

fn handle_window(action: WindowAction, json: bool) {
    match action {
        WindowAction::List => output::not_yet_implemented("window list", json),
        WindowAction::Focus { .. } => output::not_yet_implemented("window focus", json),
        WindowAction::Resize { .. } => output::not_yet_implemented("window resize", json),
        WindowAction::Move { .. } => output::not_yet_implemented("window move", json),
        WindowAction::Minimize { .. } => output::not_yet_implemented("window minimize", json),
        WindowAction::Maximize { .. } => output::not_yet_implemented("window maximize", json),
        WindowAction::Fullscreen { .. } => output::not_yet_implemented("window fullscreen", json),
    }
}

fn handle_os(action: OsAction, json: bool) {
    match action {
        OsAction::OpenUrl { .. } => output::not_yet_implemented("os open-url", json),
        OsAction::OpenApp { .. } => output::not_yet_implemented("os open-app", json),
        OsAction::Notify { .. } => output::not_yet_implemented("os notify", json),
        OsAction::FrontmostApp => output::not_yet_implemented("os frontmost-app", json),
        OsAction::ListApps => output::not_yet_implemented("os list-apps", json),
    }
}
