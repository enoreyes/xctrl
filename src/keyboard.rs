use crate::cli::KeyboardAction;
use crate::error::{exit_with_error, XctrlError};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// All recognized key names (lowercase). Used for validation and error messages.
const VALID_KEYS: &[&str] = &[
    // Modifiers
    "ctrl",
    "alt",
    "shift",
    "super",
    "meta",
    // Navigation
    "enter",
    "return",
    "tab",
    "escape",
    "esc",
    "backspace",
    "delete",
    "space",
    "insert",
    "home",
    "end",
    "pageup",
    "pagedown",
    // Arrows
    "up",
    "down",
    "left",
    "right",
    // Function keys
    "f1",
    "f2",
    "f3",
    "f4",
    "f5",
    "f6",
    "f7",
    "f8",
    "f9",
    "f10",
    "f11",
    "f12",
    // Printable single-char keys (a-z, 0-9, symbols) are also accepted
];

/// Map a key name string (case-insensitive) to an enigo `Key`.
///
/// Accepts named keys (enter, tab, f1, etc.) and single characters (a, b, 1, /, etc.).
/// Returns `Err(String)` with a helpful message if the key name is not recognized.
pub fn parse_key(name: &str) -> Result<Key, String> {
    let lower = name.to_lowercase();
    match lower.as_str() {
        // Modifiers
        "ctrl" | "control" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "shift" => Ok(Key::Shift),
        "super" | "meta" | "win" | "command" | "cmd" => Ok(Key::Meta),
        // Navigation
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "space" => Ok(Key::Space),
        "insert" => Ok(Key::Other(0xff63)), // XK_Insert
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        // Arrows
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        // Function keys
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        // Single character (letter, digit, symbol)
        _ => {
            let chars: Vec<char> = lower.chars().collect();
            if chars.len() == 1 {
                Ok(Key::Unicode(chars[0]))
            } else {
                Err(format!(
                    "Unknown key: '{}'. Valid keys: {}",
                    name,
                    valid_key_list()
                ))
            }
        }
    }
}

/// Returns a comma-separated list of valid key names for error messages.
fn valid_key_list() -> String {
    let mut keys: Vec<&str> = VALID_KEYS.to_vec();
    keys.push("a-z");
    keys.push("0-9");
    keys.join(", ")
}

/// Returns whether a key name string represents a modifier key.
fn is_modifier(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "ctrl" | "control" | "alt" | "shift" | "super" | "meta" | "win" | "command" | "cmd"
    )
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

