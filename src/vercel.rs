use rusqlite::Connection;
use colored::*;

pub fn handle_add_vercel(conn: &Connection, email: &str) {
    println!("Membuka browser untuk generate token Vercel akun {}...", email.bold());
    
    if let Err(e) = open::that("https://vercel.com/account/tokens") {
        println!("{}", format!("Gagal membuka browser otomatis: {}. Silakan buka link ini secara manual: https://vercel.com/account/tokens", e).yellow());
    }
    
    println!("{}", "1. Pastikan kamu sudah login Vercel menggunakan akun Google tersebut.".cyan());
    println!("{}", "2. Klik tombol 'Create', beri nama (misal: peternak) dan scope 'Full Account'.".cyan());
    println!("{}", "3. Copy token yang muncul.".cyan());
    
    use std::io::{self, Write};
    print!("\n{} ", "Paste Token Vercel di sini:".yellow().bold());
    io::stdout().flush().unwrap();
    
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input).unwrap();
    let token = token_input.trim().to_string();

    if token.is_empty() {
        println!("{}", "Token kosong. Dibatalkan!".red());
        return;
    }

    match conn.execute(
        "UPDATE accounts SET vercel_token = ?1 WHERE email = ?2",
        [&token, &email],
    ) {
        Ok(0) => println!("{}", format!("Akun {} belum terdaftar di database. Jalankan 'add {}' dulu.", email, email).red()),
        Ok(_) => println!("{}", "Token Vercel berhasil ditanamkan ke dalam database! 🚜".green().bold()),
        Err(e) => println!("{}", format!("Database error: {}", e).red()),
    }
}

pub async fn handle_farm_vercel(conn: &Connection, email: &str, project_name: &str) {
    println!("Menyiapkan traktor untuk deploy project Vercel '{}' di akun {}...", project_name.bold(), email.bold());
    
    let mut stmt = conn.prepare("SELECT vercel_token FROM accounts WHERE email = ?1").unwrap();
    let mut rows = stmt.query([&email]).unwrap();
    
    let vercel_token: Option<String> = if let Some(row) = rows.next().unwrap() {
        row.get(0).unwrap_or(None)
    } else {
        println!("{}", format!("Akun {} tidak ditemukan.", email).red());
        return;
    };
    
    let vercel_token = match vercel_token {
        Some(t) => t,
        None => {
            println!("{}", "Token Vercel belum di-set! Gunakan perintah 'add-vercel' dulu.".red());
            return;
        }
    };

    let client = reqwest::Client::new();
    
    // Test token validity by fetching user info
    println!("{}", "1. Mengecek kredensial Vercel...".dimmed());
    let user_res = client.get("https://api.vercel.com/v2/user")
        .bearer_auth(&vercel_token)
        .send().await;
        
    match user_res {
        Ok(res) => {
            if !res.status().is_success() {
                println!("{}", "Token Vercel invalid atau expired!".red());
                return;
            }
        },
        Err(e) => {
            println!("{}", format!("Gagal memanggil API Vercel: {}", e).red());
            return;
        }
    };

    println!("2. Memerintahkan Vercel membangun project...");
    let payload = serde_json::json!({
        "name": project_name,
        "framework": "nextjs",
        "gitRepository": {
            "type": "github",
            "repo": "timbubadibako/peternak"
        }
    });
    
    let create_res = client.post("https://api.vercel.com/v9/projects")
        .bearer_auth(&vercel_token)
        .json(&payload)
        .send().await;
        
    match create_res {
        Ok(res) => {
            if res.status().is_success() {
                let json: serde_json::Value = res.json().await.unwrap_or_default();
                let ref_id = json["id"].as_str().unwrap_or("unknown");
                println!("{}", "==================================================".green());
                println!("{} Project {} BERHASIL DIDAFTARKAN DI VERCEL!", "🚀".green(), project_name.bold());
                println!("Project ID : {}", ref_id.cyan());
                println!("{}", "==================================================".green());
            } else {
                let err_body = res.text().await.unwrap_or_default();
                println!("{}", format!("Gagal membuat project di Vercel! Status/Error: {}", err_body).red());
            }
        },
        Err(e) => println!("{}", format!("Gagal mengirim request pembuatan project: {}", e).red()),
    }
}
