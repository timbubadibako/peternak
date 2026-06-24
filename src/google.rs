use rusqlite::Connection;
use colored::*;
use crate::config;

pub async fn handle_add(conn: &Connection, email: &str) {
    println!("{}", format!("Menyiapkan proses login Google untuk akun: {}", email).blue());
    
    let secret_path = config::get_secret_path();
    
    if !secret_path.exists() {
        println!("{}", "=========================================================".red());
        println!("{}", "ERROR: File 'client_secret.json' tidak ditemukan!".red().bold());
        println!("Silakan buat OAuth Client ID (Tipe: Desktop App) di Google Cloud Console,");
        println!("unduh file JSON-nya, dan simpan di path berikut:");
        println!("{}", secret_path.display().to_string().yellow());
        println!("{}", "=========================================================".red());
        return;
    }

    match yup_oauth2::read_application_secret(&secret_path).await {
        Ok(secret) => {
            println!("{}", "Membuka browser untuk otentikasi...".cyan());
            let token_cache_path = config::get_token_cache_path(&email);
            
            let auth_result = yup_oauth2::InstalledFlowAuthenticator::builder(
                secret,
                yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
            )
            .persist_tokens_to_disk(&token_cache_path)
            .build().await;

            match auth_result {
                Ok(auth) => {
                    let scopes = &["https://www.googleapis.com/auth/drive.readonly"];
                    match auth.token(scopes).await {
                        Ok(_) => {
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

pub async fn handle_get(_conn: &Connection, email: &str, service: &str) {
    println!("Mengambil token {} untuk akun {}...", service.bold(), email.bold());
    
    let secret_path = config::get_secret_path();
    let token_cache_path = config::get_token_cache_path(&email);

    if !token_cache_path.exists() {
        println!("{}", format!("ERROR: Sesi untuk {} belum ada. Ketik 'add {}' dulu.", email, email).red());
        return;
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
                    let scopes = &["https://www.googleapis.com/auth/drive.readonly"];
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

pub async fn handle_inject(_conn: &Connection, email: &str, service: &str) {
    println!("Menyiapkan injeksi token {} untuk akun {}...", service.bold(), email.bold());
    
    let secret_path = config::get_secret_path();
    let token_cache_path = config::get_token_cache_path(&email);

    if !token_cache_path.exists() {
        println!("{}", format!("ERROR: Sesi untuk {} belum ada. Ketik 'add {}' dulu.", email, email).red());
        return;
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

pub async fn handle_space(_conn: &Connection, email: &str) {
    println!("Mengecek kapasitas Google Drive untuk akun {}...", email.bold());
    
    let secret_path = config::get_secret_path();
    let token_cache_path = config::get_token_cache_path(&email);

    if !token_cache_path.exists() {
        println!("{}", format!("ERROR: Sesi untuk {} belum ada. Ketik 'add {}' dulu.", email, email).red());
        return;
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
