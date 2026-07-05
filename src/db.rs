use crate::config;
use colored::*;
use rusqlite::{Connection, Result, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn from_sso_id(sso_id: &str) -> Self {
        if sso_id.trim() == "999" {
            Self::Admin
        } else {
            Self::User
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

#[derive(Clone, Debug)]
pub struct BackendAccount {
    pub id: i64,
    pub provider: String,
    pub alias: String,
    pub email: Option<String>,
    pub status: String,
    pub credential_ref: Option<String>,
}

pub fn init_db() -> Result<Connection> {
    let db_path = crate::config::get_app_dir().join("data.db");

    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            refresh_token TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN supabase_token TEXT", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN vercel_token TEXT", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN github_token TEXT", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL,
            service TEXT NOT NULL,
            access_token TEXT NOT NULL,
            expires_at DATETIME,
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            sso_id TEXT PRIMARY KEY,
            role TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS backend_accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            alias TEXT NOT NULL,
            email TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            credential_ref TEXT,
            last_used_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(provider, alias)
        )",
        [],
    )?;

    Ok(conn)
}

pub fn ensure_user(conn: &Connection, sso_id: &str) -> Result<Role> {
    let role = Role::from_sso_id(sso_id);
    conn.execute(
        "INSERT INTO users (sso_id, role)
         VALUES (?1, ?2)
         ON CONFLICT(sso_id) DO UPDATE SET role = excluded.role",
        params![sso_id.trim(), role.as_str()],
    )?;
    Ok(role)
}

pub fn register_email_account(conn: &Connection, email: &str) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
    if count >= 99 {
        println!("{}", "Batas 99 akun email sudah tercapai.".red());
        return Ok(());
    }

    conn.execute(
        "INSERT INTO accounts (email)
         VALUES (?1)
         ON CONFLICT(email) DO NOTHING",
        params![email.trim()],
    )?;

    println!(
        "{}",
        format!("Email account {} terdaftar.", email.trim()).green()
    );
    Ok(())
}

pub fn get_registered_emails(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT email FROM accounts ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut emails = Vec::new();
    for row in rows {
        emails.push(row?);
    }
    Ok(emails)
}

pub fn add_backend_account(
    conn: &Connection,
    provider: &str,
    alias: &str,
    email: Option<&str>,
    credential_ref: Option<&str>,
) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM backend_accounts", [], |row| {
        row.get(0)
    })?;
    if count >= 99 {
        println!("{}", "Batas 99 backend account sudah tercapai.".red());
        return Ok(());
    }

    conn.execute(
        "INSERT INTO backend_accounts
            (provider, alias, email, status, credential_ref, updated_at)
         VALUES (?1, ?2, ?3, 'active', ?4, CURRENT_TIMESTAMP)
         ON CONFLICT(provider, alias) DO UPDATE SET
            email = excluded.email,
            status = 'active',
            credential_ref = excluded.credential_ref,
            updated_at = CURRENT_TIMESTAMP",
        params![
            provider.trim(),
            alias.trim(),
            email.map(str::trim),
            credential_ref.map(str::trim)
        ],
    )?;

    println!(
        "{}",
        format!("Backend account {}:{} aktif.", provider, alias).green()
    );
    Ok(())
}

pub fn list_backend_accounts(conn: &Connection) -> Result<Vec<BackendAccount>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider, alias, email, status, credential_ref
         FROM backend_accounts
         ORDER BY provider ASC, COALESCE(last_used_at, '') ASC, id ASC",
    )?;
    let rows = stmt.query_map([], backend_account_from_row)?;

    let mut accounts = Vec::new();
    for row in rows {
        accounts.push(row?);
    }
    Ok(accounts)
}

pub fn find_backend_account(conn: &Connection, selector: &str) -> Result<Option<BackendAccount>> {
    let selector = selector.trim();
    if let Ok(display_number) = selector.parse::<usize>() {
        if display_number == 0 {
            return Ok(None);
        }

        let accounts = list_backend_accounts(conn)?;
        return Ok(accounts.into_iter().nth(display_number - 1));
    }

    let mut stmt = conn.prepare(
        "SELECT id, provider, alias, email, status, credential_ref
         FROM backend_accounts
         WHERE alias = ?1 OR email = ?1
         ORDER BY id ASC
         LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![selector], backend_account_from_row)?;
    rows.next().transpose()
}

pub fn find_backend_account_by_id(conn: &Connection, id: i64) -> Result<Option<BackendAccount>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider, alias, email, status, credential_ref
         FROM backend_accounts
         WHERE id = ?1
         LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![id], backend_account_from_row)?;
    rows.next().transpose()
}

