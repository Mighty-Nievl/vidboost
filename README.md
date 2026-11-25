<div align="center">

# VidBoost

**Video enhancer lokal yang berjalan di device kamu.**

[![Build Tauri](https://github.com/Mighty-Nievl/vidboost/actions/workflows/build.yml/badge.svg)](https://github.com/Mighty-Nievl/vidboost/actions/workflows/build.yml)
[![Build Android](https://github.com/Mighty-Nievl/vidboost/actions/workflows/android.yml/badge.svg)](https://github.com/Mighty-Nievl/vidboost/actions/workflows/android.yml)
[![Deploy Landing](https://github.com/Mighty-Nievl/vidboost/actions/workflows/pages.yml/badge.svg)](https://github.com/Mighty-Nievl/vidboost/actions/workflows/pages.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-7fb524.svg)](LICENSE)

[Web · Landing](https://vidboost.nievl.site/) · [Download](https://github.com/Mighty-Nievl/vidboost/releases) · [Dokumentasi](docs/)

</div>

---

## Kenapa VidBoost?

Sebagian tools video enhancement mengupload file kamu ke server. VidBoost melakukan sebaliknya: **semua proses berjalan lokal**, mulai dari koreksi warna sampai AI upscaling. Model AI tidak dibundel — kamu unduh sendiri dan pakai offline.

- 🧱 **Backend Rust** + **ffmpeg** untuk encode yang cepat dan aman
- 🖥️ **Tauri desktop** (Linux · Windows · macOS) dan **Android** (API 29+, semua ABI)
- 🎨 **Claymorphism** dengan dua tema: light dan dark, satu aksen lime
- 🧠 **AI lokal**: FSRCNN, ESPCN, LapSRN, EDSR, Real-ESRGAN
- 📊 **Progress hidup**: progress ring, ETA, log proses, dan tombol batal nyata

> ⚠️ Super-resolution **memperkirakan** detail, bukan mengembalikan data yang hilang. Untuk video yang benar-benar blur, kombinasi denoise + sharpen + kontras biasanya lebih terasa daripada sekadar memperbesar.

## Tampilan

| Area | Yang bisa kamu atur |
|---|---|
| **Source** | Drag & drop video, metadata (resolusi, durasi, fps, codec), ganti file |
| **Warna** | Brightness, contrast, saturation, gamma, reset ke netral |
| **Detail** | Sharpen, denoise dengan hint efek samping |
| **Upscale** | Faktor 1–4x, mode Lanczos atau AI, model AI, faktor efektif |
| **Crop & rotate** | Rotasi 0/90/180/270, crop X/Y/W/H dalam persen |
| **Teks** | Watermark teks atau logo, ukuran, warna, posisi, opacity |
| **Export** | Format MP4/WebM/MOV/MKV, codec H.264/HEVC/VP9/AV1, CRF, preset, audio |
| **Proses** | Full-screen overlay dengan progress ring, ETA, log, Batal, Lihat hasil |

Navigasi menyesuaikan device:

- **Mobile**: bottom tab + sheet yang bisa diciutkan, preview tetap terlihat
- **Desktop**: rail kategori di samping panel kontrol

## Model AI

| Model | Ukuran | Runtime | RAM | Kecepatan | Kualitas |
|---|---|---|---|---|---|
| **FSRCNN** (default) | 39 KB | OpenCV CPU | ~200 MB | Sedang | Sedang |
| ESPCN | 100 KB | OpenCV CPU | ~150 MB | Paling cepat | Rendah |
| LapSRN | 4 MB | OpenCV CPU | ~400 MB | Sedang | Cukup baik |
| EDSR | 38 MB | OpenCV CPU | ~1 GB | Lambat | Tinggi |
| Real-ESRGAN Anime v3 | 1,2 MB | Vulkan | ~300 MB | Cepat di GPU | Baik (anime) |
| Real-ESRGAN x4plus | 32 MB | Vulkan | ~600 MB | Lambat | Tertinggi |

Detail kelebihan/kekurangan ada di halaman **Model AI** di dalam aplikasi dan di [`docs/MODELS.md`](docs/MODELS.md).

## Quickstart

### Pakai aplikasi

1. Download installer dari [Releases](https://github.com/Mighty-Nievl/vidboost/releases).
2. Buka VidBoost → pilih video.
3. Atur parameter → **Export** → **Enhance video**.
4. Tunggu proses selesai → **Lihat hasil** → simpan video.

### Jalankan dari source (web/dev)

```bash
bun install
bun run dev          # frontend di :3000/3001
```

> Aturan pengembangan: **build lokal hanya `bun`**. Semua build Rust, Tauri, dan Android dijalankan di GitHub Actions.

## Arsitektur

```text
app/                    Nuxt 4 + Tailwind 4 (UI, komponen, composable)
crates/vidboost-core/   Logika Rust murni: tipe, filter, codec, registry job
server-rs/              Axum HTTP API (wrapper tipis di atas core)
src-tauri/              Tauri desktop + Android + Tauri commands
landing/                Landing page GitHub Pages
.github/workflows/      build.yml · android.yml · pages.yml
docs/                   Arsitektur, deploy, model, VPS, panduan AI
```

Baca detailnya di [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Build & rilis

| Workflow | Output |
|---|---|
| `build.yml` | Installer Linux, Windows, macOS; draft Release saat tag `v*` |
| `android.yml` | APK untuk arm64, armv7, i686, x86_64; rilis saat tag `v*` |
| `pages.yml` | Landing page ke GitHub Pages |

Untuk membuat rilis:

```bash
git tag v0.1.0
git push origin v0.1.0
```

> GitHub Pages harus diaktifkan sekali di **Settings → Pages → Source: GitHub Actions**.

## Pengaman resource

- Batas thread ffmpeg & FSRCNN mengikuti jumlah vCPU
- Batas output pixel (default 5 MP) agar video tetap bisa diputar di HP
- Watchdog encode 15 menit, timeout ekstrak frame & upscale 5 menit
- Upload dibatasi 500 MB di backend; streaming ke disk, bukan RAM
- Tombol **Batal** mengirim sinyal kill ke proses aktif

## Dokumentasi

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — prinsip & peta lapisan kode
- [`docs/MODELS.md`](docs/MODELS.md) — semua model AI, plus/minus, cara pasang
- [`docs/AI-ENHANCER.md`](docs/AI-ENHANCER.md) — riset & alasan pilihan model
- [`docs/VPS.md`](docs/VPS.md) — deploy VPS shared tanpa sudo
- [`docs/DEPLOY.md`](docs/DEPLOY.md) — opsi deploy server

## License

MIT © Mighty-Nievl
