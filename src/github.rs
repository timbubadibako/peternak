use rusqlite::Connection;
use colored::*;

pub fn handle_add_github(conn: &Connection, email: &str) {
    println!("Membuka browser untuk generate token GitHub akun {}...", email.bold());
    
    if let Err(e) = open::that("https://github.com/settings/tokens/new") {
        println!("{}", format!("Gagal membuka browser otomatis: {}. Silakan buka link ini secara manual: https://github.com/settings/tokens/new", e).yellow());
    }
    
    println!("{}", "1. Pastikan kamu sudah login GitHub menggunakan akun tersebut.".cyan());
    println!("{}", "2. Centang scope 'repo' dan 'workflow'.".cyan());
    println!("{}", "3. Generate token dan copy token yang berawalan 'ghp_'.".cyan());
    
    use std::io::{self, Write};
    print!("\n{} ", "Paste Token GitHub di sini:".yellow().bold());
    io::stdout().flush().unwrap();
    
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input).unwrap();
    let token = token_input.trim().to_string();

    if token.is_empty() {
        println!("{}", "Token kosong. Dibatalkan!".red());
        return;
    }

    match conn.execute(
        "UPDATE accounts SET github_token = ?1 WHERE email = ?2",
        [&token, &email],
    ) {
        Ok(0) => println!("{}", format!("Akun {} belum terdaftar di database. Jalankan 'add {}' dulu.", email, email).red()),
        Ok(_) => println!("{}", "Token GitHub berhasil ditanamkan ke dalam database! 🚜".green().bold()),
        Err(e) => println!("{}", format!("Database error: {}", e).red()),
    }
}

pub async fn handle_farm_github(conn: &Connection, email: &str, repo_name: &str) {
    println!("Menyiapkan traktor untuk membuat repository GitHub '{}' di akun {}...", repo_name.bold(), email.bold());
    
    let mut stmt = conn.prepare("SELECT github_token FROM accounts WHERE email = ?1").unwrap();
    let mut rows = stmt.query([&email]).unwrap();
    
    let github_token: Option<String> = if let Some(row) = rows.next().unwrap() {
        row.get(0).unwrap_or(None)
    } else {
        println!("{}", format!("Akun {} tidak ditemukan.", email).red());
        return;
    };
    
    let github_token = match github_token {
        Some(t) => t,
        None => {
            println!("{}", "Token GitHub belum di-set! Gunakan perintah 'add-github' dulu.".red());
            return;
        }
    };

    let client = reqwest::Client::new();
    
    println!("Memerintahkan GitHub membangun repository...");
    let payload = serde_json::json!({
        "name": repo_name,
        "private": true,
        "auto_init": true
    });
    
    let create_res = client.post("https://api.github.com/user/repos")
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "peternak-aiai")
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send().await;
        
    match create_res {
        Ok(res) => {
            if res.status().is_success() {
                let json: serde_json::Value = res.json().await.unwrap_or_default();
                let html_url = json["html_url"].as_str().unwrap_or("unknown");
                println!("{}", "==================================================".green());
                println!("{} Repository {} BERHASIL DIBUAT DI GITHUB!", "🚀".green(), repo_name.bold());
                println!("URL : {}", html_url.cyan());
                println!("{}", "==================================================".green());
            } else {
                let err_body = res.text().await.unwrap_or_default();
                println!("{}", format!("Gagal membuat repository di GitHub! Status/Error: {}", err_body).red());
            }
        },
        Err(e) => println!("{}", format!("Gagal mengirim request pembuatan repository: {}", e).red()),
    }
}
