use crate::{config, credential_store, db};
use colored::*;
use rusqlite::Connection;
use std::fs;
use std::process::Command;

const AGY_PROVIDER: &str = "agy";

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

pub fn run_interactive(conn: &Connection, agy_args: &[String], forced_account: Option<&str>) {
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

        let _ = db::mark_backend_account_used(conn, account.id);
        let status = Command::new("agy").args(agy_args).status();

        match status {
            Ok(status) if status.success() => {
                println!("{}", "[Peternak] agy session closed.".dimmed());
                return;
            }
            Ok(status) => {
                println!(
                    "{}",
                    format!("[Peternak] agy exited with status: {}", status).yellow()
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
