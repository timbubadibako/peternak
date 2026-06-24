use rusqlite::{Connection, Result};
use std::path::PathBuf;

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
    
    // Auto-update schema
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN supabase_token TEXT", []);

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

    Ok(conn)
}

use colored::*;
use crate::config;

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

pub fn delete_account(conn: &Connection, email: &str) -> Result<()> {
    match conn.execute("DELETE FROM accounts WHERE email = ?1", [&email]) {
        Ok(0) => println!("{}", format!("Akun {} tidak ditemukan di database.", email).yellow()),
        Ok(_) => {
            let token_cache_path = config::get_token_cache_path(email);
            let _ = std::fs::remove_file(token_cache_path);
            
            println!("{}", format!("Sukses! Akun {} dan tokennya telah dihapus secara permanen.", email).green());
        },
        Err(e) => println!("{}", format!("Gagal menghapus akun: {}", e).red()),
    }
    Ok(())
}
