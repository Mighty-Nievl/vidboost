# Model AI di VidBoost Desktop

VidBoost versi Tauri berjalan **di device user**. Model tidak dibundel — user mengunduhnya sendiri lewat menu **Model AI**, lalu model disimpan di `app_data_dir/models`.

## Prasyarat di device

- **Python 3.9+**
- `python3 -m pip install --user opencv-contrib-python-headless==4.10.0.84 numpy`
- Untuk model Vulkan: binary `realesrgan-ncnn-vulkan` + driver Vulkan yang stabil

Backend Rust menyalin skrip upscaler ke `app_data_dir/runtime/` saat aplikasi pertama kali jalan, lalu memanggilnya lewat wrapper `upscaler.sh`.

## Daftar model

| Model | Ukuran | Runtime | RAM | Kecepatan | Kualitas | + | − |
|---|---|---|---|---|---|---|---|
| FSRCNN | 39 KB | OpenCV CPU | ~200 MB | sedang | sedang | Paling ringan & aman | Detail modest |
| ESPCN | 100 KB | OpenCV CPU | ~150 MB | paling cepat | rendah | Cepat | Garis licin |
| LapSRN | 4 MB | OpenCV CPU | ~400 MB | sedang | cukup baik | Lebih tajam, bisa 8x | Boram RAM |
| EDSR | 38 MB | OpenCV CPU | ~1 GB | lambat | tinggi | Paling tajam di CPU | Berat |
| Real-ESRGAN Anime v3 | 1,2 MB | Vulkan | ~300 MB | cepat di GPU | baik (anime) | Ringan & cepat | Butuh GPU stabil |
| Real-ESRGAN x4plus | 32 MB | Vulkan | ~600 MB | lambat | tertinggi | Kualitas terbaik | Butuh GPU kuat |

## Catatan penting

Model SR **tidak mengembalikan detail yang hilang**. Ia hanya memperkirakan tepi dari pixel yang ada. Untuk video yang benar-benar buram (fokus/motion blur), hasil paling terasa justru dari kombinasi **denoise + sharpen (CAS/unsharp) + kontras ringan**.

Model Vulkan belum bisa otomatis terunduh lewat menu; unduh manual binary dari rilis Real-ESRGAN, lalu set `REALESRGAN_BIN`. Model ini tidak disarankan di Radeon Vega 8 karena pernah menyebabkan `device wedged`.
