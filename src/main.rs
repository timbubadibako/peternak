use clap::Parser;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Parser, Debug)]
#[command(name = "")]
#[command(about = "CLI Tool untuk menanam dan memanen token", long_about = None)]
#[command(no_binary_name = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Tambah akun Google (Alias: a)
    #[command(alias = "a")]
    Add {
        /// Email akun Google
        email: String,
    },
    /// Lihat daftar akun yang tersimpan (Alias: ls)
    #[command(alias = "ls")]
    List,
    /// Ambil token untuk service tertentu (Alias: g)
    #[command(alias = "g")]
    Get {
        /// Email akun Google
        email: String,
        /// Service tujuan (vercel, gdrive, supabase, antigravity)
        service: String,
    },
    /// Hapus akun dari database lokal (Alias: rm, del)
    #[command(alias = "rm", alias = "del")]
    Delete {
        /// Email akun yang ingin dihapus
        email: String,
    },
    /// Injeksi token langsung ke file config aplikasi (Alias: inj)
    #[command(alias = "inj")]
    Inject {
        /// Email akun Google
        email: String,
        /// Service tujuan (vercel, antigravity)
        service: String,
    },
    /// Cek sisa kapasitas penyimpanan Google Drive (Alias: space, quota)
    #[command(alias = "quota")]
    Space {
        /// Email akun Google
        email: String,
    },
    /// Tambah Supabase Access Token ke akun secara interaktif (Alias: add-supa)
    AddSupa {
        /// Email akun Google
        email: String,
    },
    /// Ternak project baru di Supabase dan ambil kuncinya (Alias: farm-supa)
    FarmSupa {
        /// Email akun Google
        email: String,
        /// Nama Project Supabase
        project_name: String,
        /// Password Database (minimal 8 karakter)
        db_password: String,
    },
    /// Keluar dari aplikasi
    #[command(alias = "q", alias = "exit")]
    Quit,
mod db;
pub mod config;
pub mod supabase;
pub mod google;

fn print_hacker_logo() {
    let logo = r#"
░▒▓███████▓▒░░▒▓████████▓▒░▒▓████████▓▒░▒▓████████▓▒░▒▓███████▓▒░░▒▓███████▓▒░ ░▒▓██████▓▒░░▒▓█▓▒░░▒▓█▓▒░ 
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░         ░▒▓█▓▒░   ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ 
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░         ░▒▓█▓▒░   ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ 
░▒▓███████▓▒░░▒▓██████▓▒░    ░▒▓█▓▒░   ░▒▓██████▓▒░ ░▒▓███████▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓████████▓▒░▒▓███████▓▒░  
░▒▓█▓▒░      ░▒▓█▓▒░         ░▒▓█▓▒░   ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ 
░▒▓█▓▒░      ░▒▓█▓▒░         ░▒▓█▓▒░   ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ 
░▒▓█▓▒░      ░▒▓████████▓▒░  ░▒▓█▓▒░   ░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ 
"#;
    println!("{}", logo.green().bold());
    println!("{}", "Welcome to Peternak-AIAI Interactive Shell".green());
    println!("{}", "Ketik 'help' untuk melihat daftar command. Ketik 'quit' atau 'exit' untuk keluar.\n".dimmed());
}

#[tokio::main]
async fn main() -> Result<()> {
    print_hacker_logo();

    let conn = db::init_db()?;
    let mut rl = DefaultEditor::new().expect("Failed to initialize rustyline");

    loop {
        let readline = rl.readline("peternak> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                rl.add_history_entry(line).unwrap();

                // Split input ke bentuk vector string (contoh: "add a@g.com" -> ["add", "a@g.com"])
                let args = shlex::split(line).unwrap_or_else(|| vec![line.to_string()]);

                match Cli::try_parse_from(args) {
                    Ok(cli) => {
                        match cli.command {
                            Commands::Add { email } => {
                                google::handle_add(&conn, &email).await;
                            }
                            Commands::List => {
                                let _ = db::list_accounts(&conn);
                            }
                            Commands::Get { email, service } => {
                                google::handle_get(&conn, &email, &service).await;
                            }
                            Commands::Inject { email, service } => {
                                google::handle_inject(&conn, &email, &service).await;
                            }
                            Commands::Space { email } => {
                                google::handle_space(&conn, &email).await;
                            }
                            Commands::AddSupa { email } => {
                                supabase::handle_add_supa(&conn, &email);
                            }
                            Commands::FarmSupa { email, project_name, db_password } => {
                                supabase::handle_farm_supa(&conn, &email, &project_name, &db_password).await;
                            }
                            Commands::Delete { email } => {
                                let _ = db::delete_account(&conn, &email);
                            }
                            Commands::Quit => {
                                println!("{}", "Exiting...".dimmed());
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        // Print help atau error message dari clap
                        err.print().unwrap();
                        println!();
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("{}", "Exiting...".dimmed());
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
