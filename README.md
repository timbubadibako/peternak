# Peternak AIAI 🚜

Peternak AIAI adalah sebuah alat Command Line Interface (CLI) interaktif yang ditulis menggunakan Rust. 
Alat ini dibuat khusus untuk mempermudah manajemen "ternak" akun cloud (Google, Supabase, Vercel, GitHub) dalam jumlah besar.

## Fitur Utama:
- 🔐 **OAuth Terpusat:** Manajamen token Google, Supabase, Vercel, dan GitHub menggunakan SQLite internal.
- 🚀 **Farming Supabase:** Buat project Supabase otomatis di seluruh akun menggunakan satu perintah.
- 🚀 **Farming Vercel:** Deploy project Next.js otomatis.
- 🚀 **Farming GitHub:** Auto-create repository private.
- 💉 **Mock Injection:** Memasang kredensial akun ke alat CLI lain (seperti Antigravity) secara on-the-fly.
- 📊 **Cek Kuota:** Cek sisa kuota Google Drive API secara instan.

## Teknologi:
- **Rust** (reqwest, tokio, rusqlite, yup-oauth2)
- **Clap** (untuk Interactive shell REPL)
- **Colored** (untuk UI CLI yang cantik)

## Cara Pakai:
1. Jalankan `cargo run`
2. Ketik `help` di dalam interactive shell untuk melihat semua command.
3. Mulai dengan `add <email>` untuk menambahkan akun.

## Release Installer

Build archive untuk host saat ini:

```bash
bash scripts/package-release.sh
```

Archive akan dibuat di `dist/peternak-aiai-<target>.tar.gz`.

Install via curl untuk Linux/macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/timbubadibako/peternak/main/install.sh | sh
```

Untuk private/self-hosted release, arahkan URL archive:

```bash
PETERNAK_DOWNLOAD_BASE=https://example.com/releases/v0.1.0 \
  curl -fsSL https://example.com/install.sh | sh
```

Install via PowerShell di Windows:

```powershell
irm https://raw.githubusercontent.com/timbubadibako/peternak/main/install.ps1 | iex
```

Install via npm:

```bash
npm install -g peternak-aiai
```

Untuk private/self-hosted binary:

```bash
PETERNAK_DOWNLOAD_BASE=https://example.com/releases/v0.1.0 npm install -g peternak-aiai
```

Catatan runtime:
- Device target harus punya `agy` di PATH.
- Credential default disimpan di OS keyring device target.
- Untuk testing portable, `data.db` dan `agy-*.secret.json` bisa dikirim terpisah, tapi itu membawa credential sensitif.

---
*Dibuat untuk kebutuhan Enterprise skala besar.*
