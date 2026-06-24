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
