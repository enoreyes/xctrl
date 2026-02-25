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

## Flow Validator Guidance: Display

Test display commands (screenshot, info, list) via CLI. These are stateless read-only operations (except screenshot which writes a file).

**Xvfb setup:** Xvfb is already running on display :99. Always prefix commands with `DISPLAY=:99`.

**Isolation rules:**
- Display info/list commands are read-only and fully safe to parallelize
- Screenshot commands write to specified output files. Each subagent MUST use unique output file paths (e.g., `/tmp/test_display_<subagent_id>_screenshot.png`)
- No shared mutable state between display commands

**Testing patterns:**
- `DISPLAY=:99 cargo run -- display info --json` → check JSON has width, height, scale_factor keys
- `DISPLAY=:99 cargo run -- display list --json` → check JSON is array with at least one monitor
- `DISPLAY=:99 cargo run -- display screenshot --output /tmp/test.png` → verify file exists and is valid PNG
- `DISPLAY=:99 cargo run -- display screenshot --output /tmp/region.png --x 0 --y 0 --width 100 --height 100` → verify file and dimensions
- `cargo run -- display screenshot --output /nonexistent/dir/shot.png` → verify non-zero exit code and error message

**Validating PNG files:** Use `python3 -c "import struct; f=open('/tmp/test.png','rb'); h=f.read(8); print('Valid PNG' if h[:8]==b'\\x89PNG\\r\\n\\x1a\\n' else 'Not PNG')"` to verify PNG header.

**Validating image dimensions:** Use `python3 -c "import struct; f=open('/tmp/region.png','rb'); f.read(8); f.read(4); ihdr=f.read(4); w,h=struct.unpack('>II',f.read(8)); print(f'Dimensions: {w}x{h}')"` to check PNG dimensions.

## Flow Validator Guidance: Window Management

Test window management commands (list, focus, resize, move, minimize, maximize, fullscreen) via CLI. These commands require a running X11 display with a window manager.

**Environment setup (already done for you):**
- Xvfb is running on display :99
- Metacity window manager is running on :99 (required for _NET_CLIENT_LIST to work)
- Two xterm windows are running with PIDs available via `xctrl window list --json`
- Always prefix commands with `DISPLAY=:99`

**IMPORTANT:** xterm window titles are set by the shell (PS1), not the -T flag. The title typically shows the user@host:path. Use `xctrl window list --json` to get the actual window ID and title to use for targeting.

**Isolation rules:**
- Window operations modify shared X11 window state. If running in parallel, each subagent should operate on DIFFERENT windows identified by ID (not title, since titles may be identical).
- Focus operations change the _NET_ACTIVE_WINDOW which is shared state — coordinate focus tests carefully.
- Resize/move are per-window and safe if targeting different windows.

**Testing patterns:**
1. **List (VAL-WIN-001):**
   ```bash
   DISPLAY=:99 cargo run -- window list --json
   # Should return array with at least 2 windows with title, id, x, y, width, height, pid
   ```

2. **Focus (VAL-WIN-002):**
   ```bash
   DISPLAY=:99 cargo run -- window focus --id <window_id>
   # Should exit 0
   ```

3. **Resize (VAL-WIN-003):**
   ```bash
   DISPLAY=:99 cargo run -- window resize --id <window_id> --width 800 --height 600
   # Should exit 0, verify with window list showing updated dimensions
   ```

4. **Move (VAL-WIN-004):**
   ```bash
   DISPLAY=:99 cargo run -- window move --id <window_id> --x 100 --y 100
   # Should exit 0, verify with window list showing updated position
   ```

5. **Minimize/Maximize/Fullscreen (VAL-WIN-005..007):**
   ```bash
   DISPLAY=:99 cargo run -- window minimize --id <window_id>
   DISPLAY=:99 cargo run -- window maximize --id <window_id>
   DISPLAY=:99 cargo run -- window fullscreen --id <window_id>
   # All should exit 0
   ```

6. **Window not found (VAL-WIN-008):**
   ```bash
   DISPLAY=:99 cargo run -- window focus --title "ThisWindowDoesNotExist_12345"
   # Should exit non-zero with "window not found" error
   ```

## Flow Validator Guidance: OS Actions

Test OS action commands (open-url, open-app, notify, frontmost-app, list-apps) via CLI. Some commands have limited functionality in headless environments.

**Environment setup (already done for you):**
- Xvfb on :99 with metacity running
- xterm windows running for frontmost-app detection
- Always prefix commands with `DISPLAY=:99`

