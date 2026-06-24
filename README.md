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

---
*Dibuat untuk kebutuhan Enterprise skala besar.*
