use crate::{config, credential_store, db};
use colored::*;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use portable_pty::{CommandBuilder, ExitStatus as PtyExitStatus, PtySize, native_pty_system};
use rusqlite::Connection;
use std::fs;
use std::io::{self, Read, Write};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const AGY_PROVIDER: &str = "agy";
const CONTEXT_ENTRY_LIMIT: usize = 30;
const CONTEXT_CHAR_LIMIT: usize = 6_000;

#[derive(Default)]
pub struct AgyContinuityContext {
    user_entries: Vec<String>,
    user_rules: Vec<String>,
    last_account_alias: Option<String>,
}

#[derive(Default)]
struct InputCaptureState {
    line: Vec<u8>,
    escape: EscapeState,
}

#[derive(Default)]
enum EscapeState {
    #[default]
    None,
    Esc,
    Csi,
    Osc,
}

impl AgyContinuityContext {
    fn push_user(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        if looks_like_user_rule(line) && !self.user_rules.iter().any(|rule| rule == line) {
            self.user_rules.push(line.chars().take(800).collect());
        }

        self.user_entries.push(line.chars().take(800).collect());
        if self.user_entries.len() > CONTEXT_ENTRY_LIMIT {
            let overflow = self.user_entries.len() - CONTEXT_ENTRY_LIMIT;
            self.user_entries.drain(0..overflow);
        }

        while self.render().len() > CONTEXT_CHAR_LIMIT && self.user_entries.len() > 1 {
            self.user_entries.remove(0);
        }
    }

    fn render(&self) -> String {
        let mut lines = Vec::new();
        if !self.user_rules.is_empty() {
            lines.push("Explicit user rules:".to_string());
            lines.extend(self.user_rules.iter().map(|rule| format!("- {}", rule)));
        }

        if !self.user_entries.is_empty() {
            lines.push("Recent user messages:".to_string());
            lines.extend(self.user_entries.iter().map(|entry| format!("- {}", entry)));
        }

        lines.join("\n")
    }

    fn has_context(&self) -> bool {
        !self.user_entries.is_empty() || !self.user_rules.is_empty()
    }

    fn should_restore_for(&self, account_alias: &str) -> bool {
        self.has_context()
            && self
                .last_account_alias
                .as_deref()
                .is_some_and(|last| last != account_alias)
    }

    fn mark_account(&mut self, account_alias: &str) {
        self.last_account_alias = Some(account_alias.to_string());
    }
}

fn looks_like_user_rule(line: &str) -> bool {
    let lower = line.to_lowercase();
    (lower.contains("kalau") || lower.contains("kalo") || lower.contains("jika"))
        && (lower.contains("jawab") || lower.contains("balas") || lower.contains("harus"))
}

fn account_alias(account: &str) -> String {
    let base = account.split('@').next().unwrap_or(account).trim();
    base.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

fn existing_secret_path(account: &str) -> Option<std::path::PathBuf> {
    let alias = account_alias(account);
    let candidates = [
        config::get_antigravity_secret_path(account),
        config::get_antigravity_secret_path(&alias),
        config::get_antigravity_secret_path(alias.trim_end_matches(|c: char| c.is_ascii_digit())),
    ];

    candidates.into_iter().find(|path| path.exists())
}

fn resolve_account_selector(conn: &Connection, selector: &str) -> Option<String> {
    let selector = selector.trim();
    if selector.is_empty() {
        println!("{}", "Nama akun tidak valid.".red());
        return None;
    }

    if let Ok(number) = selector.parse::<usize>() {
        match db::get_account_by_number(conn, number) {
            Ok(Some(email)) => return Some(email),
            Ok(None) => {
                println!(
                    "{}",
                    format!("Akun nomor {} tidak ada di database.", number).red()
                );
                println!("{}", "Jalankan 'list' untuk melihat nomor akun.".yellow());
                return None;
            }
            Err(e) => {
                println!("{}", format!("Gagal membaca database: {}", e).red());
                return None;
            }
        }
    }

    Some(selector.to_string())
}

fn set_owner_only_permissions(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = fs::set_permissions(path, permissions);
        }
    }
}

