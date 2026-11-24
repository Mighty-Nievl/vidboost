# Deploy: Cloudflare Pages + Fly.io

## Arsitektur

```text
HP / browser
  └── Cloudflare Pages  → UI statis Nuxt (hasil `nuxt generate`)
        └── /api/*  →  https://vidboost-api.fly.dev  →  Rust + ffmpeg
```

Pages **tidak** menjalankan ffmpeg. Ia hanya menyajikan HTML/JS/CSS. Semua proses video terjadi di mesin Fly.io.

## Prasyarat

- Akun Fly.io + `fly` CLI (`fly auth login`)
- Akun Cloudflare + `wrangler` (`npm i -g wrangler`, lalu `wrangler login`)
- Bun

## 1. Deploy backend ke Fly.io

Dari root folder `vidboost/`:

```bash
fly auth login
fly apps create vidboost-api          # kalau belum ada
fly volumes create vidboost_data --region sin --size 5
fly deploy
```

Cek:

```bash
fly logs
fly open /api/status
```

URL backend biasanya `https://vidboost-api.fly.dev`. Kalau nama app berbeda, sesuaikan.

Catatan:

- `fly.toml` memakai region `sin` (Singapore), 1 shared CPU, 1024 MB RAM, volume 5 GB.
- `auto_stop_machines = false` supaya mesin nggak mati saat idle (encode butuh waktu).
- Volume dipakai untuk upload/output/`jobs.json`. Kalau penuh, ukurannya bisa ditambah: `fly volumes extend <volume-id> --size 10`.
- shared-cpu + volume masuk kategori berbayar (murah, tapi bukan free tier murni). Cek harga di dashboard Fly.

## 2. Build & deploy frontend ke Cloudflare Pages

```bash
cp .env.production.example .env.production
# edit NUXT_PUBLIC_API_BASE sesuai URL backend Fly kamu

bun install
NUXT_PUBLIC_API_BASE=https://vidboost-api.fly.dev bun run generate
npx wrangler pages deploy .output/public --project-name vidboost
```

`NUXT_PUBLIC_API_BASE` akan ditanam ke bundle saat build. Kalau kosong, frontend memakai `/api/...` relatif (mode lokal).

Setelah deploy, buka URL Pages di HP. Pastikan backend Fly sudah aktif, karena Pages akan memanggilnya langsung.

## 3. Verifikasi

```bash
# backend
curl https://vidboost-api.fly.dev/api/status

# frontend (ganti dengan URL Pages)
curl https://vidboost.pages.dev
```

Lalu dari HP: upload video → Enhance → preview → download.

## Keamanan (opsional tapi disarankan)

Saat ini backend `CORS: allow-all` supaya mode lokal jalan. Untuk produksi:

1. Di `server-rs/src/main.rs`, ganti `CorsLayer::permissive()` dengan origin Pages kamu:
   ```rust
   CorsLayer::new()
       .allow_origin("https://vidboost.pages.dev".parse::<HeaderValue>().unwrap())
       .allow_methods([Method::GET, Method::POST])
       .allow_headers([CONTENT_TYPE]);
   ```
2. Tambahkan **Cloudflare Access** di depan domain Pages kalau tidak ingin aplikasi bisa dibuka orang lain.
3. Atau pasang auth sederhana di backend (token) sebelum encode.

Tanpa ini, siapa pun yang tahu URL bisa memakai backend kamu.

## Troubleshooting

| Gejala | Penyebab | Solusi |
|---|---|---|
| `Mixed content blocked` | Pages HTTPS memanggil backend HTTP | Pastikan backend lewat Fly HTTPS (`force_https = true`) |
| `CORS error` | Origin belum diizinkan | Sesuaikan `CorsLayer` seperti di atas |
| `Blocked request. This host is not allowed` | Vite menolak host tunnel | Sudah diatasi: `vite.server.allowedHosts = ['.trycloudflare.com']` di `nuxt.config.ts` |
| Upload gagal di HP | Video terlalu besar / koneksi putus | UI membatasi 400 MB; pakai WiFi stabil |
| Job error setelah restart | Encode terputus saat mesin restart | Set `auto_stop_machines = false` |
| Disk penuh | Frame AI menumpuk di `/data` | Besarkan volume atau kurangi durasi video |
| AI tidak aktif | `AI_UPSCALER_BIN` belum ada di container | Mode Lanczos tetap jalan; AI menyusul |

## Alternatif lebih cepat (tanpa deploy)

Kalau cuma mau akses dari HP sekarang, quick tunnel sudah cukup:

```bash
cloudflared tunnel --url http://localhost:3001 --no-autoupdate
```

URL `*.trycloudflare.com` berlaku selama proses jalan dan laptop menyala.
