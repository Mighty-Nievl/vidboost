# Deploy ke VPS shared (tanpa sudo)

Konfigurasi aktif: `nievl@85.9.194.109`, Rocky Linux 8.10, 2 vCPU, 7,7 GB RAM.
Karena bukan sudoers, semua dependency dipasang di `$HOME`.

## Arsitektur

```text
Internet → Cloudflare Tunnel → Caddy :18080 → static site (/site)
                                      └─ /api/* → Rust backend :19000
                                                   ├─ ffmpeg statis
                                                   └─ FSRCNN (Python/OpenCV CPU)
```

Port 18080 tidak dibuka di firewall provider; akses lewat URL `trycloudflare.com`.

## Yang terpasang di VPS

| Komponen | Lokasi |
|---|---|
| Source + skrip | `~/vidboost/` |
| Frontend statis | `~/vidboost/site/` |
| Backend release | `~/vidboost/server-rs/target/release/vidboost-server` |
| ffmpeg / ffprobe | `~/vidboost/bin/` (static build 7.0.2) |
| Caddy | `~/vidboost/bin/caddy` |
| cloudflared | `~/vidboost/bin/cloudflared` |
| Rust | `~/.cargo/bin` (rustup) |
| zig cc + lib | `~/vidboost/bin/` (linker, tanpa gcc sistem) |
| Model FSRCNN | `~/vidboost/models/FSRCNN_x{2,3,4}.pb` |
| Font drawtext | `~/vidboost/fonts/DejaVuSans.ttf` |
| Data & log | `~/vidboost/data/`, `~/vidboost/logs/` |

## Pengaman CPU / RAM

| Pengaman | Nilai | Fungsi |
|---|---|---|
| `FFMPEG_THREADS=2` | 2 thread | Encode tidak menyedot semua vCPU |
| `-filter_threads 2` | 2 thread | Filter chain dibatasi |
| `MAX_OUTPUT_PIXELS=20.000.000` | 20 MP | Faktor upscale diturunkan otomatis kalau hasil melewati batas |
| Watchdog encode | 900 detik | ffmpeg dihentikan kalau encode nggak selesai 15 menit |
| Ekstrak frame AI | 300 detik | Timeout ekstrak & resize frame |
| Upscale AI | 300 detik | Timeout FSRCNN |
| `FSRCNN_THREADS=2` | 2 thread | OpenCV tidak oversubscribe vCPU |
| Guard AI | 720p / 30fps / 10 detik | Menolak input yang terlalu berat sebelum diproses |
| Body limit | 500 MB | Upload tidak menumpuk di RAM |
| `monitor.sh` | tiap 30 detik | Menyalakan ulang service yang mati |

Filter upscale dihitung: `factor aman = sqrt(maks_pixel / pixel_sumber)`, jadi video 4K 2x otomatis diturunkan.

## Start / stop

```bash
bash ~/vidboost/start.sh   # backend + caddy + tunnel
bash ~/vidboost/stop.sh
```

`start.sh` me-load `env.sh` dan tidak menyalakan proses yang sudah jalan.

## Update deploy

Dari mesin lokal:

```bash
bun run generate
rsync -az --exclude node_modules --exclude .nuxt --exclude target \
  -e 'ssh -i ~/.ssh/vidboost_vps' \
  ./ nievl@85.9.194.109:~/vidboost-src/
ssh -i ~/.ssh/vidboost_vps nievl@85.9.194.109 \
  'cp -r ~/vidboost-src/.output/public/* ~/vidboost/site/ && \
   cd ~/vidboost/server-rs && export PATH=$HOME/vidboost/bin:$HOME/.cargo/bin:$PATH && \
   cargo build --release && bash ~/vidboost/stop.sh && bash ~/vidboost/start.sh'
```

## Catatan

- ffmpeg statis punya `libx264`, jadi tidak ada batas 9,2 MP seperti `libopenh264` di laptop.
- AI memakai FSRCNN CPU; hindari Real-ESRGAN/Vulkan di VPS ini.
- URL tunnel berubah setiap kali cloudflared di-restart. Cek `~/vidboost/logs/tunnel.log`.
- Untuk domain permanen, daftarkan named tunnel Cloudflare di akun user.
