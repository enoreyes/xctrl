# xctrl

**Cross-platform computer control CLI** — a lightweight, OS-agnostic Rust CLI for fine-grained computer control.

Inspired by [mctrl](https://github.com/enoreyes/mctrl) but cross-platform (Linux, macOS, Windows), Rust-based, and focused on basic control primitives rather than high-level app integrations.

All commands follow the pattern: `xctrl <primitive> <action> [options]`

All commands support `--json` for structured output, making it suitable for both human use and AI agent integration.

## Installation

### From source (requires [Rust](https://rustup.rs/))

```bash
cargo install --path .
```

### Build from source

```bash
git clone https://github.com/enoreyes/xctrl.git
cd xctrl
cargo build --release
# Binary will be at target/release/xctrl
```

### System dependencies

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get install libxdo-dev libx11-dev libxcb1-dev libxtst-dev ffmpeg xvfb
```

**Linux (Fedora/Amazon Linux):**
```bash
sudo dnf install libxdo-devel libX11-devel libxcb-devel libXtst-devel ffmpeg xorg-x11-server-Xvfb
```

**macOS / Windows:** No extra dependencies required for most primitives.

## Primitives

### Mouse

Control the mouse cursor — move, click, scroll, drag, and query position.

```bash
# Move cursor to absolute position
xctrl mouse move --x 500 --y 300

# Left click at current position
xctrl mouse click

# Left click at specific coordinates
xctrl mouse click --x 200 --y 150

# Double-click
xctrl mouse double-click

# Right-click
xctrl mouse right-click

# Scroll (positive = up, negative = down)
xctrl mouse scroll --amount 5
xctrl mouse scroll --amount -3

# Drag from one position to another
xctrl mouse drag --from-x 100 --from-y 100 --to-x 500 --to-y 500

# Get current cursor position
xctrl mouse position
xctrl mouse position --json
# {"x": 500, "y": 300}
```

### Keyboard

Simulate keyboard input — type text, press keys, key combinations.

```bash
# Type a text string
xctrl keyboard type "Hello, world!"

# Press a named key
xctrl keyboard press enter
xctrl keyboard press tab
xctrl keyboard press escape
xctrl keyboard press f1

# Key combination (hotkey)
xctrl keyboard hotkey ctrl c
xctrl keyboard hotkey ctrl shift s
xctrl keyboard hotkey alt tab

# Hold a key down
xctrl keyboard key-down shift

# Release a held key
xctrl keyboard key-up shift
```

### Clipboard

Read and write the system clipboard.

```bash
# Set clipboard text
xctrl clipboard set "some text"

# Get clipboard text
xctrl clipboard get
xctrl clipboard get --json
# {"text": "some text"}

# Clear the clipboard
xctrl clipboard clear
```

### Display

Capture screenshots and query display information.

```bash
# Take a screenshot
xctrl display screenshot --output /tmp/screenshot.png

# Take a regional screenshot
xctrl display screenshot --output /tmp/region.png --x 0 --y 0 --width 100 --height 100

# Get display info (resolution, scale)
xctrl display info
xctrl display info --json
# {"width": 1920, "height": 1080, "scale_factor": 1.0}

# List all monitors
xctrl display list
xctrl display list --json
```

### Screen Recording

Record the screen using FFmpeg (must be installed).

```bash
# Start recording
xctrl screen record start --output /tmp/recording.mp4

# Check recording status
xctrl screen record status
xctrl screen record status --json
# {"recording": true, "output": "/tmp/recording.mp4", "pid": 12345}

# Stop recording
xctrl screen record stop
```

On headless Linux systems, xctrl automatically detects the missing display and starts a Xvfb virtual framebuffer for recording.

### Window

Manage application windows — list, focus, resize, move, minimize, maximize, fullscreen.

```bash
# List all windows
xctrl window list
xctrl window list --json

# Focus a window by title
xctrl window focus --title "Firefox"

# Focus a window by ID
xctrl window focus --id 12345

# Resize a window
xctrl window resize --title "Firefox" --width 800 --height 600

# Move a window
xctrl window move --title "Firefox" --x 100 --y 100

# Minimize / maximize / fullscreen
xctrl window minimize --title "Firefox"
xctrl window maximize --title "Firefox"
xctrl window fullscreen --title "Firefox"
```

### OS Actions

OS-level utility actions — open URLs, launch apps, send notifications.

```bash
# Open a URL in the default browser
xctrl os open-url "https://example.com"

# Launch an application
xctrl os open-app "firefox"

# Send a desktop notification
xctrl os notify --title "Hello" --body "World"

# Get the frontmost (focused) application
xctrl os frontmost-app
xctrl os frontmost-app --json

# List running applications
xctrl os list-apps
xctrl os list-apps --json
```

## JSON Output

All data-returning commands support the `--json` flag for structured output:

```bash
xctrl mouse position --json
# {"x": 500, "y": 300}

xctrl clipboard get --json
# {"text": "clipboard content"}

xctrl display info --json
# {"width": 1920, "height": 1080, "scale_factor": 1.0}
```

Errors in JSON mode produce `{"error": "..."}` on stderr.

## Platform Support

| Feature | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Mouse | ✅ | ✅ | ✅ |
| Keyboard | ✅ | ✅ | ✅ |
| Clipboard | ✅ | ✅ | ✅ |
| Display | ✅ | ✅ | ✅ |
| Screen Recording | ✅ | ✅ | ✅ |
| Window Management | ✅ | ✅ | ✅ |
| OS Actions | ✅ | ✅ | ✅ |

## License

MIT
