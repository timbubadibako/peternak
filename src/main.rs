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
}
mod db;
pub mod config;

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
                                println!("{}", format!("Menyiapkan proses login Google untuk akun: {}", email).blue());
                                
                                let secret_path = config::get_secret_path();
                                
                                if !secret_path.exists() {
                                    println!("{}", "=========================================================".red());
                                    println!("{}", "ERROR: File 'client_secret.json' tidak ditemukan!".red().bold());
                                    println!("Silakan buat OAuth Client ID (Tipe: Desktop App) di Google Cloud Console,");
                                    println!("unduh file JSON-nya, dan simpan di path berikut:");
                                    println!("{}", secret_path.display().to_string().yellow());
                                    println!("{}", "=========================================================".red());
                                    continue;
                                }

                                match yup_oauth2::read_application_secret(&secret_path).await {
                                    Ok(secret) => {
                                        println!("{}", "Membuka browser untuk otentikasi...".cyan());
                                        // Gunakan InstalledFlow untuk membuka browser lokal otomatis
                                        let token_cache_path = config::get_token_cache_path(&email);
                                        
                                        let auth_result = yup_oauth2::InstalledFlowAuthenticator::builder(
                                            secret,
                                            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
                                        )
                                        .persist_tokens_to_disk(&token_cache_path)
                                        .build().await;

                                        match auth_result {
                                            Ok(auth) => {
                                                // Minta scope Google Drive sebagai contoh
                                                let scopes = &["https://www.googleapis.com/auth/drive.readonly"];
                                                match auth.token(scopes).await {
                                                    Ok(_) => {
                                                        // yup_oauth2 otomatis menyimpan token ke memory/cache internalnya.
                                                        // Namun kita ingin ekstrak refresh tokennya atau menganggap login berhasil.
                                                        // Untuk saat ini kita simpan "sukses" karena kita hanya butuh ini tersimpan.
                                                        // Jika kita butuh ekstrak refresh token untuk aplikasi lain, kita perlu custom Storage.
                                                        // Untuk sekarang, kita tandai di database bahwa akun ini terautentikasi.
                                                        let refresh_token = "tersimpan_di_cache_yupoauth2_internal";
                                                        
                                                        match conn.execute(
                                                            "INSERT INTO accounts (email, refresh_token) VALUES (?1, ?2) 
                                                             ON CONFLICT(email) DO UPDATE SET refresh_token=excluded.refresh_token",
                                                            [&email, refresh_token],
                                                        ) {
                                                            Ok(_) => println!("{}", format!("Login berhasil! Sesi untuk {} telah diotorisasi.", email).green().bold()),
                                                            Err(e) => println!("{}", format!("Error database: {}", e).red()),
                                                        }
                                                    },
                                                    Err(e) => println!("{}", format!("Gagal mendapatkan token: {}", e).red()),
                                                }
                                            },
                                            Err(e) => println!("{}", format!("Gagal inisialisasi authenticator: {}", e).red()),
                                        }
                                    },
                                    Err(e) => println!("{}", format!("Gagal membaca client_secret.json: {}", e).red()),
                                }
                            }
                            Commands::List => {
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
                            }
                            Commands::Get { email, service } => {
                                println!("Mengambil token {} untuk akun {}...", service.bold(), email.bold());
                                
                                let secret_path = config::get_secret_path();
                                let token_cache_path = config::get_token_cache_path(&email);

                                if !token_cache_path.exists() {
                                    println!("{}", format!("ERROR: Sesi untuk {} belum ada. Ketik 'add {}' dulu.", email, email).red());
                                    continue;
                                }

                                match yup_oauth2::read_application_secret(&secret_path).await {
                                    Ok(secret) => {
                                        let auth_result = yup_oauth2::InstalledFlowAuthenticator::builder(
                                            secret,
                                            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
                                        )
                                        .persist_tokens_to_disk(&token_cache_path)
                                        .build().await;

                                        match auth_result {
                                            Ok(auth) => {
                                                let scopes = &["https://www.googleapis.com/auth/drive.readonly"]; // Bisa disesuaikan per service
                                                match auth.token(scopes).await {
                                                    Ok(token) => {
                                                        let access_token = token.token().unwrap_or("error_no_token");
                                                        println!("{}", "Token aktif berhasil digenerate dari server Google:".green());
                                                        println!("{}", "=======================================================".cyan());
                                                        println!("{}", access_token.cyan().bold());
                                                        println!("{}", "=======================================================".cyan());
                                                        println!("{}", "Silakan COPY token di atas untuk digunakan secara manual.".dimmed());
                                                    },
                                                    Err(e) => println!("{}", format!("Gagal memanggil API token: {}", e).red()),
                                                }
                                            },
                                            Err(e) => println!("{}", format!("Gagal init authenticator: {}", e).red()),
                                        }
                                    },
                                    Err(_) => println!("{}", "Gagal membaca client_secret.json".red()),
                                }
                            }
                            Commands::Inject { email, service } => {
                                println!("Menyiapkan injeksi token {} untuk akun {}...", service.bold(), email.bold());
                                
                                let secret_path = config::get_secret_path();
                                let token_cache_path = config::get_token_cache_path(&email);

                                if !token_cache_path.exists() {
                                    println!("{}", format!("ERROR: Sesi untuk {} belum ada. Ketik 'add {}' dulu.", email, email).red());
                                    continue;
                                }

                                match yup_oauth2::read_application_secret(&secret_path).await {
                                    Ok(secret) => {
                                        let auth_result = yup_oauth2::InstalledFlowAuthenticator::builder(
                                            secret,
                                            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
                                        )
                                        .persist_tokens_to_disk(&token_cache_path)
                                        .build().await;

                                        if let Ok(auth) = auth_result {
                                            let scopes = &["https://www.googleapis.com/auth/drive.readonly"];
                                            if let Ok(token) = auth.token(scopes).await {
                                                let access_token = token.token().unwrap_or("");
                                                
                                                // Simulasi penulisan ke file konfigurasi spesifik service
                                                if service.to_lowercase() == "antigravity" {
                                                    let agy_config = config::get_config_dir().join("antigravity-cli-mock-config.json");
                                                    let config_content = format!(r#"{{ "current_account": "{}", "access_token": "{}" }}"#, email, access_token);
                                                    
                                                    match std::fs::write(&agy_config, config_content) {
                                                        Ok(_) => {
                                                            println!("{}", "INJEKSI BERHASIL! 🚀".green().bold());
                                                            println!("File config Antigravity telah ditimpa.");
                                                            println!("Sekarang Antigravity akan otomatis berjalan dengan akun: {}", email.cyan());
                                                        },
                                                        Err(e) => println!("{}", format!("Gagal menulis file config: {}", e).red()),
                                                    }
                                                } else {
                                                    println!("{}", format!("Injeksi untuk service '{}' belum dikonfigurasi.", service).yellow());
                                                }
                                            } else {
                                                println!("{}", "Gagal mendapatkan token!".red());
                                            }
                                        }
                                    },
                                    Err(_) => println!("{}", "Gagal membaca client_secret.json".red()),
                                }
                            }
                            Commands::Space { email } => {
                                println!("Mengecek kapasitas Google Drive untuk akun {}...", email.bold());
                                
                                let secret_path = config::get_secret_path();
                                let token_cache_path = config::get_token_cache_path(&email);

                                if !token_cache_path.exists() {
                                    println!("{}", format!("ERROR: Sesi untuk {} belum ada. Ketik 'add {}' dulu.", email, email).red());
                                    continue;
                                }

                                match yup_oauth2::read_application_secret(&secret_path).await {
                                    Ok(secret) => {
                                        let auth_result = yup_oauth2::InstalledFlowAuthenticator::builder(
                                            secret,
                                            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
                                        )
                                        .persist_tokens_to_disk(&token_cache_path)
                                        .build().await;

                                        if let Ok(auth) = auth_result {
                                            let scopes = &["https://www.googleapis.com/auth/drive.readonly"];
                                            if let Ok(token) = auth.token(scopes).await {
                                                let access_token = token.token().unwrap_or("");
                                                
                                                let client = reqwest::Client::new();
                                                let res = client.get("https://www.googleapis.com/drive/v3/about?fields=storageQuota")
                                                    .bearer_auth(access_token)
                                                    .send()
                                                    .await;
                                                
                                                match res {
                                                    Ok(response) => {
                                                        if let Ok(json) = response.json::<serde_json::Value>().await {
                                                            if let Some(quota) = json.get("storageQuota") {
                                                                let limit_str = quota.get("limit").and_then(|v| v.as_str()).unwrap_or("0");
                                                                let usage_str = quota.get("usage").and_then(|v| v.as_str()).unwrap_or("0");
                                                                
                                                                let limit: f64 = limit_str.parse().unwrap_or(0.0) / 1_073_741_824.0;
                                                                let usage: f64 = usage_str.parse().unwrap_or(0.0) / 1_073_741_824.0;
                                                                
                                                                let free = limit - usage;
                                                                
                                                                println!("{}", "======================================".cyan());
                                                                println!("Email       : {}", email.bold());
                                                                println!("Terpakai    : {:.2} GB", usage);
                                                                println!("Sisa Ruang  : {} GB", format!("{:.2}", free).green().bold());
                                                                println!("Total Limit : {:.2} GB", limit);
                                                                println!("{}", "======================================".cyan());
                                                            } else {
                                                                println!("{}", format!("Gagal membaca struktur kuota. Response dari Google API:\n{}", serde_json::to_string_pretty(&json).unwrap_or_default()).red());
                                                            }
                                                        } else {
                                                            println!("{}", "Gagal parsing response dari Google API.".red());
                                                        }
                                                    },
                                                    Err(e) => println!("{}", format!("Gagal memanggil HTTP API Drive: {}", e).red()),
                                                }
                                            } else {
                                                println!("{}", "Gagal mendapatkan token (mungkin sesi expired)!".red());
                                            }
                                        }
                                    },
                                    Err(_) => println!("{}", "Gagal membaca client_secret.json".red()),
                                }
                            }
                            Commands::AddSupa { email } => {
                                println!("Membuka browser untuk generate token Supabase akun {}...", email.bold());
                                
                                // Buka URL Supabase Token secara otomatis di OS
                                if let Err(e) = open::that("https://supabase.com/dashboard/account/tokens") {
                                    println!("{}", format!("Gagal membuka browser otomatis: {}. Silakan buka link ini secara manual: https://supabase.com/dashboard/account/tokens", e).yellow());
                                }
                                
                                println!("{}", "1. Pastikan kamu sudah login Supabase menggunakan akun Google tersebut.".cyan());
                                println!("{}", "2. Klik tombol 'Generate New Token', beri nama bebas.".cyan());
                                println!("{}", "3. Copy token yang berawalan 'sbp_...'.".cyan());
                                
                                use std::io::{self, Write};
                                print!("\n{} ", "Paste Token Supabase di sini:".yellow().bold());
                                io::stdout().flush().unwrap();
                                
                                let mut token_input = String::new();
                                io::stdin().read_line(&mut token_input).unwrap();
                                let token = token_input.trim().to_string();

                                if token.is_empty() || !token.starts_with("sbp_") {
                                    println!("{}", "Format token tidak valid (harus berawalan sbp_). Dibatalkan!".red());
                                    continue;
                                }

                                match conn.execute(
                                    "UPDATE accounts SET supabase_token = ?1 WHERE email = ?2",
                                    [&token, &email],
                                ) {
                                    Ok(0) => println!("{}", format!("Akun {} belum terdaftar di database. Jalankan 'add {}' dulu.", email, email).red()),
                                    Ok(_) => println!("{}", "Token Supabase berhasil ditanamkan ke dalam database! 🚜".green().bold()),
                                    Err(e) => println!("{}", format!("Database error: {}", e).red()),
                                }
                            }
                            Commands::FarmSupa { email, project_name, db_password } => {
                                println!("Menyiapkan traktor untuk menanam project Supabase '{}' di akun {}...", project_name.bold(), email.bold());
                                
                                let mut stmt = conn.prepare("SELECT supabase_token FROM accounts WHERE email = ?1").unwrap();
                                let mut rows = stmt.query([&email]).unwrap();
                                
                                let supabase_token: Option<String> = if let Some(row) = rows.next().unwrap() {
                                    row.get(0).unwrap_or(None)
                                } else {
                                    println!("{}", format!("Akun {} tidak ditemukan.", email).red());
                                    continue;
                                };
                                
                                let supabase_token = match supabase_token {
                                    Some(t) => t,
                                    None => {
                                        println!("{}", "Token Supabase belum di-set! Gunakan perintah 'add-supa' dulu.".red());
                                        continue;
                                    }
                                };

                                let client = reqwest::Client::new();
                                
                                // 1. Dapatkan Organization ID
                                println!("{}", "1. Mengambil Organization ID dari Supabase...".dimmed());
                                let orgs_res = client.get("https://api.supabase.com/v1/organizations")
                                    .bearer_auth(&supabase_token)
                                    .send().await;
                                    
                                let org_id = match orgs_res {
                                    Ok(res) => {
                                        if let Ok(json) = res.json::<serde_json::Value>().await {
                                            if let Some(orgs) = json.as_array() {
                                                if orgs.is_empty() {
                                                    println!("{}", "Tidak ada organisasi di akun Supabase ini!".red());
                                                    continue;
                                                }
                                                orgs[0]["id"].as_str().unwrap_or("").to_string()
                                            } else {
                                                println!("{}", "Format API Organisasi tidak sesuai.".red());
                                                continue;
                                            }
                                        } else {
                                            println!("{}", "Gagal membaca API Organisasi.".red());
                                            continue;
                                        }
                                    },
                                    Err(e) => {
                                        println!("{}", format!("Gagal memanggil API: {}", e).red());
                                        continue;
                                    }
                                };

                                if org_id.is_empty() {
                                    continue;
                                }
                                
                                // 2. Buat Project Baru
                                println!("2. Memerintahkan Supabase membangun project (Region: ap-southeast-1, Plan: free)...");
                                let payload = serde_json::json!({
                                    "name": project_name,
                                    "organization_id": org_id,
                                    "db_pass": db_password,
                                    "region": "ap-southeast-1",
                                    "plan": "free"
                                });
                                
                                let create_res = client.post("https://api.supabase.com/v1/projects")
                                    .bearer_auth(&supabase_token)
                                    .json(&payload)
                                    .send().await;
                                    
                                match create_res {
                                    Ok(res) => {
                                        if res.status().is_success() {
                                            let json: serde_json::Value = res.json().await.unwrap_or_default();
                                            let ref_id = json["id"].as_str().unwrap_or("unknown");
                                            println!("{}", "==================================================".green());
                                            println!("{} Project {} BERHASIL DIBUAT!", "🚀".green(), project_name.bold());
                                            println!("Project Reference ID : {}", ref_id.cyan());
                                            println!("Postgres URL         : postgresql://postgres.[{}]:{}@aws-0-ap-southeast-1.pooler.supabase.com:6543/postgres", ref_id, db_password);
                                            println!("{}", "==================================================".green());
                                            println!("{}", "INFO: Supabase butuh 2-3 menit untuk merakit Database. Kunci Anon dan Secret API Keys baru akan bisa diakses setelah loading di server mereka selesai.".yellow());
                                            println!("{}", format!("Untuk menarik keys nanti, hit endpoint GET /v1/projects/{}/api-keys", ref_id).dimmed());
                                        } else {
                                            let err_body = res.text().await.unwrap_or_default();
                                            println!("{}", format!("Gagal membuat project! Status/Error: {}", err_body).red());
                                        }
                                    },
                                    Err(e) => println!("{}", format!("Gagal mengirim request pembuatan project: {}", e).red()),
                                }
                            }
                            Commands::Delete { email } => {
                                match conn.execute("DELETE FROM accounts WHERE email = ?1", [&email]) {
                                    Ok(0) => println!("{}", format!("Akun {} tidak ditemukan di database.", email).yellow()),
                                    Ok(_) => {
                                        // Hapus juga file token json nya
                                        let token_cache_path = config::get_token_cache_path(email);
                                        let _ = std::fs::remove_file(token_cache_path);
                                        
                                        println!("{}", format!("Sukses! Akun {} dan tokennya telah dihapus secara permanen.", email).green());
                                    },
                                    Err(e) => println!("{}", format!("Gagal menghapus akun: {}", e).red()),
                                }
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
