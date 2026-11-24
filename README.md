# VidBoost

Video enhancer lokal, full-stack. Upload video → atur warna, detail, upscale, crop, watermark → export. Semua proses berjalan di mesin sendiri, tidak ada data yang dikirim ke cloud.

- **Frontend:** Nuxt 4 + Tailwind 4 (Bun)
- **Backend:** Rust + Axum + Tokio, mengorkestrasi ffmpeg
- **AI opsional:** FSRCNN via OpenCV CPU (model 39 KB, tanpa Vulkan/GPU) — lihat [`docs/AI-ENHANCER.md`](docs/AI-ENHANCER.md)

## Fitur

| Grup | Kontrol |
|---|---|
| Warna | Brightness, contrast, saturation, gamma |
| Detail | Sharpen, denoise |
| Upscale | 2×/3×/4× via Lanczos, atau AI lokal |
| Crop & rotate | Rotasi 0/90/180/270, crop persen (X/Y/W/H) |
| Watermark | Teks (font, ukuran, warna, posisi) atau logo gambar (skala, opacity, posisi) |
| Output | MP4 / WebM / MOV / MKV · H.264 / HEVC / VP9 / AV1 · CRF · preset · audio on/off |
| Preview | Sebelum / sesudah, atau split comparison dengan slider |
| Progres | Polling status encode dengan persen dan tahap |

## Jalankan (development)

Butuh **Bun**, **Rust (cargo)**, dan **ffmpeg + ffprobe** di PATH.

```bash
# satu perintah untuk dua-duanya:
bash scripts/dev.sh

# atau manual:
# terminal 1 — backend
(cd server-rs && cargo run)
# terminal 2 — frontend
bun install && bun run dev
```

Buka `http://localhost:3000`. Frontend mem-proxy `/api/**` ke `http://127.0.0.1:8080`. Kalau port 3000 sudah dipakai aplikasi lain, Nuxt otomatis naik ke 3001 (perhatikan URL yang ditampilkan di terminal).

Cek cepat:

```bash
curl http://127.0.0.1:8080/api/status
```

## Konfigurasi

Salin `.env.example` ke `.env`, lalu sesuaikan. Backend membaca env saat startup.

| Env | Default | Keterangan |
|---|---|---|
| `PORT` | `8080` | Port backend Rust |
| `DATA_DIR` | `/run/media/rezal/Data/vidboost` (fallback `.data`) | Tempat upload, output, `jobs.json` |
| `FFMPEG_PATH` | `ffmpeg` | Path binary ffmpeg |
| `FFPROBE_PATH` | `ffprobe` | Path binary ffprobe |
| `DRAWTEXT_FONTFILE` | auto-detect | Font untuk watermark teks |
| `AI_UPSCALER_BIN` | kosong | Path `realesrgan-ncnn-vulkan` |

## Pasang AI upscaler (opsional)

```bash
bash scripts/setup-fsrcnn.sh
```

Lalu isi `AI_UPSCALER_BIN` di `.env` dengan path yang ditampilkan script, restart backend, dan cek badge **AI ready** di header. Detail model, perbandingan RAM, dan alasan pilihannya ada di [`docs/AI-ENHANCER.md`](docs/AI-ENHANCER.md).

## Struktur

```
vidboost/
├─ app/                  # Nuxt 4: UI + komponen
├─ server-rs/            # Axum API: upload, jobs, ffmpeg, AI adapter
├─ docs/AI-ENHANCER.md   # Riset & panduan model AI lokal
├─ scripts/setup-ai.sh   # Installer binary AI portable
└─ .env.example
```

## API

| Method | Path | Fungsi |
|---|---|---|
| POST | `/api/video/upload` | Upload multipart (`file`) → Job |
| POST | `/api/video/logo?id=` | Upload logo watermark → `{ path }` |
| POST | `/api/video/process` | `{ id, settings }` → mulai encode async |
| POST | `/api/video/cancel?id=` | matikan proses ffmpeg/AI yang sedang jalan |
| GET | `/api/video/job?id=` | Status + progres |
| GET | `/api/video/source?id=` | Video asli (HTTP Range) |
| GET | `/api/video/download?id=` | Hasil (attachment, HTTP Range) |
| GET | `/api/video/thumbnail?id=` | Poster frame |
| GET | `/api/status` | Kesehatan ffmpeg & AI |
| GET | `/api/ai/models` | Daftar model AI |

## Tampilan & arsitektur

Full **claymorphism** dengan dua tema: **Clay Light** (default) dan **Clay Dark**. Tombol toggle ada di header; pilihan tersimpan di localStorage dan diterapkan sebelum hydration.

Struktur kode mengikuti [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): halaman hanya komposisi, state/aksi di `useVidBoost()`, tipe & konstanta di `utils/`, warna lewat token tema.

## Motion & feedback

Tombol, chip, dan toggle punya state hover/press yang jelas; panel bawah dan konten tab pakai transisi spring, notifikasi slide-in dengan timer 9 detik, dan progress bar berdenyut. Semua otomatis dinonaktifkan kalau sistem meminta `prefers-reduced-motion`. Tidak ada library animasi tambahan.

## Halaman Export

Di mobile ada tombol **Export** di top bar. Halamannya (`/export`) berisi kontrol output penuh, preview, progress, dan download. Job + setting disimpan di `localStorage`, jadi bisa bolak-balik `/` ↔ `/export` tanpa kehilangan state.

## Mobile

Di lebar <1024px layout berubah jadi mobile-first:

- Preview tampil **paling atas** dan sticky saat scroll, kontrol di bawahnya.
- Hanya panel Source dan Output yang dibuka default; sisanya di-collapse.
- Tombol **Enhance video**, progress, dan Download ada di action bar bawah yang sticky (aman dari safe-area iPhone).
- Tap target diperbesar, slider responsif untuk `pointer: coarse`, input pakai font 16px agar iOS tidak auto-zoom.

Akses dari HP di jaringan yang sama: `http://<IP-LAN>:3000` (atau port yang ditampilkan Nuxt).

## Catatan

- Upload dibatasi 400 MB di UI; backend mengizinkan body sampai 500 MB (`DefaultBodyLimit`). Streaming ke disk, jadi tidak menumpuk di RAM.
- Encode berjalan sebagai background task di proses backend. Kalau backend di-restart di tengah encode, job ditandai error.
- Tombol **Batal** (di preview & panel Export) mengirim sinyal kill ke proses aktif lewat `/api/video/cancel`; job ditandai "Dibatalkan oleh pengguna".
- Mode AI mengekstrak frame ke disk dulu — butuh ruang kosong beberapa kali ukuran video.
- Simulasi A/B split menyinkronkan kedua video; kalau browser menghambat autoplay, tekan play pada video kiri.
- Backend mendeteksi encoder yang tersedia. Di mesin tanpa `libx264`/`libx265`, H.264 otomatis pakai `libopenh264` (bitrate adaptif, bukan CRF) dan HEVC tidak tersedia. Cek dengan `ffmpeg -encoders`.
- `libopenh264` maksimal ~9,2 MP per frame. Faktor upscale otomatis diturunkan kalau hasilnya akan melewati batas itu (mis. video 4K 2x jadi ~1,05x), supaya encode tidak gagal di tengah jalan.
- Kondisi itu ditampilkan di UI sebagai badge **⚠ N encode** di preview dan tombol peringatan di panel Export. Ketuk untuk melihat alasan + pintasan "Ganti ke WebM + VP9" atau "Turunkan faktor". Daftar encoder tersedia di `GET /api/status` (`encoders`).
