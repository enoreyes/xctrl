# Rust Crates Reference

API patterns and usage notes for key dependencies.

---

## enigo 0.6.x (Mouse/Keyboard)
```rust
use enigo::{Enigo, Keyboard, Mouse, Settings, Coordinate, Button, Key, Direction};

let mut enigo = Enigo::new(&Settings::default()).unwrap();

// Mouse
enigo.move_mouse(500, 300, Coordinate::Abs)?;
enigo.button(Button::Left, Direction::Click)?;
enigo.button(Button::Right, Direction::Click)?;
enigo.scroll(0, -5)?; // horizontal, vertical
// For drag: mouse_down, move, mouse_up

// Keyboard  
enigo.text("Hello world")?;
enigo.key(Key::Return, Direction::Click)?;
enigo.key(Key::Tab, Direction::Click)?;
// Hotkey: key down modifier, press key, key up modifier
enigo.key(Key::Control, Direction::Press)?;
enigo.key(Key::Unicode('c'), Direction::Click)?;
enigo.key(Key::Control, Direction::Release)?;
```

**Linux feature flag:** `enigo = { version = "0.6", features = ["xdo"] }`
**Requires:** `libxdo-devel` system package on Linux

## arboard 3.6.x (Clipboard)
```rust
use arboard::Clipboard;

let mut clipboard = Clipboard::new()?;
clipboard.set_text("hello")?;
let text = clipboard.get_text()?;
clipboard.clear()?;
```

**Linux:** Requires X11 display (Xvfb works). Add `wayland-data-control` feature for Wayland.

## xcap 0.8.x (Screenshots)
```rust
use xcap::Monitor;

let monitors = Monitor::all()?;
let primary = &monitors[0];
let image = primary.capture_image()?; // Returns image::RgbaImage

// Save to file
image.save("screenshot.png")?;

// Monitor info
let name = primary.name();
let width = primary.width();
let height = primary.height();
let x = primary.x();
let y = primary.y();
let scale = primary.scale_factor();
```

**Dependencies:** Also pulls in `image` crate.

## x-win 5.x (Window Listing)
```rust
// Cross-platform window listing
// Check x-win docs for current API
```

## FFmpeg Process Invocation (Screen Recording)
```rust
use std::process::Command;

// Linux (X11/Xvfb)
Command::new("ffmpeg")
    .args(&["-f", "x11grab", "-video_size", "1920x1080", 
            "-framerate", "30", "-i", ":99", 
            "-c:v", "libx264", "-preset", "ultrafast",
            "output.mp4"])
    .spawn()?;

// macOS
// -f avfoundation -framerate 30 -i "1:none"

// Windows
// -f gdigrab -framerate 30 -i desktop
```

## notify-rust (Notifications)
```rust
use notify_rust::Notification;

Notification::new()
    .summary("Title")
    .body("Body text")
    .show()?;
```

## open (Open URLs/Apps)
```rust
open::that("https://example.com")?;  // Opens URL in default browser
open::that("path/to/file")?;         // Opens file with default app
```
