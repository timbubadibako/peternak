use colored::*;
use rusqlite::Connection;

pub fn handle_add_supa(conn: &Connection, email: &str) {
    println!(
        "Membuka browser untuk generate token Supabase akun {}...",
        email.bold()
    );

    // Buka URL Supabase Token secara otomatis di OS
    if let Err(e) = open::that("https://supabase.com/dashboard/account/tokens") {
        println!("{}", format!("Gagal membuka browser otomatis: {}. Silakan buka link ini secara manual: https://supabase.com/dashboard/account/tokens", e).yellow());
    }

    println!(
        "{}",
        "1. Pastikan kamu sudah login Supabase menggunakan akun Google tersebut.".cyan()
    );
    println!(
        "{}",
        "2. Klik tombol 'Generate New Token', beri nama bebas.".cyan()
    );
    println!("{}", "3. Copy token yang berawalan 'sbp_...'.".cyan());

    use std::io::{self, Write};
    print!("\n{} ", "Paste Token Supabase di sini:".yellow().bold());
    io::stdout().flush().unwrap();

    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input).unwrap();
    let token = token_input.trim().to_string();

    if token.is_empty() || !token.starts_with("sbp_") {
        println!(
            "{}",
            "Format token tidak valid (harus berawalan sbp_). Dibatalkan!".red()
        );
        return;
    }

    match conn.execute(
        "UPDATE accounts SET supabase_token = ?1 WHERE email = ?2",
        [&token, &email],
    ) {
        Ok(0) => println!(
            "{}",
            format!(
                "Akun {} belum terdaftar di database. Jalankan 'add {}' dulu.",
                email, email
            )
            .red()
        ),
        Ok(_) => println!(
            "{}",
            "Token Supabase berhasil ditanamkan ke dalam database! 🚜"
                .green()
                .bold()
        ),
        Err(e) => println!("{}", format!("Database error: {}", e).red()),
    }
}

pub async fn handle_farm_supa(
    conn: &Connection,
    email: &str,
    project_name: &str,
    db_password: &str,
) {
    println!(
        "Menyiapkan traktor untuk menanam project Supabase '{}' di akun {}...",
        project_name.bold(),
        email.bold()
    );

    let mut stmt = conn
        .prepare("SELECT supabase_token FROM accounts WHERE email = ?1")
        .unwrap();
    let mut rows = stmt.query([&email]).unwrap();

    let supabase_token: Option<String> = if let Some(row) = rows.next().unwrap() {
        row.get(0).unwrap_or(None)
    } else {
        println!("{}", format!("Akun {} tidak ditemukan.", email).red());
        return;
    };

    let supabase_token = match supabase_token {
        Some(t) => t,
        None => {
            println!(
                "{}",
                "Token Supabase belum di-set! Gunakan perintah 'add-supa' dulu.".red()
            );
            return;
        }
    };

    let client = reqwest::Client::new();

    // 1. Dapatkan Organization ID
    println!(
        "{}",
        "1. Mengambil Organization ID dari Supabase...".dimmed()
    );
    let orgs_res = client
        .get("https://api.supabase.com/v1/organizations")
        .bearer_auth(&supabase_token)
        .send()
        .await;

    let org_id = match orgs_res {
        Ok(res) => {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if let Some(orgs) = json.as_array() {
                    if orgs.is_empty() {
                        println!("{}", "Tidak ada organisasi di akun Supabase ini!".red());
                        return;
                    }
                    orgs[0]["id"].as_str().unwrap_or("").to_string()
                } else {
                    println!("{}", "Format API Organisasi tidak sesuai.".red());
                    return;
                }
            } else {
                println!("{}", "Gagal membaca API Organisasi.".red());
                return;
            }
        }
        Err(e) => {
            println!("{}", format!("Gagal memanggil API: {}", e).red());
            return;
        }
    };

    if org_id.is_empty() {
        return;
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

    let create_res = client
        .post("https://api.supabase.com/v1/projects")
        .bearer_auth(&supabase_token)
        .json(&payload)
        .send()
        .await;

    match create_res {
        Ok(res) => {
            if res.status().is_success() {
                let json: serde_json::Value = res.json().await.unwrap_or_default();
                let ref_id = json["id"].as_str().unwrap_or("unknown");
                println!(
                    "{}",
                    "==================================================".green()
                );
                println!(
                    "{} Project {} BERHASIL DIBUAT!",
                    "🚀".green(),
                    project_name.bold()
                );
                println!("Project Reference ID : {}", ref_id.cyan());
                println!(
                    "Postgres URL         : postgresql://postgres.[{}]:{}@aws-0-ap-southeast-1.pooler.supabase.com:6543/postgres",
                    ref_id, db_password
                );
                println!(
                    "{}",
                    "==================================================".green()
                );
                println!("{}", "INFO: Supabase butuh 2-3 menit untuk merakit Database. Kunci Anon dan Secret API Keys baru akan bisa diakses setelah loading di server mereka selesai.".yellow());
                println!(
                    "{}",
                    format!(
                        "Untuk menarik keys nanti, hit endpoint GET /v1/projects/{}/api-keys",
                        ref_id
                    )
                    .dimmed()
                );
            } else {
                let err_body = res.text().await.unwrap_or_default();
                println!(
                    "{}",
                    format!("Gagal membuat project! Status/Error: {}", err_body).red()
                );
            }
        }
        Err(e) => println!(
            "{}",
            format!("Gagal mengirim request pembuatan project: {}", e).red()
        ),
    }
}