pub fn handle_clear_active() {
    match credential_store::clear_active_agy_secret() {
        Ok(_) => {
            println!("{}", "Active agy keyring slot dibersihkan.".green());
            println!(
                "{}",
                "Jalankan agy dan login dengan akun target, lalu /account setup agy <email>."
                    .yellow()
            );
        }
        Err(e) => println!(
            "{}",
            format!("Gagal membersihkan active agy keyring: {}", e).red()
        ),
    }
}

fn switch_from_secret(account: &db::BackendAccount) -> bool {
    let secret = match credential_store::read_peternak_secret(AGY_PROVIDER, &account.alias) {
        Ok(secret) => secret,
        Err(_) => match account.credential_ref.as_deref() {
            Some(path) => match fs::read(path) {
                Ok(secret) => secret,
                Err(e) => {
                    println!("{}", format!("Gagal membaca snapshot: {}", e).red());
                    return false;
                }
            },
            None => {
                println!(
                    "{}",
                    format!("Backend agy {} belum punya credential.", account.alias).red()
                );
                return false;
            }
        },
    };

    match credential_store::write_active_agy_secret(&secret) {
        Ok(_) => {
            let source = account
                .credential_ref
                .as_deref()
                .unwrap_or("OS keyring Peternak");
            println!(
                "{}",
                format!("Antigravity keyring aktif dari {}", source)
                    .green()
                    .bold()
            );
            true
        }
        Err(e) => {
            println!("{}", format!("Gagal inject credential agy: {}", e).red());
            false
        }
    }
}

pub fn handle_capture(conn: &Connection, account: &str) {
    let account = match resolve_account_selector(conn, account) {
        Some(account) => account,
        None => return,
    };

    let alias = account_alias(&account);
    if alias.is_empty() {
        println!("{}", "Nama akun tidak valid.".red());
        return;
    }

    println!(
        "{}",
        format!(
            "Setup agy untuk {} akan mengambil credential agy yang sedang aktif di keyring.",
            account
        )
        .yellow()
    );

    let secret = match credential_store::read_active_agy_secret() {
        Ok(secret) => secret,
        Err(e) => {
            println!(
                "{}",
                format!("Secret Antigravity aktif tidak ditemukan: {}", e).red()
            );
            return;
        }
    };

    if secret.is_empty() {
        println!(
            "{}",
            "Secret Antigravity aktif tidak ditemukan di keyring.".red()
        );
        println!(
            "{}",
            "Login dulu lewat agy, lalu jalankan capture-ag lagi.".yellow()
        );
        return;
    }

    let path = config::get_antigravity_secret_path(&alias);
    if let Err(e) = credential_store::store_peternak_secret(AGY_PROVIDER, &alias, &secret) {
        println!(
            "{}",
            format!("Gagal menyimpan secret ke OS keyring Peternak: {}", e).red()
        );
        return;
    }

    match fs::write(&path, &secret) {
        Ok(_) => {
            set_owner_only_permissions(&path);
            let credential_ref = path.to_string_lossy().to_string();
            let _ = db::add_backend_account(
                conn,
                AGY_PROVIDER,
                &alias,
                Some(&account),
                Some(&credential_ref),
            );
            println!(
                "{}",
                format!(
                    "Snapshot Antigravity untuk {} tersimpan: {}",
                    account,
                    path.display()
                )
                .green()
                .bold()
            );
        }
        Err(e) => println!("{}", format!("Gagal menyimpan snapshot: {}", e).red()),
    }
}

