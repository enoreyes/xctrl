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

**Linux note:** enigo's default x11rb backend works well. The `xdo` feature requires `libxdo-devel` which is not available on Amazon Linux 2023.
**IMPORTANT:** enigo's `move_mouse` uses xtest_fake_input which does NOT reliably persist cursor position in Xvfb. For reliable cursor movement on headless Linux, use `x11rb`'s `warp_pointer` directly instead.
**Missing key variants:** enigo 0.6.x lacks some key variants (e.g., `Key::Insert`). Use `Key::Other(keysym)` with X11 keysym values as a workaround on Linux (e.g., `Key::Other(0xff63)` for Insert/XK_Insert). These keysym values are X11-specific and will need `#[cfg(target_os)]` gates for cross-platform support.

## arboard 3.6.x (Clipboard)
```rust
use arboard::Clipboard;

let mut clipboard = Clipboard::new()?;
clipboard.set_text("hello")?;
let text = clipboard.get_text()?;
clipboard.clear()?;
```

**Linux:** Requires X11 display (Xvfb works). Add `wayland-data-control` feature for Wayland.
**X11 clipboard ownership:** On X11, clipboard data is owned by the process that set it. When that process exits, the clipboard data is lost. For CLI tools where `set` and `get` are separate process invocations, use the `SetExtLinux` extension with `LinuxClipboardKind::Clipboard` and `.wait()` to spawn a daemon that holds clipboard ownership:
```rust
use arboard::{Clipboard, SetExtLinux};
clipboard.set().clipboard(arboard::LinuxClipboardKind::Clipboard).wait().text("hello")?;
```
The daemon process blocks until another process overwrites the clipboard. For the xctrl CLI, this is implemented by spawning the current binary as a detached child process with a `--daemon` flag.

## xcap 0.8.x (Screenshots)
```rust
use xcap::Monitor;

let monitors = Monitor::all()?;
let primary = &monitors[0];
let image = primary.capture_image()?; // Returns image::RgbaImage

// Save to file
image.save("screenshot.png")?;

// Monitor info — NOTE: all property accessors return Result, not direct values
let name = primary.name()?;       // Result<String>
let width = primary.width()?;     // Result<u32>
let height = primary.height()?;   // Result<u32>
let x = primary.x()?;             // Result<i32>
let y = primary.y()?;             // Result<i32>
let scale = primary.scale_factor()?; // Result<f32>
```

**IMPORTANT:** xcap 0.8.x monitor property methods (`name()`, `width()`, `height()`, `x()`, `y()`, `scale_factor()`) all return `Result` types, not direct values. Wrap each call with `?` or handle errors explicitly. The display-primitive worker created helper wrapper functions for this pattern.

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
