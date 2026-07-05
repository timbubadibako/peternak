# Peternak Orchestrator MVP Design

## Goal

Build a local-first terminal orchestrator that gives users one Peternak panel while the admin manages multiple backend identities for `agy` and `gcloud`.

## Scope

The MVP covers a SQLite-backed identity model, admin/user roles, slash commands with autocomplete, account administration, and an `/agy` interactive supervisor. Supabase, Vercel, GitHub, remote auth, and concurrent `agy` sessions are outside this first implementation.

## Roles

Users enter an SSO ID from the local database/session prompt. The MVP hardcodes `sso_id = 999` as admin and every other SSO ID as user. Admins can add, list, test, disable, and remove backend accounts. Users cannot see or select backend accounts.

## Command UX

The main panel is a Rust REPL prompt:

```text
peternak>
```

Slash commands are first-class commands and should be discoverable through autocomplete:

```text
/agy
/account add agy <email-or-alias>
/account add gcloud <email>
/account list
/account test <id-or-alias>
/account disable <id-or-alias>
/account remove <id-or-alias>
/status
/help
```

Legacy commands can remain available for compatibility, but the MVP path uses slash commands.

## Data Model

SQLite remains local in `~/.config/peternak-aiai/data.db`.

`users` stores local SSO IDs and roles. `backend_accounts` stores metadata only: provider, alias, email, status, credential reference, and timestamps. The secret/token itself should live outside the DB.

For the first verified platform, Antigravity/`agy` credentials use the existing Linux Secret Service flow and snapshot files under Peternak config. The code should keep a clear boundary so macOS Keychain and Windows Credential Manager can be added later through the same account/switch interface.

## Agy Supervisor

`/agy` starts an interactive `agy` session from inside Peternak. Peternak automatically selects the first active `agy` backend account. It injects that account's credential, launches `agy`, waits for it to exit, and returns to `peternak>`.

Only one `agy` process is allowed at a time. This avoids credential mixups while the MVP uses a global keyring slot that `agy` reads.

If `agy` exits with failure, Peternak prints a compact-context transition message, marks the current account degraded, switches to another active account if available, reinjects credentials, and restarts `agy` in the same terminal panel. The MVP preserves the panel, current directory, and session notes; native private `agy` process state is reconstructed from Peternak-owned context rather than moved between accounts.

## Error Handling

Missing accounts, disabled accounts, missing snapshots, keyring injection failure, and exhausted account pools should produce short actionable messages. Admin-only commands must reject user role sessions.

## Testing

Verification should include `cargo check`. Manual smoke testing should cover slash command parsing, admin role gating, account list/status output, and `/agy` failure handling where possible.

## Admin Setup Flow

The admin flow is email-first:

```text
1. Admin logs in with SSO ID 999.
2. Admin registers client emails with /account add <email>.
3. For each email, admin makes that account the active agy login/keyring account.
4. Admin runs /account setup agy <email> to capture and bind the active agy credential.
5. Admin verifies with /account list and /account test <email-or-id>.
6. Users run /agy and Peternak picks an active bound agy credential automatically.
```
