# Antigravity Keyring Switch Implementation Plan

**For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans implement plan task-by-task.

**Goal:** Add REPL commands that capture and restore real Antigravity CLI authentication from the desktop Secret Service keyring.

**Architecture:** Add a focused `antigravity` module that shells out to `secret-tool` for the known keyring attributes `service=gemini username=antigravity`. Keep secret contents out of stdout; persist per-account snapshots under the existing peternak config directory.

**Tech Stack:** Rust 2024, clap subcommands, `std::process::Command`, Linux Secret Service via `secret-tool`.

---

### Task 1: Add Antigravity Secret Module

**Files:**
- Create: `src/antigravity.rs`
- Modify: `src/config.rs`

- [ ] Add snapshot path helpers under `~/.config/peternak-aiai`.
- [ ] Implement `capture` by reading `secret-tool lookup service gemini username antigravity`.
- [ ] Implement `switch` by feeding a snapshot into `secret-tool store --label ... service gemini username antigravity`.
- [ ] Implement `list` by listing `agy-*.secret.json` without reading file contents.

### Task 2: Wire REPL Commands

**Files:**
- Modify: `src/main.rs`

- [ ] Add `CaptureAg { account }`, `SwitchAg { account }`, and `ListAg`.
- [ ] Route commands to `antigravity` module functions.
- [ ] Keep old `inject <email> antigravity` behavior unchanged for now.

### Task 3: Verify

**Commands:**
- `rtk cargo build`
- `rtk cargo run`, then `list-ag`, `switch-ag muihsan`, and `switch-ag pjrlywm`

Expected result: project builds, snapshot commands do not print secret values, and `agy` header changes after restore.
