# Architecture

Architectural decisions, patterns, and conventions for xctrl.

**What belongs here:** Design decisions, module structure, cross-cutting concerns, platform abstraction strategy.

---

## CLI Structure
```
xctrl <primitive> <action> [options]
```

Primitives: mouse, keyboard, clipboard, display, screen, window, os

## Module Layout
```
src/
├── main.rs          # Entry point, clap CLI definition
├── cli.rs           # CLI argument parsing (clap derive)
├── output.rs        # JSON/text output formatting
├── error.rs         # Error types and display
├── mouse.rs         # Mouse primitive (inline #[cfg] for platform code)
├── keyboard.rs      # Keyboard primitive
├── clipboard.rs     # Clipboard primitive (inline #[cfg] for platform code)
├── display.rs       # Display/screenshot primitive
├── screen.rs        # Screen recording primitive (inline #[cfg] for platform code)
├── window.rs        # Window management primitive
└── os_actions.rs    # OS-level actions primitive
```

**Note:** Platform-specific code uses inline `#[cfg(target_os = "...")]` blocks within each module rather than a separate `platform/` directory.

## Key Crates
| Function | Crate | Notes |
|----------|-------|-------|
| CLI | `clap` (derive) | Subcommand per primitive, nested subcommands per action |
| Mouse/KB | `enigo` 0.6.x | Cross-platform. Linux needs `xdo` feature + libxdo-devel |
| Clipboard | `arboard` 3.6.x | Cross-platform. 1Password-backed |
| Screenshots | `xcap` 0.8.x | Cross-platform. Returns `image::RgbaImage` |
| Screen recording | FFmpeg process | Spawned via `std::process::Command` |
| Window listing (Linux) | `x11rb` | Uses _NET_CLIENT_LIST EWMH property (no separate x-win dependency) |
| Window control (Linux) | `x11rb` | EWMH/ICCCM protocols |
| Window control (macOS) | `core-graphics` + accessibility | AXUIElement API |
| Window control (Win) | `windows` crate | Win32 API |
| Open URLs/apps | `open` | Cross-platform |
| Notifications | `notify-rust` | Cross-platform |
| JSON | `serde` + `serde_json` | All output structs derive Serialize |

## Platform Abstraction
- Use `#[cfg(target_os = "linux")]`, `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "windows")]`
- Prefer cross-platform crates (enigo, arboard, xcap, x-win) over platform-specific code
- Platform-specific code uses inline `#[cfg(target_os)]` gates within each module (e.g., `screen.rs`, `clipboard.rs`, `mouse.rs`) rather than a separate `src/platform/` directory. This is the established convention — all existing primitives follow this pattern.
- Window control is the most platform-specific area

## JSON Output Convention
All commands that return data support `--json`. Without the flag, output is human-readable text. With the flag, output is valid JSON to stdout. Errors always go to stderr.

## Error Handling
- Use `exit_with_error(message, hint, json_mode)` for user-facing errors — prints to stderr and exits 1
- Display errors include remediation hints (e.g., missing ffmpeg)
- Exit codes: 0 = success, 1 = all errors (including missing dependencies)
- Do NOT use anyhow::Result in command handlers — the exit_with_error pattern is the established convention