pub fn handle_setup_login(conn: &Connection, account: &str) {
    let account = match resolve_account_selector(conn, account) {
        Some(account) => account,
        None => return,
    };

    println!(
        "{}",
        format!(
            "Setup agy untuk {} dimulai. Peternak akan membersihkan keyring aktif lalu membuka agy untuk login.",
            account
        )
        .cyan()
    );

    match credential_store::clear_active_agy_secret() {
        Ok(_) => {
            println!("{}", "Active agy keyring slot dibersihkan.".green());
        }
        Err(e) => {
            println!(
                "{}",
                format!("Gagal membersihkan active agy keyring: {}", e).red()
            );
            return;
        }
    }

    println!(
        "{}",
        "Login di agy dengan akun target. Setelah banner email benar, keluar dari agy untuk menyimpan setup."
            .yellow()
    );

    match Command::new("agy").status() {
        Ok(status) => {
            println!(
                "{}",
                format!("[Peternak] agy setup session closed ({status}).").dimmed()
            );
        }
        Err(e) => {
            println!("{}", format!("Gagal menjalankan agy: {}", e).red());
            return;
        }
    }

    println!(
        "{}",
        "Peternak mengambil credential agy hasil login barusan...".cyan()
    );
    handle_capture(conn, &account);
}

pub fn handle_switch(conn: &Connection, account: &str) {
    let account = match resolve_account_selector(conn, account) {
        Some(account) => account,
        None => return,
    };

    if let Some(backend) = db::find_backend_account(conn, &account).ok().flatten() {
        if switch_from_secret(&backend) {
            let _ = db::mark_backend_account_used(conn, backend.id);
        }
        return;
    }

    let path = match existing_secret_path(&account) {
        Some(path) => path,
        None => {
            println!(
                "{}",
                format!("Snapshot Antigravity untuk {} belum ditemukan.", account).red()
            );
            println!(
                "{}",
                "Jalankan capture-ag saat agy sedang login dengan akun itu.".yellow()
            );
            println!(
                "{}",
                format!(
                    "Nama file yang dicari: agy-{}.secret.json",
                    account_alias(&account)
                )
                .dimmed()
            );
            return;
        }
    };

    let alias = account_alias(&account);
    let credential_ref = path.to_string_lossy().to_string();
    let backend = db::BackendAccount {
        id: 0,
        provider: AGY_PROVIDER.to_string(),
        alias: alias.clone(),
        email: Some(account.clone()),
        status: "active".to_string(),
        credential_ref: Some(credential_ref.clone()),
    };

    if switch_from_secret(&backend) {
        let _ = db::add_backend_account(
            conn,
            AGY_PROVIDER,
            &alias,
            Some(&account),
            Some(&credential_ref),
        );
    }
}

pub fn handle_reset(conn: &Connection, selector: &str) {
    let account = match db::find_backend_account(conn, selector) {
        Ok(Some(account)) => account,
        Ok(None) => {
            println!(
                "{}",
                format!("Backend account '{}' tidak ditemukan.", selector).red()
            );
            return;
        }
        Err(e) => {
            println!("{}", format!("Gagal membaca backend account: {}", e).red());
            return;
        }
    };

    if account.provider != AGY_PROVIDER {
        println!(
            "{}",
            format!("Backend account '{}' bukan provider agy.", selector).red()
        );
        return;
    }

    let _ = credential_store::delete_peternak_secret(AGY_PROVIDER, &account.alias);

    if let Some(credential_ref) = account.credential_ref.as_deref() {
        match fs::remove_file(credential_ref) {
            Ok(_) => println!(
                "{}",
                format!("Snapshot dihapus: {}", credential_ref).yellow()
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => println!(
                "{}",
                format!("Gagal menghapus snapshot {}: {}", credential_ref, e).red()
            ),
        }
    }

    let fallback_path = config::get_antigravity_secret_path(&account.alias);
    if fallback_path.exists() {
        let _ = fs::remove_file(&fallback_path);
    }

    let _ = db::remove_backend_account_by_id(conn, account.id);
    println!(
        "{}",
        "Email client tetap terdaftar. Jalankan setup ulang setelah agy login ke akun yang benar."
            .cyan()
    );
}

pub fn handle_list() {
    let dir = config::get_app_dir();
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            println!(
                "{}",
                format!("Gagal membaca {}: {}", dir.display(), e).red()
            );
            return;
        }
    };

    let mut accounts = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| {
            name.strip_prefix("agy-")
                .and_then(|name| name.strip_suffix(".secret.json"))
                .map(|name| name.to_string())
        })
        .collect::<Vec<_>>();

    accounts.sort();

    if accounts.is_empty() {
        println!("{}", "Belum ada snapshot Antigravity.".yellow());
        return;
    }

    println!("{}", "Snapshot Antigravity:".cyan().bold());
    for (index, account) in accounts.iter().enumerate() {
        println!("{}. {}", index + 1, account);
    }
}

