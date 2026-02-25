# User Testing

Testing surface, tools, setup steps, and known quirks for manual/automated testing.

**What belongs here:** How to test xctrl commands, what works in headless, known limitations.

---

## Testing Surface
xctrl is a CLI tool. All testing is done by executing commands and checking:
1. Exit codes (0 = success, non-zero = error)
2. Stdout output (text or JSON)
3. Stderr for error messages
4. Side effects (files created, clipboard changed, etc.)

## Testing in Headless Linux (This Environment)
This environment is headless Amazon Linux 2023. To test display-related features:

1. Start Xvfb: `Xvfb :99 -screen 0 1920x1080x24 -ac &`
2. Set DISPLAY: `export DISPLAY=:99`
3. Now mouse, keyboard, display, window commands work against the virtual display

## What Can Be Tested Locally
- ✅ CLI argument parsing (all commands, --help, --json)
- ✅ Clipboard operations (with Xvfb for X11 clipboard)
- ✅ Mouse position/move (with Xvfb)
- ✅ Keyboard type/press (with Xvfb, no visual verification)
- ✅ Screenshots (with Xvfb, produces image of virtual display)
- ✅ Screen recording (with Xvfb + ffmpeg)
- ✅ Display info (with Xvfb)
- ⚠️ Window management (limited - need apps running in Xvfb)
- ⚠️ OS actions (open-url/open-app may not work fully in headless)
- ❌ macOS/Windows-specific code paths (compile-check only)

## Testing Commands
```bash
# Build
cargo build

# Run all tests
cargo test -- --test-threads=2

# Run specific test
cargo test test_name

# Lint
cargo clippy -- -D warnings
cargo fmt --check

# Manual CLI testing
cargo run -- --help
cargo run -- mouse position --json
cargo run -- clipboard set "test"
cargo run -- clipboard get
```

## Known Quirks
- Xvfb virtual display appears as a single 1920x1080 monitor
- Mouse events in Xvfb work but there's nothing visual to click on
- Clipboard requires a running X server (Xvfb counts)
- Screen recording in Xvfb records a blank/static display unless apps are running

## Flow Validator Guidance: CLI

xctrl is a stateless CLI tool. Each command invocation is independent. Testing is done by:
1. Running `cargo run -- <args>` from `/home/ec2-user/code/work/xctrl`
2. Checking exit code
3. Checking stdout/stderr output
4. For JSON mode, piping through `python3 -m json.tool` to validate

**Xvfb setup:** Xvfb is already running on display :99. Always set `DISPLAY=:99` before running display-dependent commands (mouse, keyboard, clipboard, display, window).

**Isolation rules for parallel subagents:**
- CLI framework tests (--help, --version, error handling) don't affect shared state - fully safe to parallelize
- Mouse tests use the shared Xvfb cursor - run mouse position verification quickly to avoid races with other subagents. Each mouse subagent should own the cursor exclusively for its test window.
- Clipboard tests share a single X11 clipboard - ONLY ONE subagent should test clipboard at a time, or clipboard tests should be in a single subagent group.
- Keyboard tests produce keystrokes into Xvfb - safe in isolation since there's no target window to receive them

**Binary path:** Use `cargo run --` for all tests (from the repo root). This ensures the latest build is used.

**Environment:** Source `$HOME/.cargo/env` before running cargo commands if needed.
