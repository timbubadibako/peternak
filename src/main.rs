use clap::Parser;
use colored::*;
use rusqlite::{Connection, Result};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};

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
    /// Capture secret Antigravity aktif dari keyring
    #[command(name = "capture-ag")]
    CaptureAg {
        /// Email atau alias akun
        account: String,
    },
    /// Switch Antigravity ke snapshot akun tertentu
    #[command(name = "switch-ag", alias = "inject-ag")]
    SwitchAg {
        /// Email atau alias akun
        account: String,
    },
    /// Lihat snapshot Antigravity yang tersimpan
    #[command(name = "list-ag")]
    ListAg,
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
    /// Tambah Vercel Access Token ke akun secara interaktif
    AddVercel {
        /// Email akun Google
        email: String,
    },
    /// Tanam Vercel Project
    #[command(name = "farm-vercel")]
    FarmVercel {
        /// Email akun
        email: String,
        /// Nama project Vercel
        project_name: String,
    },
    /// Tambah GitHub Access Token ke akun secara interaktif
    AddGithub {
        /// Email akun
        email: String,
    },
    /// Tanam Repository GitHub
    #[command(name = "farm-github")]
    FarmGithub {
        /// Email akun
        email: String,
        /// Nama repository
        repo_name: String,
    },
    /// Bersihkan layar (Clear screen)
    #[command(alias = "c", alias = "cls")]
    Clear,
    /// Keluar dari aplikasi
    #[command(alias = "q", alias = "exit")]
    Quit,
}

pub mod antigravity;
pub mod config;
pub mod credential_store;
mod db;
pub mod github;
pub mod google;
pub mod supabase;
pub mod vercel;

#[derive(Clone)]
struct Session {
    sso_id: String,
    role: db::Role,
}

struct AgyCommand {
    args: Vec<String>,
    account_selector: Option<String>,
}

struct SlashHelper;

impl Helper for SlashHelper {}
impl Highlighter for SlashHelper {}
impl Validator for SlashHelper {}

impl Hinter for SlashHelper {
    type Hint = String;
}

impl Completer for SlashHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let commands = [
            "/agy",
            "/agy -c",
            "/agy --continue",
            "/agy --conversation ",
            "/agy --account ",
            "/account setup agy ",
            "/account reset agy ",
            "/account clear agy",
            "/account add agy ",
            "/account add gcloud ",
            "/account add ",
            "/account list",
            "/account test ",
            "/account disable ",
            "/account remove ",
            "/status",
            "/help",
            "/quit",
        ];

        let prefix = &line[..pos];
        if !prefix.starts_with('/') {
            return Ok((0, Vec::new()));
        }

        let matches = commands
            .iter()
            .filter(|command| command.starts_with(prefix))
            .map(|command| Pair {
                display: command.to_string(),
                replacement: command.to_string(),
            })
            .collect::<Vec<_>>();

        Ok((0, matches))
    }
}

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
    println!(
        "{}",
        "Ketik 'help' untuk melihat daftar command. Ketik 'quit' atau 'exit' untuk keluar.\n"
            .dimmed()
    );
}

fn print_slash_help(role: &db::Role) {
    println!("{}", "Peternak slash commands:".cyan().bold());
    println!("  /agy                    Masuk ke agy interactive panel");
    println!("  /agy -c                 Resume conversation terakhir");
    println!("  /agy --conversation <id> Resume conversation spesifik");
    println!("  /status                 Lihat status session dan backend");
    println!("  /help                   Tampilkan bantuan");
    println!("  /quit                   Keluar dari Peternak");

    if role.is_admin() {
        println!();
        println!("{}", "Admin commands:".cyan().bold());
        println!("  /agy --account <email-or-id>");
        println!("  /account add <email>");
        println!("  /account setup agy <email>");
        println!("  /account reset agy <email-or-id>");
        println!("  /account clear agy");
        println!("  /account add gcloud <email>");
        println!("  /account list");
        println!("  /account test <id-or-alias>");
        println!("  /account disable <id-or-alias>");
        println!("  /account remove <id-or-alias>");
    }
}

fn require_admin(session: &Session) -> bool {
    if session.role.is_admin() {
        true
    } else {
        println!("{}", "Command ini hanya untuk admin.".red());
        false
    }
}

