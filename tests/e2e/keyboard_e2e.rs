//! Keyboard e2e tests — Linux only, requires Xvfb (DISPLAY must be set).

#[cfg(target_os = "linux")]
mod linux {
    use crate::{has_display, xctrl_bin};
    use std::process::Command;

    #[test]
    fn e2e_keyboard_type_exits_0() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let output = Command::new(xctrl_bin())
            .args(["keyboard", "type", "hello e2e test"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            output.status.success(),
            "keyboard type should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e2e_keyboard_press_enter_exits_0() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let output = Command::new(xctrl_bin())
            .args(["keyboard", "press", "enter"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            output.status.success(),
            "keyboard press enter should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e2e_keyboard_press_tab_exits_0() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let output = Command::new(xctrl_bin())
            .args(["keyboard", "press", "tab"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            output.status.success(),
            "keyboard press tab should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e2e_keyboard_press_escape_exits_0() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let output = Command::new(xctrl_bin())
            .args(["keyboard", "press", "escape"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            output.status.success(),
            "keyboard press escape should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e2e_keyboard_hotkey_ctrl_c_exits_0() {
        if !has_display() {
            eprintln!("SKIP: no DISPLAY set");
            return;
        }
        let output = Command::new(xctrl_bin())
            .args(["keyboard", "hotkey", "ctrl", "c"])
            .env("DISPLAY", std::env::var("DISPLAY").unwrap())
            .output()
            .expect("failed to run xctrl");
        assert!(
            output.status.success(),
            "keyboard hotkey ctrl c should exit 0: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
