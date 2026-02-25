use crate::cli::DisplayAction;
use crate::error::{exit_with_error, XctrlError};
use crate::output::print_success;
use image::GenericImageView;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct DisplayInfo {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

#[derive(Serialize)]
pub struct MonitorInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f32,
}

pub fn handle_display(action: DisplayAction, json: bool) {
    match action {
        DisplayAction::Screenshot {
            output,
            x,
            y,
            width,
            height,
        } => handle_screenshot(&output, x, y, width, height, json),
        DisplayAction::Info => handle_info(json),
        DisplayAction::List => handle_list(json),
    }
}

fn handle_screenshot(
    output: &str,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    json: bool,
) {
    // Validate output directory exists
    let output_path = Path::new(output);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let err = XctrlError::with_hint(
                format!("Output directory does not exist: {}", parent.display()),
                "Create the directory first or specify a path in an existing directory.",
            );
            exit_with_error(&err, json, 1);
        }
    }

    // Validate region dimensions if specified
    let has_region = x.is_some() || y.is_some() || width.is_some() || height.is_some();
    if has_region {
        if let Some(w) = width {
            if w == 0 {
                let err = XctrlError::with_hint(
                    "Invalid region: width must be greater than zero",
                    "Specify a positive width value for the capture region.",
                );
                exit_with_error(&err, json, 1);
            }
        }
        if let Some(h) = height {
            if h == 0 {
                let err = XctrlError::with_hint(
                    "Invalid region: height must be greater than zero",
                    "Specify a positive height value for the capture region.",
                );
                exit_with_error(&err, json, 1);
            }
        }
    }

    // Capture screenshot from primary monitor
    let monitors = match xcap::Monitor::all() {
        Ok(m) => m,
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("Failed to enumerate monitors: {e}"),
                "Ensure a display is available. On headless Linux, start Xvfb first.",
            );
            exit_with_error(&err, json, 1);
        }
    };

    if monitors.is_empty() {
        let err = XctrlError::with_hint(
            "No monitors found",
            "Ensure a display is available. On headless Linux, start Xvfb first.",
        );
        exit_with_error(&err, json, 1);
    }

    let primary = &monitors[0];
    let image = match primary.capture_image() {
        Ok(img) => img,
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("Failed to capture screenshot: {e}"),
                "Ensure a display is available and accessible.",
            );
            exit_with_error(&err, json, 1);
        }
    };

    // Crop if region specified
    let final_image: image::DynamicImage = if has_region {
        let rx = x.unwrap_or(0);
        let ry = y.unwrap_or(0);
        let rw = width.unwrap_or(image.width().saturating_sub(rx));
        let rh = height.unwrap_or(image.height().saturating_sub(ry));

        // Validate region is within image bounds
        let (img_w, img_h) = image.dimensions();
        if rx >= img_w || ry >= img_h {
            let err = XctrlError::with_hint(
                format!(
                    "Region origin ({rx}, {ry}) is outside the screen bounds ({img_w}x{img_h})"
                ),
                "Specify coordinates within the screen dimensions.",
            );
            exit_with_error(&err, json, 1);
        }

        // Clamp region to image bounds
        let clamped_w = rw.min(img_w.saturating_sub(rx));
        let clamped_h = rh.min(img_h.saturating_sub(ry));

        if clamped_w == 0 || clamped_h == 0 {
            let err = XctrlError::with_hint(
                "Region results in zero-size image after clamping to screen bounds",
                "Adjust the region coordinates and dimensions.",
            );
            exit_with_error(&err, json, 1);
        }

        let dynamic = image::DynamicImage::ImageRgba8(image);
        dynamic.crop_imm(rx, ry, clamped_w, clamped_h)
    } else {
        image::DynamicImage::ImageRgba8(image)
    };

    // Save to file
    if let Err(e) = final_image.save(output) {
        let err = XctrlError::with_hint(
            format!("Failed to save screenshot to {output}: {e}"),
            "Check that the output path is writable and the directory exists.",
        );
        exit_with_error(&err, json, 1);
    }

    if json {
        #[derive(Serialize)]
        struct ScreenshotResult {
            path: String,
            width: u32,
            height: u32,
        }
        let (w, h) = final_image.dimensions();
        let result = ScreenshotResult {
            path: output.to_string(),
            width: w,
            height: h,
        };
        print_success(&result, "", json);
    } else {
        let (w, h) = final_image.dimensions();
        println!("Screenshot saved to {output} ({w}x{h})");
    }
}

fn handle_info(json: bool) {
    let monitors = match xcap::Monitor::all() {
        Ok(m) => m,
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("Failed to enumerate monitors: {e}"),
                "Ensure a display is available. On headless Linux, start Xvfb first.",
            );
            exit_with_error(&err, json, 1);
        }
    };

    if monitors.is_empty() {
        let err = XctrlError::with_hint(
            "No monitors found",
            "Ensure a display is available. On headless Linux, start Xvfb first.",
        );
        exit_with_error(&err, json, 1);
    }

    let primary = &monitors[0];
    let width = monitor_width(primary, json);
    let height = monitor_height(primary, json);
    let scale = monitor_scale_factor(primary, json);
    let info = DisplayInfo {
        width,
        height,
        scale_factor: scale,
    };

    let text = format!(
        "Resolution: {}x{}\nScale factor: {}",
        info.width, info.height, info.scale_factor
    );
    print_success(&info, &text, json);
}

