---
name: ci-worker
description: Sets up GitHub repo, Actions workflows, and e2e test suites for xctrl
---

# CI Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Use for features involving GitHub repository setup, GitHub Actions CI/CD workflows, e2e test scripts, and iterating on CI failures.

## Work Procedure

### 1. Understand the Feature

Read the feature description carefully. Check:
- `preconditions` — what code/tests must already exist
- `expectedBehavior` — what the CI should do
- `verificationSteps` — how to verify success

### 2. Plan the Work

For GitHub repo setup:
- Initialize git repo if not done
- Create GitHub repo via `gh repo create` (or document manual steps if gh unavailable)
- Push all code

For GitHub Actions workflows:
- Create `.github/workflows/` directory
- Design workflow YAML with OS matrix (ubuntu-latest, macos-latest, windows-latest)
- Include platform-specific setup steps (Xvfb on Linux, accessibility permissions on macOS)

For e2e tests:
- Create test scripts that exercise xctrl commands
- Use `cargo test` for Rust-level tests
- For e2e, create shell scripts or Rust integration tests that spawn the xctrl binary
- Handle platform differences with conditional steps

### 3. Write the CI Workflow

Structure the workflow as:

```yaml
name: CI
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - cargo clippy
      - cargo fmt --check

  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - Install Rust
      - Install platform deps
      - cargo build
      - cargo test
      - Run e2e tests (platform-specific)
```

Key platform-specific considerations:
- **Linux**: Install `libxdo-dev`, `xvfb`, `ffmpeg`. Run display tests under `xvfb-run` or start Xvfb manually.
- **macOS**: May need `brew install` for some deps. Accessibility permissions may be limited in CI.
- **Windows**: Use `choco install` or `scoop` for deps. Display capture uses `gdigrab`.

### 4. Write E2E Tests

Create e2e tests as Rust integration tests in `tests/e2e/` or as shell scripts in `tests/scripts/`:
- Each test exercises a specific xctrl command
- Tests check: exit code, stdout content, file creation (for screenshots/recordings)
- Use `#[cfg(target_os = "...")]` for platform-specific tests
- Tests that need a display must be marked/gated appropriately

### 5. Push and Verify

- Commit all workflow and test files
- Push to GitHub: `git push -u origin main`
- Check GitHub Actions status: `gh run list` or `gh run watch`
- If CI fails, analyze logs and fix

### 6. Iterate on Failures

- Read CI logs: `gh run view <id> --log`
- Fix platform-specific issues
- Re-push and verify
- Repeat until all OS runners pass

### 7. Commit

After CI is green on all platforms:
- Ensure all fixes are committed
- Push final state

## Example Handoff

```json
{
  "salientSummary": "Created GitHub repo enoreyes/xctrl, pushed all code, configured GitHub Actions CI with Linux/macOS/Windows matrix. E2e tests cover clipboard, mouse, keyboard, display, recording, window, and OS actions. All 3 OS runners passing after 2 fix iterations.",
  "whatWasImplemented": ".github/workflows/ci.yml with build+test+e2e jobs across ubuntu/macos/windows matrix. tests/e2e/ directory with platform-gated integration tests for all primitives. Linux uses xvfb-run for display tests. macOS tests skip accessibility-dependent operations. Windows tests use native display capture.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      {"command": "gh repo view --web", "exitCode": 0, "observation": "Repo accessible at https://github.com/user/xctrl"},
      {"command": "gh run list --limit 3", "exitCode": 0, "observation": "Latest run: all 3 OS jobs passing"},
      {"command": "gh run view <id> --log", "exitCode": 0, "observation": "Linux: 24/24 tests pass. macOS: 20/20 tests pass. Windows: 18/18 tests pass."}
    ],
    "interactiveChecks": [
      {"action": "Checked GitHub Actions UI for latest run", "observed": "All 3 OS runners show green checkmarks"},
      {"action": "Reviewed Linux runner logs for e2e tests", "observed": "Xvfb started, mouse/keyboard/clipboard/display/recording tests all pass"},
      {"action": "Reviewed macOS runner logs", "observed": "Clipboard, display, OS actions tests pass. Mouse/keyboard tests pass."},
      {"action": "Reviewed Windows runner logs", "observed": "Clipboard, display, OS actions tests pass. Mouse/keyboard tests pass."}
    ]
  },
  "tests": {
    "added": [
      {"file": ".github/workflows/ci.yml", "cases": [
        {"name": "lint job", "verifies": "clippy and fmt pass on ubuntu"},
        {"name": "test matrix", "verifies": "cargo test passes on all 3 OSes"},
        {"name": "e2e matrix", "verifies": "e2e integration tests pass per-platform"}
      ]},
      {"file": "tests/e2e/clipboard_e2e.rs", "cases": [
        {"name": "test_clipboard_roundtrip", "verifies": "set then get returns same text"}
      ]}
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- `gh` CLI is not available and cannot be installed (need manual repo creation)
- GitHub authentication is not configured
- CI fails on a specific OS due to a code bug that requires implementation changes (not CI config)
- Platform-specific system dependencies are unavailable in CI runners
- CI runner limitations prevent testing specific features (document what can't be tested)