async fn handle_slash_command(
    conn: &Connection,
    session: &Session,
    agy_context: &mut antigravity::AgyContinuityContext,
    line: &str,
) -> bool {
    let args =
        shlex::split(line).unwrap_or_else(|| line.split_whitespace().map(str::to_string).collect());
    if args.is_empty() {
        return true;
    }

    match args[0].as_str() {
        "/help" => print_slash_help(&session.role),
        "/status" => print_status(conn, session),
        "/agy" => {
            if let Some(agy_command) = parse_agy_args(&args[1..]) {
                if agy_command.account_selector.is_some() && !require_admin(session) {
                    return true;
                }
                antigravity::run_interactive(
                    conn,
                    &agy_command.args,
                    agy_command.account_selector.as_deref(),
                    agy_context,
                );
            }
        }
        "/account" => handle_account_command(conn, session, &args[1..]).await,
        "/quit" | "/exit" => return false,
        other => println!(
            "{}",
            format!("Command slash tidak dikenal: {}", other).red()
        ),
    }

    true
}

fn parse_agy_args(args: &[String]) -> Option<AgyCommand> {
    match args {
        [] => Some(AgyCommand {
            args: Vec::new(),
            account_selector: None,
        }),
        [flag] if flag == "-c" || flag == "--continue" => Some(AgyCommand {
            args: vec![flag.clone()],
            account_selector: None,
        }),
        [flag, conversation_id] if flag == "--conversation" => Some(AgyCommand {
            args: vec![format!("--conversation={}", conversation_id)],
            account_selector: None,
        }),
        [flag] if flag.starts_with("--conversation=") => Some(AgyCommand {
            args: vec![flag.clone()],
            account_selector: None,
        }),
        [flag, selector] if flag == "--account" => Some(AgyCommand {
            args: Vec::new(),
            account_selector: Some(selector.clone()),
        }),
        [flag, selector, resume]
            if flag == "--account" && (resume == "-c" || resume == "--continue") =>
        {
            Some(AgyCommand {
                args: vec![resume.clone()],
                account_selector: Some(selector.clone()),
            })
        }
        _ => {
            println!("{}", "Format /agy tidak valid.".red());
            println!(
                "{}",
                "Coba: /agy, /agy -c, /agy --conversation <id>, atau /agy --account <email>"
                    .yellow()
            );
            None
        }
    }
}

async fn handle_account_command(conn: &Connection, session: &Session, args: &[String]) {
    if !require_admin(session) {
        return;
    }

    match args {
        [cmd, email] if cmd == "add" => {
            let _ = db::register_email_account(conn, email);
        }

        [cmd] if cmd == "list" => print_backend_accounts(conn),
        [cmd, provider, account] if cmd == "setup" && provider == "agy" => {
            let _ = db::register_email_account(conn, account);
            antigravity::handle_setup_login(conn, account);
        }

        [cmd, provider, account] if cmd == "reset" && provider == "agy" => {
            antigravity::handle_reset(conn, account);
        }

        [cmd, provider] if cmd == "clear" && provider == "agy" => {
            antigravity::handle_clear_active();
        }

        [cmd, provider, account] if cmd == "add" && provider == "agy" => {
            let _ = db::register_email_account(conn, account);
            antigravity::handle_setup_login(conn, account);
        }
        [cmd, provider, email] if cmd == "add" && provider == "gcloud" => {
            let _ = db::register_email_account(conn, email);
            google::handle_add(conn, email).await;
            let alias = email.split('@').next().unwrap_or(email);
            let credential_ref = config::get_token_cache_path(email)
                .to_string_lossy()
                .to_string();
            let _ =
                db::add_backend_account(conn, "gcloud", alias, Some(email), Some(&credential_ref));
        }
        [cmd, selector] if cmd == "test" => test_backend_account(conn, selector),
        [cmd, selector] if cmd == "disable" => {
            let _ = db::set_backend_account_status(conn, selector, "disabled");
        }
        [cmd, selector] if cmd == "remove" => {
            let _ = db::remove_backend_account(conn, selector);
        }
        _ => {
            println!("{}", "Format /account tidak valid.".red());
            println!(
                "{}",
                "Coba: /account add <email>, /account setup agy <email>, /account clear agy"
                    .yellow()
            );
        }
    }
}

fn print_backend_accounts(conn: &Connection) {
    match db::get_registered_emails(conn) {
        Ok(emails) if emails.is_empty() => println!("{}", "Belum ada email terdaftar.".yellow()),
        Ok(emails) => {
            println!("{}", "Registered Emails:".cyan().bold());
            for (index, email) in emails.iter().enumerate() {
                println!("{}. {}", index + 1, email);
            }
            println!();
        }
        Err(e) => println!("{}", format!("Gagal membaca email account: {}", e).red()),
    }

    match db::list_backend_accounts(conn) {
        Ok(accounts) if accounts.is_empty() => {
            println!("{}", "Belum ada backend account.".yellow());
        }
        Ok(accounts) => {
            println!("{}", "Backend Accounts:".cyan().bold());
            for (index, account) in accounts.into_iter().enumerate() {
                let email = account.email.unwrap_or_else(|| "-".to_string());
                println!(
                    "{:<3} {:<7} {:<18} {:<10} {}",
                    format!("{}.", index + 1),
                    account.provider,
                    account.alias,
                    account.status,
                    email
                );
            }
        }
        Err(e) => println!("{}", format!("Gagal membaca backend account: {}", e).red()),
    }
}