pub fn run_interactive(
    conn: &Connection,
    agy_args: &[String],
    forced_account: Option<&str>,
    context: &mut AgyContinuityContext,
) {
    let mut attempted = Vec::new();

    loop {
        let account = match select_run_account(conn, forced_account, &attempted) {
            Ok(Some(account)) => account,
            Ok(None) => {
                println!("{}", "Tidak ada backend agy aktif yang bisa dipakai.".red());
                println!(
                    "{}",
                    "Admin perlu menjalankan /account add agy <alias> dulu.".yellow()
                );
                return;
            }
            Err(e) => {
                println!("{}", format!("Gagal membaca backend account: {}", e).red());
                return;
            }
        };

        let mode = if agy_args.is_empty() {
            "new/default".to_string()
        } else {
            agy_args.join(" ")
        };
        println!(
            "{}",
            format!("[Peternak] Starting agy session through orchestrator ({mode})...")
                .cyan()
                .bold()
        );

        if !switch_from_secret(&account) {
            let _ = db::set_backend_account_status_by_id(conn, account.id, "degraded");
            attempted.push(account.id);
            continue;
        }

        let restore_prompt = if context.should_restore_for(&account.alias) {
            Some(build_restore_prompt(context))
        } else {
            None
        };
        context.mark_account(&account.alias);

        let _ = db::mark_backend_account_used(conn, account.id);
        let status = run_agy_pty(agy_args, restore_prompt.as_deref(), context);

        match status {
            Ok(status) if status.success() => {
                println!("{}", "[Peternak] agy session closed.".dimmed());
                return;
            }
            Ok(status) => {
                println!(
                    "{}",
                    format!("[Peternak] agy exited with code: {}", status.exit_code()).yellow()
                );
                if forced_account.is_some() {
                    return;
                }
                print_context_switch_message();
                let _ = db::set_backend_account_status_by_id(conn, account.id, "degraded");
                attempted.push(account.id);
            }
            Err(e) => {
                println!("{}", format!("Gagal menjalankan agy: {}", e).red());
                return;
            }
        }
    }
}

fn select_run_account(
    conn: &Connection,
    forced_account: Option<&str>,
    attempted: &[i64],
) -> rusqlite::Result<Option<db::BackendAccount>> {
    if let Some(selector) = forced_account {
        let Some(account) = db::find_backend_account(conn, selector)? else {
            println!(
                "{}",
                format!("Backend account '{}' tidak ditemukan.", selector).red()
            );
            return Ok(None);
        };

        if account.provider != AGY_PROVIDER {
            println!(
                "{}",
                format!("Backend account '{}' bukan provider agy.", selector).red()
            );
            return Ok(None);
        }

        if account.status != "active" {
            println!(
                "{}",
                format!(
                    "Backend account '{}' statusnya {}.",
                    selector, account.status
                )
                .red()
            );
            return Ok(None);
        }

        return Ok(Some(account));
    }

    db::select_active_backend_account(conn, AGY_PROVIDER, attempted)
}

fn print_context_switch_message() {
    println!(
        "{}",
        "[Peternak] Account limit or backend failure detected.".yellow()
    );
    println!("{}", "[Peternak] Compressing session context...".cyan());
    println!("{}", "[Peternak] Switching backend identity...".cyan());
    println!("{}", "[Peternak] Restoring workspace context...".cyan());
}

