mod cli;
mod clipboard;
mod display;
mod error;
mod keyboard;
mod mouse;
mod os_actions;
mod output;
mod screen;
mod window;

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
    window::handle_window(action, json);
}

fn handle_os(action: OsAction, json: bool) {
    os_actions::handle_os(action, json);
}