fn test_backend_account(conn: &Connection, selector: &str) {
    match db::find_backend_account(conn, selector) {
        Ok(Some(account)) if account.provider == "agy" => {
            antigravity::handle_switch(conn, selector)
        }
        Ok(Some(account)) => println!(
            "{}",
            format!(
                "Backend {}:{} terdaftar dengan status {}.",
                account.provider, account.alias, account.status
            )
            .green()
        ),
        Ok(None) => println!(
            "{}",
            format!("Backend account '{}' tidak ditemukan.", selector).red()
        ),
        Err(e) => println!("{}", format!("Gagal membaca backend account: {}", e).red()),
    }
}

fn print_status(conn: &Connection, session: &Session) {
    println!("{}", "Peternak Status".cyan().bold());
    println!("SSO ID : {}", session.sso_id);
    println!("Role   : {}", session.role.as_str());

    match db::list_backend_accounts(conn) {
        Ok(accounts) => {
            let agy = accounts
                .iter()
                .filter(|account| account.provider == "agy")
                .count();
            let gcloud = accounts
                .iter()
                .filter(|account| account.provider == "gcloud")
                .count();
            println!(
                "Backend: {} total, {} agy, {} gcloud",
                accounts.len(),
                agy,
                gcloud
            );
        }
        Err(e) => println!("{}", format!("Backend: gagal dibaca ({})", e).red()),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    print_hacker_logo();

    let conn = db::init_db()?;
    let mut rl: Editor<SlashHelper, DefaultHistory> =
        Editor::new().expect("Failed to initialize rustyline");
    rl.set_helper(Some(SlashHelper));

    let sso_id = match rl.readline("sso> ") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            println!("{}", "SSO ID wajib diisi.".red());
            return Ok(());
        }
    };

    if sso_id != "999" {
        println!("{}", "SSO ID tidak memiliki akses Peternak saat ini.".red());
        return Ok(());
    }

    let role = db::ensure_user(&conn, &sso_id)?;
    let session = Session { sso_id, role };
    let mut agy_context = antigravity::AgyContinuityContext::default();

    println!(
        "{}",
        format!("Session aktif sebagai {}.", session.role.as_str()).green()
    );
    print_slash_help(&session.role);

    loop {
        let readline = rl.readline("peternak> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line).unwrap();

                if line.starts_with('/') {
                    if !handle_slash_command(&conn, &session, &mut agy_context, line).await {
                        println!("{}", "Exiting...".dimmed());
                        break;
                    }
                    continue;
                }

                // Split input ke bentuk vector string (contoh: "add a@g.com" -> ["add", "a@g.com"])
                let args = shlex::split(line).unwrap_or_else(|| vec![line.to_string()]);

                match Cli::try_parse_from(args) {
                    Ok(cli) => match cli.command {
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
                        Commands::CaptureAg { account } => {
                            antigravity::handle_capture(&conn, &account);
                        }
                        Commands::SwitchAg { account } => {
                            antigravity::handle_switch(&conn, &account);
                        }
                        Commands::ListAg => {
                            antigravity::handle_list();
                        }
                        Commands::Space { email } => {
                            google::handle_space(&conn, &email).await;
                        }
                        Commands::AddSupa { email } => {
                            supabase::handle_add_supa(&conn, &email);
                        }
                        Commands::FarmSupa {
                            email,
                            project_name,
                            db_password,
                        } => {
                            supabase::handle_farm_supa(&conn, &email, &project_name, &db_password)
                                .await;
                        }
                        Commands::Delete { email } => {
                            let _ = db::delete_account(&conn, &email);
                        }
                        Commands::AddVercel { email } => {
                            vercel::handle_add_vercel(&conn, &email);
                        }
                        Commands::FarmVercel {
                            email,
                            project_name,
                        } => {
                            vercel::handle_farm_vercel(&conn, &email, &project_name).await;
                        }
                        Commands::AddGithub { email } => {
                            github::handle_add_github(&conn, &email);
                        }
                        Commands::FarmGithub { email, repo_name } => {
                            github::handle_farm_github(&conn, &email, &repo_name).await;
                        }
                        Commands::Clear => {
                            print!("{esc}c", esc = 27 as char);
                        }
                        Commands::Quit => {
                            println!("{}", "Exiting...".dimmed());
                            break;
                        }
                    },
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
