# Agy Continuity PTY Implementation Plan

**For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) superpowers:executing-plans implement plan task-by-task. Steps use checkbox (`- [ ]`) syntax tracking.

**Goal:** Run `agy` through a Peternak-owned PTY supervisor so context can be captured and restored across backend account switches.

**Architecture:** Add a small continuity context object in `antigravity.rs` that stores recent user inputs and assistant output snippets. Replace direct `Command::new("agy").status()` for normal `/agy` sessions with a PTY-backed runner that forwards terminal input/output while recording transcript. When a session starts on a different backend account and context exists, Peternak writes a restore prompt into `agy` before user input continues.

**Tech Stack:** Rust 2024, portable-pty, rusqlite, std::io/thread.

---

### Task 1: Add PTY Dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] Add `portable-pty` dependency.
- [ ] Run `rtk cargo check`.

### Task 2: Add Continuity Context

**Files:**
- Modify: `src/antigravity.rs`
- Modify: `src/main.rs`

- [ ] Add `AgyContinuityContext` with bounded transcript entries.
- [ ] Store one mutable context in `main()` and pass it to `/agy` calls.

### Task 3: Add PTY Runner

**Files:**
- Modify: `src/antigravity.rs`

- [ ] Spawn `agy` with `portable_pty`.
- [ ] Forward PTY output to stdout and append compact output snippets.
- [ ] Read user input lines, forward to PTY, and append user messages.
- [ ] Inject restore prompt when context exists and account changes.

### Task 4: Verify

- [ ] Run `rtk cargo fmt --check`.
- [ ] Run `rtk cargo check`.
- [ ] Smoke test SSO rejection still works.
