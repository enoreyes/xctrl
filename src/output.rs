use crate::error::{exit_with_error, XctrlError};
use serde::Serialize;

/// Print a success message to stdout.
/// In JSON mode, serializes the value as JSON.
/// In text mode, prints the human-readable text.
#[allow(dead_code)]
pub fn print_success<T: Serialize>(value: &T, text: &str, json: bool) {
    if json {
        match serde_json::to_string_pretty(value) {
            Ok(json_str) => println!("{json_str}"),
            Err(e) => eprintln!("Error serializing JSON: {e}"),
        }
    } else {
        println!("{text}");
    }
}

/// Print a plain text message (for commands that don't return structured data).
#[allow(dead_code)]
pub fn print_text(text: &str) {
    println!("{text}");
}

/// Print a "not yet implemented" message and exit with code 1.
pub fn not_yet_implemented(command: &str, json: bool) -> ! {
    let err = XctrlError::with_hint(
        format!("{command}: not yet implemented"),
        "This command is a stub and will be implemented in a future release.",
    );
    exit_with_error(&err, json, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        x: i32,
        y: i32,
    }

    #[test]
    fn test_print_success_text_mode() {
        // Just verify it doesn't panic (output goes to stdout)
        let data = TestData { x: 10, y: 20 };
        print_success(&data, "Position: (10, 20)", false);
    }

    #[test]
    fn test_print_success_json_mode() {
        let data = TestData { x: 10, y: 20 };
        print_success(&data, "Position: (10, 20)", true);
    }

    #[test]
    fn test_print_text() {
        print_text("hello world");
    }
}
