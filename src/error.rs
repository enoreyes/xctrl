use serde::Serialize;
use std::fmt;
use std::process;

/// Structured error with remediation hints.
#[derive(Debug, Serialize)]
pub struct XctrlError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl fmt::Display for XctrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.error)?;
        if let Some(hint) = &self.hint {
            write!(f, "\nHint: {hint}")?;
        }
        Ok(())
    }
}

impl XctrlError {
    #[allow(dead_code)]
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            hint: None,
        }
    }

    pub fn with_hint(error: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            hint: Some(hint.into()),
        }
    }
}

/// Print an error and exit with the given code.
/// In JSON mode, prints JSON to stderr. Otherwise, prints plain text to stderr.
pub fn exit_with_error(err: &XctrlError, json: bool, code: i32) -> ! {
    if json {
        let json_str = serde_json::to_string(err)
            .unwrap_or_else(|_| format!(r#"{{"error":"{}"}}"#, err.error));
        eprintln!("{json_str}");
    } else {
        eprintln!("{err}");
    }
    process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_without_hint() {
        let err = XctrlError::new("something went wrong");
        assert_eq!(format!("{err}"), "Error: something went wrong");
    }

    #[test]
    fn test_error_display_with_hint() {
        let err =
            XctrlError::with_hint("ffmpeg not found", "Install with: sudo dnf install ffmpeg");
        let display = format!("{err}");
        assert!(display.contains("Error: ffmpeg not found"));
        assert!(display.contains("Hint: Install with: sudo dnf install ffmpeg"));
    }

    #[test]
    fn test_error_json_serialization() {
        let err = XctrlError::with_hint("bad input", "try again");
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["error"], "bad input");
        assert_eq!(json["hint"], "try again");
    }

    #[test]
    fn test_error_json_no_hint_omits_field() {
        let err = XctrlError::new("something went wrong");
        let json_str = serde_json::to_string(&err).unwrap();
        assert!(
            !json_str.contains(r#""hint""#),
            "JSON should not contain hint field: {json_str}"
        );
    }
}