fn handle_list(json: bool) {
    let monitors = match xcap::Monitor::all() {
        Ok(m) => m,
        Err(e) => {
            let err = XctrlError::with_hint(
                format!("Failed to enumerate monitors: {e}"),
                "Ensure a display is available. On headless Linux, start Xvfb first.",
            );
            exit_with_error(&err, json, 1);
        }
    };

    if monitors.is_empty() {
        let err = XctrlError::with_hint(
            "No monitors found",
            "Ensure a display is available. On headless Linux, start Xvfb first.",
        );
        exit_with_error(&err, json, 1);
    }

    let monitor_list: Vec<MonitorInfo> = monitors
        .iter()
        .map(|m| MonitorInfo {
            name: monitor_name(m, json),
            width: monitor_width(m, json),
            height: monitor_height(m, json),
            x: monitor_x(m, json),
            y: monitor_y(m, json),
            scale_factor: monitor_scale_factor(m, json),
        })
        .collect();

    if json {
        match serde_json::to_string_pretty(&monitor_list) {
            Ok(json_str) => println!("{json_str}"),
            Err(e) => {
                let err = XctrlError::new(format!("Failed to serialize monitor list: {e}"));
                exit_with_error(&err, json, 1);
            }
        }
    } else {
        for (i, m) in monitor_list.iter().enumerate() {
            println!(
                "Monitor {}: {} ({}x{} at ({}, {}), scale: {})",
                i, m.name, m.width, m.height, m.x, m.y, m.scale_factor
            );
        }
    }
}

// Helper functions to safely extract monitor properties from xcap::Monitor.
// The xcap 0.8.x API returns Result types for these properties.

fn monitor_name(m: &xcap::Monitor, json: bool) -> String {
    match m.name() {
        Ok(name) => name,
        Err(e) => {
            let err = XctrlError::new(format!("Failed to get monitor name: {e}"));
            exit_with_error(&err, json, 1);
        }
    }
}

fn monitor_width(m: &xcap::Monitor, json: bool) -> u32 {
    match m.width() {
        Ok(w) => w,
        Err(e) => {
            let err = XctrlError::new(format!("Failed to get monitor width: {e}"));
            exit_with_error(&err, json, 1);
        }
    }
}

fn monitor_height(m: &xcap::Monitor, json: bool) -> u32 {
    match m.height() {
        Ok(h) => h,
        Err(e) => {
            let err = XctrlError::new(format!("Failed to get monitor height: {e}"));
            exit_with_error(&err, json, 1);
        }
    }
}

fn monitor_x(m: &xcap::Monitor, json: bool) -> i32 {
    match m.x() {
        Ok(x) => x,
        Err(e) => {
            let err = XctrlError::new(format!("Failed to get monitor x position: {e}"));
            exit_with_error(&err, json, 1);
        }
    }
}

fn monitor_y(m: &xcap::Monitor, json: bool) -> i32 {
    match m.y() {
        Ok(y) => y,
        Err(e) => {
            let err = XctrlError::new(format!("Failed to get monitor y position: {e}"));
            exit_with_error(&err, json, 1);
        }
    }
}

fn monitor_scale_factor(m: &xcap::Monitor, json: bool) -> f32 {
    match m.scale_factor() {
        Ok(s) => s,
        Err(e) => {
            let err = XctrlError::new(format!("Failed to get monitor scale factor: {e}"));
            exit_with_error(&err, json, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_info_serialization() {
        let info = DisplayInfo {
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["width"], 1920);
        assert_eq!(json["height"], 1080);
        assert_eq!(json["scale_factor"], 1.0);
    }

    #[test]
    fn test_display_info_snake_case_keys() {
        let info = DisplayInfo {
            width: 1920,
            height: 1080,
            scale_factor: 2.0,
        };
        let json_str = serde_json::to_string(&info).unwrap();
        assert!(
            json_str.contains("scale_factor"),
            "should use snake_case: {json_str}"
        );
        assert!(
            !json_str.contains("scaleFactor"),
            "should not use camelCase: {json_str}"
        );
    }

    #[test]
    fn test_monitor_info_serialization() {
        let monitor = MonitorInfo {
            name: "HDMI-1".to_string(),
            width: 2560,
            height: 1440,
            x: 0,
            y: 0,
            scale_factor: 1.5,
        };
        let json = serde_json::to_value(&monitor).unwrap();
        assert_eq!(json["name"], "HDMI-1");
        assert_eq!(json["width"], 2560);
        assert_eq!(json["height"], 1440);
        assert_eq!(json["x"], 0);
        assert_eq!(json["y"], 0);
        assert_eq!(json["scale_factor"], 1.5);
    }

    #[test]
    fn test_monitor_info_list_serialization() {
        let monitors = vec![
            MonitorInfo {
                name: "primary".to_string(),
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                scale_factor: 1.0,
            },
            MonitorInfo {
                name: "secondary".to_string(),
                width: 2560,
                height: 1440,
                x: 1920,
                y: 0,
                scale_factor: 1.0,
            },
        ];
        let json = serde_json::to_value(&monitors).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["name"], "primary");
        assert_eq!(json[1]["name"], "secondary");
    }
}
