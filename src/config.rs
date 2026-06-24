use std::path::PathBuf;

/// Mendapatkan root direktori konfigurasi (biasanya ~/.config di Linux)
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Mendapatkan path ke direktori peternak-aiai, membuat foldernya jika belum ada
pub fn get_app_dir() -> PathBuf {
    let mut p = get_config_dir();
    p.push("peternak-aiai");
    std::fs::create_dir_all(&p).ok();
    p
}

/// Mendapatkan path untuk file rahasia Google (client_secret.json)
pub fn get_secret_path() -> PathBuf {
    get_app_dir().join("client_secret.json")
}

/// Mendapatkan path untuk file cache token spesifik tiap email
pub fn get_token_cache_path(email: &str) -> PathBuf {
    get_app_dir().join(format!("token_{}.json", email))
}
