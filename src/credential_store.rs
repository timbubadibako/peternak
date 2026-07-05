use std::io::Write;
use std::process::{Command, Stdio};

const AGY_SERVICE: &str = "gemini";
const AGY_USERNAME: &str = "antigravity";
const AGY_LABEL: &str = "Password 'antigravity' on 'gemini'";

const APP_KEYRING_SERVICE_PREFIX: &str = "peternak-aiai";

pub type CredentialResult<T> = Result<T, String>;

pub fn read_active_agy_secret() -> CredentialResult<Vec<u8>> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("secret-tool")
            .args(["lookup", "service", AGY_SERVICE, "username", AGY_USERNAME])
            .output()
            .map_err(|e| format!("Gagal menjalankan secret-tool lookup: {e}"))?;

        if output.status.success() && !output.stdout.is_empty() {
            return Ok(output.stdout);
        }
    }

    let entry = agy_entry()?;
    entry
        .get_password()
        .map(|secret| secret.into_bytes())
        .map_err(|e| format!("Gagal membaca active agy keyring: {e}"))
}

pub fn write_active_agy_secret(secret: &[u8]) -> CredentialResult<()> {
    clear_active_agy_secret()?;

    #[cfg(target_os = "linux")]
    {
        let mut child = Command::new("secret-tool")
            .args([
                "store",
                "--label",
                AGY_LABEL,
                "service",
                AGY_SERVICE,
                "username",
                AGY_USERNAME,
            ])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Gagal menjalankan secret-tool store: {e}"))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(secret)
                .map_err(|e| format!("Gagal menulis secret ke secret-tool: {e}"))?;
        }

        let status = child
            .wait()
            .map_err(|e| format!("Gagal menunggu secret-tool store: {e}"))?;
        if status.success() {
            return Ok(());
        }
    }

    let secret = String::from_utf8(secret.to_vec())
        .map_err(|_| "Secret agy bukan UTF-8 valid.".to_string())?;
    let entry = agy_entry()?;
    entry
        .set_password(&secret)
        .map_err(|e| format!("Gagal menulis active agy keyring: {e}"))
}

pub fn clear_active_agy_secret() -> CredentialResult<()> {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("secret-tool")
            .args(["clear", "service", AGY_SERVICE, "username", AGY_USERNAME])
            .status();
    }

    let entry = agy_entry()?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Gagal menghapus active agy keyring: {e}")),
    }
}

pub fn store_peternak_secret(provider: &str, alias: &str, secret: &[u8]) -> CredentialResult<()> {
    let secret = String::from_utf8(secret.to_vec())
        .map_err(|_| "Secret Peternak bukan UTF-8 valid.".to_string())?;
    let entry = peternak_entry(provider, alias)?;
    entry
        .set_password(&secret)
        .map_err(|e| format!("Gagal menyimpan secret Peternak: {e}"))
}

pub fn read_peternak_secret(provider: &str, alias: &str) -> CredentialResult<Vec<u8>> {
    let entry = peternak_entry(provider, alias)?;
    entry
        .get_password()
        .map(|secret| secret.into_bytes())
        .map_err(|e| format!("Gagal membaca secret Peternak: {e}"))
}

pub fn delete_peternak_secret(provider: &str, alias: &str) -> CredentialResult<()> {
    let entry = peternak_entry(provider, alias)?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Gagal menghapus secret Peternak: {e}")),
    }
}

fn agy_entry() -> CredentialResult<keyring::Entry> {
    keyring::Entry::new(AGY_SERVICE, AGY_USERNAME)
        .map_err(|e| format!("Gagal membuka active agy keyring entry: {e}"))
}

fn peternak_entry(provider: &str, alias: &str) -> CredentialResult<keyring::Entry> {
    keyring::Entry::new(&format!("{APP_KEYRING_SERVICE_PREFIX}/{provider}"), alias)
        .map_err(|e| format!("Gagal membuka Peternak keyring entry: {e}"))
}
