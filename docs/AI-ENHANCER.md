# AI Enhancer — pilihan model untuk RAM 1 GB

Catatan riset dan panduan integrasi AI lokal untuk VidBoost. Ditulis 2026-11-24.

> ✅ **Status mesin ini:** AI aktif lewat **FSRCNN (CPU murni, tanpa Vulkan)**.
> GPU 0 (Radeon Vega 8) beberapa kali menyebabkan `amdgpu device wedged` + `ring reset`
> saat Vulkan compute, dan GPU 1 (llvmpipe/CPU) selalu timeout. Karena itu Real-ESRGAN
> dinonaktifkan dan diganti FSRCNN via OpenCV `dnn_superres` (model 39 KB).
> Pasang dengan `bash scripts/setup-fsrcnn.sh`.

## Pendekatan yang dipakai: FSRCNN

| Aspek | Nilai |
|---|---|
| Model | `FSRCNN_x2/x3/x4.pb`, 39–41 KB |
| Runtime | OpenCV 4.10 `dnn_superres`, Python 3, CPU (`cv2.setNumThreads(4)`) |
| Vulkan/GPU | Tidak dipakai sama sekali |
| RAM | ~200 MB |
| Kecepatan | ~330 ms per frame 640×360; video 8 detik selesai ~90 detik |
| Kualitas | Lebih modest dari Real-ESRGAN, tapi stabil & tidak mengunci GPU |

Alur di VidBoost: frame diekstrak → **di-downscale ke lebar 640** (biar beban tetap terikat) → FSRCNN upscale 2x/3x/4x → encode ulang dengan audio asli.

## Kesimpulan singkat

Untuk mesin dengan **RAM 1 GB**, jangan pakai Real-ESRGAN versi PyTorch. Pakai **build `ncnn` / `ncnn-vulkan`** yang portable: tidak butuh CUDA, tidak butuh PyTorch, modelnya beberapa MB, dan bisa diatur memory footprint-nya lewat tiling.

**Rekomendasi praktis di VidBoost: `realesrgan-ncnn-vulkan` dengan model yang ikut terbundle di binary portable.**

> ⚠️ Koreksi penting: binary portable `20220424` **tidak menyertakan** model ncnn `realesr-general-x4v3` (yang tersedia hanya file `.pth` untuk PyTorch). Jadi jangan pakai nama model itu dengan binary ini — proses akan gagal. Model yang benar-benar ada di folder `models/`: `realesrgan-x4plus` (32 MB), `realesrgan-x4plus-anime` (8,6 MB), dan `realesr-animevideov3-x2/x3/x4` (1,2 MB).

| Kandidat | Ukuran model | RAM puncak | Cocok untuk | Verdict |
|---|---|---|---|---|
| `realesrgan-x4plus` (ncnn, bundled) | 32 MB | ~600–1000 MB | Foto & video nyata, kualitas terbaik | ✅ **Pilihan utama** (2x + tile 128) |
| `realesr-animevideov3` (ncnn, bundled) | 1,2 MB | ~300–500 MB | Anime/kartun, paling cepat & ringan | ✅ Pilihan ringan |
| `realesrgan-x4plus-anime` (ncnn, bundled) | 8,6 MB | ~500–800 MB | Ilustrasi anime | ✅ Oke |
| `realesr-general-x4v3` | hanya `.pth` | — | — | ❌ Tidak tersedia di binary portable |
| FSRCNN (`dnn_superres`, OpenCV) | ~40–80 KB | <200 MB | Fallback ekstrem, real-time, tanpa Vulkan | ⚠️ Perlu integrasi terpisah |
| `RealESRGAN_x4plus` (PyTorch) | ~64 MB weights | 2–4 GB | Kualitas maksimal | ❌ Tidak masuk 1 GB |
| RIFE (interpolasi frame) | puluhan MB | 2 GB+ | Slow-motion / FPS naik | ❌ Tidak masuk 1 GB |
| GFPGAN / CodeFormer | ~100–350 MB | 1,5–3 GB | Restorasi wajah | ❌ Tidak masuk 1 GB |
| Upscaler berbasis Stable Diffusion | multi-GB | 4 GB+ | Generatif | ❌ Mustahil di 1 GB |

Sumber: [Real-ESRGAN README](https://github.com/xinntao/Real-ESRGAN) · [Real-ESRGAN-ncnn-vulkan](https://github.com/xinntao/Real-ESRGAN-ncnn-vulkan) · [waifu2x-ncnn-vulkan](https://github.com/nihui/waifu2x-ncnn-vulkan).

## Kenapa FSRCNN

- Modelnya 39–41 KB, jauh lebih kecil dari Real-ESRGAN (1,2–32 MB).
- `dnn_superres` berjalan di CPU tanpa Vulkan, jadi tidak menyentuh driver GPU yang bermasalah.
- Frame terlebih dahulu dikecilkan ke lebar 640 supaya beban CPU tetap terikat.
- Denoise tetap memakai filter `hqdn3d` di VidBoost.

## Setting aman

```bash
bash scripts/fsrcnn-upscaler.sh \
  -i frames \
  -o upscaled \
  -n fsrcnn \
  -s 2
```

Artinya:

- `-s 2` — 2× paling seimbang antara waktu dan hasil.
- `cv2.setNumThreads(4)` — empat thread CPU (tercepat di mesin ini).
- Timeout upscale 300 detik, ekstrak frame 300 detik.
- Batas AI: 720p, 30 fps, 10 detik.

## Alur di VidBoost

Mode AI bukan filter real-time, tapi pipeline frame:

```text
video asli
  → ffmpeg ekstrak frame (-vsync 0) ke PNG
  → realesrgan-ncnn-vulkan upscale per frame
  → ffmpeg encode ulang frame + audio asli
  → filter warna / detail / watermark / crop diterapkan saat encode ulang
  → hasil akhir
```

Konsekuensi: makin panjang video, makin banyak disk dan waktu yang dipakai. Untuk video pendek (30–60 detik) ini nyaman; untuk video panjang sebaiknya pakai mode Lanczos biasa atau potong dulu.

## Cara pasang (Linux)

```bash
bash scripts/setup-ai.sh
```

Lalu di `.env`:

```env
AI_UPSCALER_BIN=$HOME/.local/share/vidboost-ai/realesrgan-ncnn-vulkan
```

Cek di UI: badge **AI ready** di header. Kalau belum aktif, UI otomatis memakai Lanczos dan menampilkan pesan.

## Catatan penting

- `ncnn-vulkan` butuh runtime Vulkan. Di mesin tanpa GPU, install `libvulkan1` + `vulkan-driver` (mis. `mesa-vulkan-drivers`) atau pakai build ncnn CPU / FSRCNN.
- Model anime (`realesr-animevideov3`, `realesrgan-x4plus-anime`) akan terlihat bagus di animasi tapi bisa menghasilkan artefak "lukisan" di foto realistis.
- AI upscale **tidak** menciptakan detail yang benar-benar hilang; ia mengestimasi. Jangan dipakai sebagai bukti/forensik.
- Kalau hasil terlihat terlalu halus, turunkan denoise atau matikan filter `hqdn3d` di panel Detail.
