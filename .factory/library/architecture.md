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
├── mouse.rs         # Mouse primitive
├── keyboard.rs      # Keyboard primitive
├── clipboard.rs     # Clipboard primitive
├── display.rs       # Display/screenshot primitive
├── screen.rs        # Screen recording primitive
├── window.rs        # Window management primitive
├── os_actions.rs    # OS-level actions primitive
└── platform/        # Platform-specific implementations
    ├── mod.rs
    ├── linux.rs
    ├── macos.rs
    └── windows.rs
```

## Key Crates
| Function | Crate | Notes |
|----------|-------|-------|
| CLI | `clap` (derive) | Subcommand per primitive, nested subcommands per action |
| Mouse/KB | `enigo` 0.6.x | Cross-platform. Linux needs `xdo` feature + libxdo-devel |
| Clipboard | `arboard` 3.6.x | Cross-platform. 1Password-backed |
| Screenshots | `xcap` 0.8.x | Cross-platform. Returns `image::RgbaImage` |
| Screen recording | FFmpeg process | Spawned via `std::process::Command` |
| Window listing | `x-win` 5.x | Cross-platform read-only window info |
| Window control (Linux) | `x11rb` | EWMH/ICCCM protocols |
| Window control (macOS) | `core-graphics` + accessibility | AXUIElement API |
| Window control (Win) | `windows` crate | Win32 API |
| Open URLs/apps | `open` | Cross-platform |
| Notifications | `notify-rust` | Cross-platform |
| JSON | `serde` + `serde_json` | All output structs derive Serialize |

## Platform Abstraction
- Use `#[cfg(target_os = "linux")]`, `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "windows")]`
- Prefer cross-platform crates (enigo, arboard, xcap, x-win) over platform-specific code
- Platform-specific code goes in `src/platform/` module
- Window control is the most platform-specific area

## JSON Output Convention
All commands that return data support `--json`. Without the flag, output is human-readable text. With the flag, output is valid JSON to stdout. Errors always go to stderr.

## Error Handling
- Use `anyhow` for error propagation
- Display errors include remediation hints (e.g., missing ffmpeg)
- Exit codes: 0 = success, 1 = error, 2 = missing dependency
