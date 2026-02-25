---
name: rust-worker
description: Implements Rust features for the xctrl CLI tool using TDD
---

# Rust Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Use for features that involve writing Rust code for the xctrl CLI: new primitives, subcommands, platform-specific implementations, error handling, output formatting, and unit/integration tests.

## Work Procedure

### 1. Understand the Feature

Read the feature description in features.json carefully. Check:
- `preconditions` — what must already exist
- `expectedBehavior` — what success looks like
- `verificationSteps` — how to verify

Read `.factory/library/architecture.md` for module layout conventions and `.factory/library/rust-crates.md` for API patterns.

### 2. Write Tests First (TDD)

Before writing any implementation code:
- Create or update test files in `tests/` (integration tests) or inline `#[cfg(test)]` modules
- Write tests that capture the expected behavior from the feature description
- Tests should cover: happy path, error cases, JSON output format, CLI argument parsing
- Run `cargo test` — confirm tests FAIL (red phase)

For CLI argument parsing tests, test that clap correctly parses all documented flags and subcommands.
For output tests, verify JSON structure matches the contract.

### 3. Implement

- Follow the module layout in `.factory/library/architecture.md`
- Use platform gates: `#[cfg(target_os = "linux")]`, `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "windows")]`
- Use crate APIs from `.factory/library/rust-crates.md`
- All output structs must derive `serde::Serialize`
- Error messages must include remediation hints (e.g., "Install ffmpeg with: ...")
- Keep functions small and testable

### 4. Make Tests Pass (Green Phase)

- Run `cargo test` — all tests must pass
- Fix any failures iteratively
- Do not skip or ignore tests

### 5. Lint and Format

Run these commands and fix all issues:
- `cargo clippy -- -D warnings` (zero warnings)
- `cargo fmt` (auto-format)
- `cargo fmt --check` (verify formatting)

### 6. Manual Verification

Test the CLI manually by running commands:
- `cargo run -- <primitive> <action> [options]`
- Test with `--help` at each level
- Test with `--json` where applicable
- Test error cases (missing args, invalid values)

If the feature involves display-dependent operations (mouse, keyboard, display, window), start Xvfb first:
```bash
Xvfb :99 -screen 0 1920x1080x24 -ac &
export DISPLAY=:99
```

Record each manual check as an `interactiveChecks` entry in the handoff.

### 7. Commit

- `git add -A`
- Write a clear commit message describing what was implemented
- `git commit`

## Example Handoff

```json
{
  "salientSummary": "Implemented mouse primitive with 7 subcommands (move, click, double-click, right-click, scroll, drag, position). All use enigo crate. Added --json support for position command. 12 tests passing, clippy clean.",
  "whatWasImplemented": "src/mouse.rs with Move, Click, DoubleClick, RightClick, Scroll, Drag, Position subcommands. Each dispatches to enigo. Position returns JSON with {x, y}. Added MouseCommands enum to cli.rs. Error handling for out-of-bounds coords (clamps to screen). Tests in tests/mouse_test.rs covering CLI parsing, JSON output format, and error cases.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      {"command": "cargo test", "exitCode": 0, "observation": "12 tests passing including mouse CLI parsing and JSON output"},
      {"command": "cargo clippy -- -D warnings", "exitCode": 0, "observation": "No warnings"},
      {"command": "cargo fmt --check", "exitCode": 0, "observation": "Formatted correctly"}
    ],
    "interactiveChecks": [
      {"action": "cargo run -- mouse --help", "observed": "Shows all 7 mouse subcommands with descriptions"},
      {"action": "cargo run -- mouse position --json (with DISPLAY=:99)", "observed": "Returns {\"x\": 0, \"y\": 0} valid JSON"},
      {"action": "cargo run -- mouse move --x 100 --y 200 && cargo run -- mouse position --json", "observed": "Returns {\"x\": 100, \"y\": 200}"},
      {"action": "cargo run -- mouse click --x 50 --y 50", "observed": "Exits 0, cursor at (50, 50)"},
      {"action": "cargo run -- mouse move --x 99999 --y 99999", "observed": "Exits 0, position clamped to screen bounds"}
    ]
  },
  "tests": {
    "added": [
      {"file": "tests/mouse_test.rs", "cases": [
        {"name": "test_mouse_position_json", "verifies": "position --json returns valid JSON with x,y fields"},
        {"name": "test_mouse_move_args", "verifies": "move requires --x and --y arguments"},
        {"name": "test_mouse_click_optional_position", "verifies": "click works with and without --x/--y"},
        {"name": "test_mouse_scroll_amount", "verifies": "scroll requires --amount argument"},
        {"name": "test_mouse_drag_args", "verifies": "drag requires all four position arguments"}
      ]}
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- A dependency crate doesn't compile on this platform (missing system library that init.sh didn't install)
- The feature depends on another primitive that hasn't been implemented yet
- The feature description is ambiguous about CLI interface (flag names, subcommand structure)
- System-level operations require permissions not available in the environment
- Xvfb or ffmpeg is unavailable and the feature requires it for testing