pub fn select_active_backend_account(
    conn: &Connection,
    provider: &str,
    excluded_ids: &[i64],
) -> Result<Option<BackendAccount>> {
    let accounts = list_backend_accounts(conn)?;
    Ok(accounts.into_iter().find(|account| {
        account.provider == provider
            && account.status == "active"
            && !excluded_ids.contains(&account.id)
    }))
}

pub fn set_backend_account_status(conn: &Connection, selector: &str, status: &str) -> Result<()> {
    let Some(account) = find_backend_account(conn, selector)? else {
        println!(
            "{}",
            format!("Backend account '{}' tidak ditemukan.", selector).red()
        );
        return Ok(());
    };

    conn.execute(
        "UPDATE backend_accounts
         SET status = ?1, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?2",
        params![status, account.id],
    )?;

    println!(
        "{}",
        format!(
            "Backend account {}:{} sekarang {}.",
            account.provider, account.alias, status
        )
        .green()
    );
    Ok(())
}

pub fn set_backend_account_status_by_id(conn: &Connection, id: i64, status: &str) -> Result<()> {
    let Some(account) = find_backend_account_by_id(conn, id)? else {
        println!(
            "{}",
            format!("Backend account id '{}' tidak ditemukan.", id).red()
        );
        return Ok(());
    };

    conn.execute(
        "UPDATE backend_accounts
         SET status = ?1, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?2",
        params![status, account.id],
    )?;

    println!(
        "{}",
        format!(
            "Backend account {}:{} sekarang {}.",
            account.provider, account.alias, status
        )
        .green()
    );
    Ok(())
}

pub fn remove_backend_account(conn: &Connection, selector: &str) -> Result<()> {
    let Some(account) = find_backend_account(conn, selector)? else {
        println!(
            "{}",
            format!("Backend account '{}' tidak ditemukan.", selector).red()
        );
        return Ok(());
    };

    conn.execute(
        "DELETE FROM backend_accounts WHERE id = ?1",
        params![account.id],
    )?;
    println!(
        "{}",
        format!(
            "Backend account {}:{} dihapus.",
            account.provider, account.alias
        )
        .green()
    );
    Ok(())
}

pub fn remove_backend_account_by_id(conn: &Connection, id: i64) -> Result<()> {
    let Some(account) = find_backend_account_by_id(conn, id)? else {
        println!(
            "{}",
            format!("Backend account id '{}' tidak ditemukan.", id).red()
        );
        return Ok(());
    };

    conn.execute(
        "DELETE FROM backend_accounts WHERE id = ?1",
        params![account.id],
    )?;
    println!(
        "{}",
        format!(
            "Backend account {}:{} dihapus.",
            account.provider, account.alias
        )
        .green()
    );
    Ok(())
}

pub fn mark_backend_account_used(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE backend_accounts
         SET last_used_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

fn backend_account_from_row(row: &rusqlite::Row<'_>) -> Result<BackendAccount> {
    Ok(BackendAccount {
        id: row.get(0)?,
        provider: row.get(1)?,
        alias: row.get(2)?,
        email: row.get(3)?,
        status: row.get(4)?,
        credential_ref: row.get(5)?,
    })
}

pub fn list_accounts(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT email FROM accounts ORDER BY id ASC")?;
    let account_iter = stmt.query_map([], |row| {
        let email: String = row.get(0)?;
        Ok(email)
    })?;

    println!("{}", "Daftar Akun:".cyan().bold());
    for (index, account) in account_iter.enumerate() {
        let email = account?;
        println!("{}. {}", index + 1, email);
    }
    Ok(())
}

pub fn get_account_by_number(conn: &Connection, number: usize) -> Result<Option<String>> {
    if number == 0 {
        return Ok(None);
    }

    let mut stmt = conn.prepare("SELECT email FROM accounts ORDER BY id ASC LIMIT 1 OFFSET ?1")?;
    let mut rows = stmt.query([(number - 1) as i64])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn delete_account(conn: &Connection, email: &str) -> Result<()> {
    match conn.execute("DELETE FROM accounts WHERE email = ?1", [&email]) {
        Ok(0) => println!(
            "{}",
            format!("Akun {} tidak ditemukan di database.", email).yellow()
        ),
        Ok(_) => {
            let token_cache_path = config::get_token_cache_path(email);
            let _ = std::fs::remove_file(token_cache_path);

            println!(
                "{}",
                format!(
                    "Sukses! Akun {} dan tokennya telah dihapus secara permanen.",
                    email
                )
                .green()
            );
        }
        Err(e) => println!("{}", format!("Gagal menghapus akun: {}", e).red()),
    }
    Ok(())
}