**Headless limitations:**
- `open-url` may fail or succeed depending on whether xdg-open/browser is available. The assertion only requires exit code 0 — but in headless, a browser may not be installed. Accept a clear error if no browser is configured.
- `open-app` uses exec-based approach on Linux. It may or may not succeed depending on the app. Test with a known-available app.
- `notify` sends desktop notifications via libnotify. In headless, notification daemon may not be running. Accept exit code 0 (notification sent) or a clear error.

**Isolation rules:**
- OS commands are mostly stateless (list-apps, frontmost-app are read-only)
- open-url and open-app launch processes — each subagent should clean up any launched processes
- notify is fire-and-forget

**Testing patterns:**
1. **Open URL (VAL-OS-001):**
   ```bash
   DISPLAY=:99 cargo run -- os open-url "https://example.com"
   # Check exit code — may succeed or give a clear error about missing browser
   ```

2. **Open application (VAL-OS-002):**
   ```bash
   DISPLAY=:99 cargo run -- os open-app "xterm"
   # Should exit 0 — xterm is available
   ```

3. **Notify (VAL-OS-003):**
   ```bash
   DISPLAY=:99 cargo run -- os notify --title "Hello" --body "World"
   # Should exit 0 if notification system available
   ```

4. **Frontmost app (VAL-OS-004):**
   ```bash
   DISPLAY=:99 cargo run -- os frontmost-app --json
   # Should return JSON with name and pid fields
   ```

5. **List apps (VAL-OS-005):**
   ```bash
   DISPLAY=:99 cargo run -- os list-apps --json
   # Should return JSON array of app objects with name and pid
   ```

6. **Non-existent app (VAL-OS-006):**
   ```bash
   DISPLAY=:99 cargo run -- os open-app "NonExistentApp_12345"
   # Should exit non-zero with clear error
   ```

## Flow Validator Guidance: Screen Recording

Test screen recording lifecycle (start, status, stop) via CLI. Recording commands are stateful - they manage a background FFmpeg process and a state file.

**Xvfb setup:** Xvfb is already running on display :99. Always prefix commands with `DISPLAY=:99` for start/status (stop reads from state file).

**CRITICAL isolation rules:**
- Screen recording uses a SHARED state file (`~/.xctrl/recording.json`) 
- Only ONE subagent may test recording at a time - recording tests MUST be sequential
- Each test run should clean up: ensure recording is stopped before starting a new one
- Use unique output file paths for each test: `/tmp/test_rec_<test_name>.mp4`

**Testing patterns:**
1. **Full lifecycle (VAL-REC-001, VAL-REC-002, VAL-CROSS-004):**
   ```bash
   DISPLAY=:99 cargo run -- screen record start --output /tmp/test_rec.mp4
   # Check exit code 0
   DISPLAY=:99 cargo run -- screen record status --json
   # Verify {"recording": true, "output": "/tmp/test_rec.mp4", "pid": N}
   sleep 3  # Let it record for a few seconds
   DISPLAY=:99 cargo run -- screen record stop
   # Verify exit code 0, file exists with size > 0
   ```

2. **Cold status (VAL-REC-008):**
   ```bash
   # Ensure no recording is active first
   DISPLAY=:99 cargo run -- screen record status --json
   # Verify {"recording": false} and exit code 0
   ```

3. **Double start (VAL-REC-006):**
   ```bash
   DISPLAY=:99 cargo run -- screen record start --output /tmp/rec1.mp4
   DISPLAY=:99 cargo run -- screen record start --output /tmp/rec2.mp4
   # Second should fail with "recording already in progress"
   DISPLAY=:99 cargo run -- screen record stop  # cleanup
   ```

4. **Stop when not recording (VAL-REC-007):**
   ```bash
   DISPLAY=:99 cargo run -- screen record stop
   # Should fail with "no active recording" message
   ```

5. **FFmpeg missing (VAL-REC-005):**
   ```bash
   PATH=/usr/bin:/bin cargo run -- screen record start --output /tmp/rec.mp4
   # Temporarily hide ffmpeg from PATH and check error message
   ```

6. **Headless auto-Xvfb (VAL-REC-004):**
   ```bash
   # Unset DISPLAY and test auto-detection
   unset DISPLAY
   cargo run -- screen record start --output /tmp/headless_rec.mp4
   # Should auto-start Xvfb and begin recording
   sleep 3
   cargo run -- screen record stop
   ```

**FFmpeg location:** FFmpeg is installed at `/usr/local/bin/ffmpeg` (or discoverable via `which ffmpeg`).

**Cleanup:** Always stop any active recording at the end of tests. Check state file and kill orphaned FFmpeg processes if needed.
