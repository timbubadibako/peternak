# Peternak Orchestrator MVP Implementation Plan

**For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) superpowers:executing-plans implement plan task-by-task. Steps use checkbox (`- [ ]`) syntax tracking.

**Goal:** Add the local Orchestrator MVP with slash commands, admin/user role gating, backend account metadata, and an `/agy` interactive supervisor.

**Architecture:** Keep the existing CLI commands intact, then add a slash-command layer in `main.rs` for the new MVP UX. Store role and backend account metadata in SQLite through focused functions in `db.rs`. Keep `agy` credential capture/switch/run behavior in `antigravity.rs` so future keyring adapters can replace the Linux `secret-tool` implementation behind the same interface.

**Tech Stack:** Rust 2024, rusqlite, rustyline, clap, colored, std::process.

---

### Task 1: Extend SQLite Schema

**Files:**
- Modify: `src/db.rs`

- [ ] **Step 1: Add role and backend account schema**

Create `users` and `backend_accounts` tables in `init_db()`. Add small helper types and functions for role resolution, backend account CRUD, status updates, and active account selection.

- [ ] **Step 2: Verify schema compiles**

Run: `rtk cargo check`

Expected: command exits successfully.

### Task 2: Add Slash REPL Autocomplete and Role Session

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add REPL helper**

Use rustyline completion for slash commands and prompt the user for local SSO ID before entering the loop.

- [ ] **Step 2: Add slash command dispatcher**

Route `/help`, `/status`, `/account ...`, and `/agy` before legacy clap parsing. Enforce admin-only access for `/account` mutation commands.

- [ ] **Step 3: Verify command parsing compiles**

Run: `rtk cargo check`

Expected: command exits successfully.

### Task 3: Add Agy Supervisor

**Files:**
- Modify: `src/antigravity.rs`

- [ ] **Step 1: Add account metadata integration**

Expose helpers to capture/switch/list Antigravity snapshots through the new `backend_accounts` table.

- [ ] **Step 2: Add `/agy` interactive runner**

Select an active account automatically, switch credentials, spawn `agy`, and restart on failure after printing compact-context transition messages.

- [ ] **Step 3: Verify build**

Run: `rtk cargo check`

Expected: command exits successfully.

### Task 4: Final Verification

**Files:**
- Modify: `src/main.rs`
- Modify: `src/db.rs`
- Modify: `src/antigravity.rs`
- Create: `docs/superpowers/specs/2026-07-05-peternak-orchestrator-mvp-design.md`
- Create: `docs/superpowers/plans/2026-07-05-peternak-orchestrator-mvp.md`

- [ ] **Step 1: Format**

Run: `rtk cargo fmt`

Expected: formatting completes successfully.

- [ ] **Step 2: Check**

Run: `rtk cargo check`

Expected: command exits successfully.