pub fn handle_keyboard(action: KeyboardAction, json: bool) {
    match action {
        KeyboardAction::Type { text } => {
            let mut enigo = create_enigo(json);
            if let Err(e) = enigo.text(&text) {
                let err = XctrlError::new(format!("Failed to type text: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
        KeyboardAction::Press { key } => {
            let parsed = match parse_key(&key) {
                Ok(k) => k,
                Err(msg) => {
                    let err = XctrlError::with_hint(msg, "Use --help to see available commands.");
                    exit_with_error(&err, json, 1);
                }
            };
            let mut enigo = create_enigo(json);
            if let Err(e) = enigo.key(parsed, Direction::Click) {
                let err = XctrlError::new(format!("Failed to press key: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
        KeyboardAction::Hotkey { keys } => {
            if keys.is_empty() {
                let err = XctrlError::with_hint(
                    "No keys specified for hotkey.".to_string(),
                    "Usage: xctrl keyboard hotkey <modifier...> <key> (e.g., ctrl c, ctrl shift s)",
                );
                exit_with_error(&err, json, 1);
            }

            // Parse all keys first to fail early on invalid input
            let parsed: Vec<(Key, bool)> = keys
                .iter()
                .map(|k| {
                    let parsed_key = match parse_key(k) {
                        Ok(pk) => pk,
                        Err(msg) => {
                            let err =
                                XctrlError::with_hint(msg, "Use --help to see available commands.");
                            exit_with_error(&err, json, 1);
                        }
                    };
                    (parsed_key, is_modifier(k))
                })
                .collect();

            let mut enigo = create_enigo(json);

            // Press modifiers down
            for (key, is_mod) in &parsed {
                if *is_mod {
                    if let Err(e) = enigo.key(*key, Direction::Press) {
                        let err = XctrlError::new(format!("Failed to press modifier key: {e}"));
                        exit_with_error(&err, json, 1);
                    }
                }
            }

            // Click non-modifier keys
            for (key, is_mod) in &parsed {
                if !*is_mod {
                    if let Err(e) = enigo.key(*key, Direction::Click) {
                        let err = XctrlError::new(format!("Failed to press key: {e}"));
                        exit_with_error(&err, json, 1);
                    }
                }
            }

            // Release modifiers in reverse order
            for (key, is_mod) in parsed.iter().rev() {
                if *is_mod {
                    if let Err(e) = enigo.key(*key, Direction::Release) {
                        let err = XctrlError::new(format!("Failed to release modifier key: {e}"));
                        exit_with_error(&err, json, 1);
                    }
                }
            }
        }
        KeyboardAction::KeyDown { key } => {
            let parsed = match parse_key(&key) {
                Ok(k) => k,
                Err(msg) => {
                    let err = XctrlError::with_hint(msg, "Use --help to see available commands.");
                    exit_with_error(&err, json, 1);
                }
            };
            let mut enigo = create_enigo(json);
            if let Err(e) = enigo.key(parsed, Direction::Press) {
                let err = XctrlError::new(format!("Failed to hold key down: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
        KeyboardAction::KeyUp { key } => {
            let parsed = match parse_key(&key) {
                Ok(k) => k,
                Err(msg) => {
                    let err = XctrlError::with_hint(msg, "Use --help to see available commands.");
                    exit_with_error(&err, json, 1);
                }
            };
            let mut enigo = create_enigo(json);
            if let Err(e) = enigo.key(parsed, Direction::Release) {
                let err = XctrlError::new(format!("Failed to release key: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Key name parsing tests ----

    #[test]
    fn test_parse_key_enter() {
        assert!(matches!(parse_key("enter"), Ok(Key::Return)));
        assert!(matches!(parse_key("Enter"), Ok(Key::Return)));
        assert!(matches!(parse_key("ENTER"), Ok(Key::Return)));
        assert!(matches!(parse_key("return"), Ok(Key::Return)));
    }

    #[test]
    fn test_parse_key_tab() {
        assert!(matches!(parse_key("tab"), Ok(Key::Tab)));
        assert!(matches!(parse_key("Tab"), Ok(Key::Tab)));
    }

    #[test]
    fn test_parse_key_escape() {
        assert!(matches!(parse_key("escape"), Ok(Key::Escape)));
        assert!(matches!(parse_key("esc"), Ok(Key::Escape)));
    }

    #[test]
    fn test_parse_key_backspace() {
        assert!(matches!(parse_key("backspace"), Ok(Key::Backspace)));
    }

    #[test]
    fn test_parse_key_space() {
        assert!(matches!(parse_key("space"), Ok(Key::Space)));
    }

    #[test]
    fn test_parse_key_arrows() {
        assert!(matches!(parse_key("up"), Ok(Key::UpArrow)));
        assert!(matches!(parse_key("down"), Ok(Key::DownArrow)));
        assert!(matches!(parse_key("left"), Ok(Key::LeftArrow)));
        assert!(matches!(parse_key("right"), Ok(Key::RightArrow)));
    }

    #[test]
    fn test_parse_key_function_keys() {
        assert!(matches!(parse_key("f1"), Ok(Key::F1)));
        assert!(matches!(parse_key("F1"), Ok(Key::F1)));
        assert!(matches!(parse_key("f12"), Ok(Key::F12)));
        assert!(matches!(parse_key("F12"), Ok(Key::F12)));
    }

    #[test]
    fn test_parse_key_modifiers() {
        assert!(matches!(parse_key("ctrl"), Ok(Key::Control)));
        assert!(matches!(parse_key("alt"), Ok(Key::Alt)));
        assert!(matches!(parse_key("shift"), Ok(Key::Shift)));
        assert!(matches!(parse_key("super"), Ok(Key::Meta)));
        assert!(matches!(parse_key("meta"), Ok(Key::Meta)));
    }

    #[test]
    fn test_parse_key_delete() {
        assert!(matches!(parse_key("delete"), Ok(Key::Delete)));
        assert!(matches!(parse_key("del"), Ok(Key::Delete)));
    }

    #[test]
    fn test_parse_key_home_end() {
        assert!(matches!(parse_key("home"), Ok(Key::Home)));
        assert!(matches!(parse_key("end"), Ok(Key::End)));
    }

    #[test]
    fn test_parse_key_page_up_down() {
        assert!(matches!(parse_key("pageup"), Ok(Key::PageUp)));
        assert!(matches!(parse_key("pagedown"), Ok(Key::PageDown)));
    }

    #[test]
    fn test_parse_key_single_char() {
        assert!(matches!(parse_key("a"), Ok(Key::Unicode('a'))));
        assert!(matches!(parse_key("z"), Ok(Key::Unicode('z'))));
        assert!(matches!(parse_key("1"), Ok(Key::Unicode('1'))));
        assert!(matches!(parse_key("0"), Ok(Key::Unicode('0'))));
    }

    #[test]
    fn test_parse_key_case_insensitive_single_char() {
        // Uppercase letter input maps to lowercase Unicode
        assert!(matches!(parse_key("A"), Ok(Key::Unicode('a'))));
        assert!(matches!(parse_key("Z"), Ok(Key::Unicode('z'))));
    }

    #[test]
    fn test_parse_key_invalid() {
        let result = parse_key("nonexistentkey");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Unknown key: 'nonexistentkey'"),
            "Error should mention the invalid key: {err}"
        );
        assert!(
            err.contains("Valid keys:"),
            "Error should list valid keys: {err}"
        );
        assert!(
            err.contains("enter"),
            "Error should include 'enter' in valid keys: {err}"
        );
        assert!(
            err.contains("tab"),
            "Error should include 'tab' in valid keys: {err}"
        );
    }

    #[test]
    fn test_parse_key_invalid_multi_char() {
        let result = parse_key("invalidkey123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown key"));
        assert!(err.contains("Valid keys:"));
    }

    // ---- Modifier detection tests ----

    #[test]
    fn test_is_modifier() {
        assert!(is_modifier("ctrl"));
        assert!(is_modifier("Ctrl"));
        assert!(is_modifier("CTRL"));
        assert!(is_modifier("alt"));
        assert!(is_modifier("shift"));
        assert!(is_modifier("super"));
        assert!(is_modifier("meta"));
    }

    #[test]
    fn test_is_not_modifier() {
        assert!(!is_modifier("enter"));
        assert!(!is_modifier("a"));
        assert!(!is_modifier("f1"));
        assert!(!is_modifier("space"));
        assert!(!is_modifier("tab"));
    }

    // ---- Valid key list tests ----

    #[test]
    fn test_valid_key_list_contents() {
        let list = valid_key_list();
        assert!(list.contains("enter"), "Should contain enter: {list}");
        assert!(list.contains("tab"), "Should contain tab: {list}");
        assert!(list.contains("escape"), "Should contain escape: {list}");
        assert!(list.contains("f1"), "Should contain f1: {list}");
        assert!(list.contains("f12"), "Should contain f12: {list}");
        assert!(list.contains("a-z"), "Should contain a-z: {list}");
        assert!(list.contains("0-9"), "Should contain 0-9: {list}");
    }
}