fn build_restore_prompt(context: &AgyContinuityContext) -> String {
    format!(
        "Continue the same Peternak session after a backend account switch. Use this context as active instructions and memory.\n\n<context>\n{}\n</context>\n\nAcknowledge in one short sentence, then wait for the user.\n",
        context.render()
    )
}

fn run_agy_pty(
    agy_args: &[String],
    restore_prompt: Option<&str>,
    context: &mut AgyContinuityContext,
) -> io::Result<PtyExitStatus> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 40,
            cols: 140,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut cmd = CommandBuilder::new("agy");
    for arg in agy_args {
        cmd.arg(arg);
    }
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd.as_os_str());
        cmd.env("PWD", cwd.as_os_str());
    }

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let mut writer = pair
        .master
        .take_writer()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let (output_tx, output_rx) = mpsc::channel::<String>();
    let output_thread = thread::spawn(move || {
        let mut stdout = io::stdout();
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = stdout.write_all(&buffer[..n]);
                    let _ = stdout.flush();
                    let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                    let _ = output_tx.send(chunk);
                }
                Err(_) => break,
            }
        }
    });

    enable_raw_mode()?;
    let _raw_guard = RawModeGuard;
    let (input_tx, input_rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        let mut stdin = io::stdin();
        let mut buffer = [0_u8; 1024];
        loop {
            match stdin.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if input_tx.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut input_capture = InputCaptureState::default();
    let mut restore_pending = restore_prompt.map(str::to_string);
    let mut output_seen = String::new();
    let started_at = Instant::now();
    loop {
        drain_output(&output_rx, &mut output_seen);

        if let Some(prompt) = restore_pending.as_deref() {
            if agy_looks_ready(&output_seen) || started_at.elapsed() > Duration::from_secs(3) {
                println!(
                    "{}",
                    "[Peternak] Restoring captured context into agy...".cyan()
                );
                writer.write_all(prompt.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.flush()?;
                restore_pending = None;
            }
        }

        if let Some(status) = child
            .try_wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
        {
            let _ = output_thread.join();
            drain_output(&output_rx, &mut output_seen);
            return Ok(status);
        }

        for bytes in input_rx.try_iter() {
            capture_user_bytes(context, &mut input_capture, &bytes);
            writer.write_all(&bytes)?;
            writer.flush()?;
        }

        thread::sleep(Duration::from_millis(20));
    }
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn capture_user_bytes(
    context: &mut AgyContinuityContext,
    state: &mut InputCaptureState,
    bytes: &[u8],
) {
    for byte in bytes {
        match state.escape {
            EscapeState::Esc => {
                state.escape = match *byte {
                    b'[' => EscapeState::Csi,
                    b']' => EscapeState::Osc,
                    _ => EscapeState::None,
                };
                continue;
            }
            EscapeState::Csi => {
                if (0x40..=0x7e).contains(byte) {
                    state.escape = EscapeState::None;
                }
                continue;
            }
            EscapeState::Osc => {
                if *byte == 0x07 {
                    state.escape = EscapeState::None;
                }
                continue;
            }
            EscapeState::None => {}
        }

        match *byte {
            0x1b => {
                state.escape = EscapeState::Esc;
            }
            b'\r' | b'\n' => {
                if !state.line.is_empty() {
                    let line = String::from_utf8_lossy(&state.line).to_string();
                    context.push_user(line.trim());
                    state.line.clear();
                }
            }
            8 | 127 => {
                state.line.pop();
            }
            0x20..=0x7e | 0x80..=0xff => {
                state.line.push(*byte);
            }
            _ => {}
        }
    }
}

fn drain_output(output_rx: &mpsc::Receiver<String>, output_seen: &mut String) {
    for chunk in output_rx.try_iter() {
        output_seen.push_str(&chunk);
        if output_seen.len() > 20_000 {
            let keep_from = output_seen.len() - 10_000;
            output_seen.drain(..keep_from);
        }
    }
}

fn agy_looks_ready(output_seen: &str) -> bool {
    output_seen.contains("Antigravity CLI")
        && (output_seen.contains("? for shortcuts") || output_seen.contains("────────────────"))
}
