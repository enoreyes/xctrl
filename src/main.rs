mod cli;
mod error;
mod mouse;
mod output;

use clap::Parser;

use cli::{
    Cli, ClipboardAction, DisplayAction, KeyboardAction, OsAction, Primitive, ScreenAction,
    WindowAction,
};

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    match cli.command {
        Primitive::Mouse { action } => mouse::handle_mouse(action, json),
        Primitive::Keyboard { action } => handle_keyboard(action, json),
        Primitive::Clipboard { action } => handle_clipboard(action, json),
        Primitive::Display { action } => handle_display(action, json),
        Primitive::Screen { action } => handle_screen(action, json),
        Primitive::Window { action } => handle_window(action, json),
        Primitive::Os { action } => handle_os(action, json),
    }
}

fn handle_keyboard(action: KeyboardAction, json: bool) {
    match action {
        KeyboardAction::Type { .. } => output::not_yet_implemented("keyboard type", json),
        KeyboardAction::Press { .. } => output::not_yet_implemented("keyboard press", json),
        KeyboardAction::Hotkey { .. } => output::not_yet_implemented("keyboard hotkey", json),
        KeyboardAction::KeyDown { .. } => output::not_yet_implemented("keyboard key-down", json),
        KeyboardAction::KeyUp { .. } => output::not_yet_implemented("keyboard key-up", json),
    }
}

fn handle_clipboard(action: ClipboardAction, json: bool) {
    match action {
        ClipboardAction::Set { .. } => output::not_yet_implemented("clipboard set", json),
        ClipboardAction::Get => output::not_yet_implemented("clipboard get", json),
        ClipboardAction::Clear => output::not_yet_implemented("clipboard clear", json),
    }
}

fn handle_display(action: DisplayAction, json: bool) {
    match action {
        DisplayAction::Screenshot { .. } => output::not_yet_implemented("display screenshot", json),
        DisplayAction::Info => output::not_yet_implemented("display info", json),
        DisplayAction::List => output::not_yet_implemented("display list", json),
    }
}

fn handle_screen(action: ScreenAction, json: bool) {
    match action {
        ScreenAction::Record { action } => match action {
            cli::RecordAction::Start { .. } => {
                output::not_yet_implemented("screen record start", json)
            }
            cli::RecordAction::Stop => output::not_yet_implemented("screen record stop", json),
            cli::RecordAction::Status => output::not_yet_implemented("screen record status", json),
        },
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
