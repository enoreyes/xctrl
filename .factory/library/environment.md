# Environment

Environment variables, external dependencies, and setup notes.

**What belongs here:** Required env vars, external API keys/services, dependency quirks, platform-specific notes.
**What does NOT belong here:** Service ports/commands (use `.factory/services.yaml`).

---

## Build Environment
- Amazon Linux 2023 (x86_64)
- Rust stable toolchain (installed via rustup)
- 4 CPU cores, 15GB RAM

## System Dependencies (Linux)
- `libxdo-devel` — Required by `enigo` crate for X11 input simulation
- `libX11-devel`, `libxcb-devel` — X11 development headers
- `libXtst-devel` — XTest extension for input events
- `xorg-x11-server-Xvfb` — Virtual framebuffer for headless testing
- `ffmpeg` — Required for screen recording
- `gcc`, `pkg-config` — Build essentials

## External Tools
- `ffmpeg` — Must be on PATH for screen recording. Platform-specific capture devices:
  - Linux: `-f x11grab -i :DISPLAY`
  - macOS: `-f avfoundation -i "1:none"`
  - Windows: `-f gdigrab -i desktop`
- `Xvfb` — Virtual framebuffer for headless Linux. Started automatically by xctrl when no DISPLAY detected.
- `gh` — GitHub CLI for pushing code and managing CI workflows.
