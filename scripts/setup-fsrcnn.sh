#!/usr/bin/env bash
# Pasang FSRCNN (OpenCV CPU) — alternatif AI tanpa Vulkan/GPU.
set -euo pipefail

MODEL_DIR="${FSRCNN_MODEL_DIR:-/run/media/rezal/Data/vidboost-ai/fsrcnn}"
BASE="https://raw.githubusercontent.com/Saafke/FSRCNN_Tensorflow/master/models"

echo "==> Menyiapkan pip"
python3 -m ensurepip --user >/dev/null 2>&1 || true

echo "==> Memasang OpenCV contrib 4.10 (kompatibel dengan model .pb FSRCNN)"
python3 -m pip install --user 'opencv-contrib-python-headless==4.10.0.84' 'numpy>=2'

echo "==> Mengunduh model FSRCNN (~40 KB tiap)"
mkdir -p "$MODEL_DIR"
for scale in 2 3 4; do
  curl -fsSL --retry 3 "$BASE/FSRCNN_x${scale}.pb" -o "$MODEL_DIR/FSRCNN_x${scale}.pb"
done
ls -lh "$MODEL_DIR"

echo
echo "Selesai. Tambahkan ke .env:"
echo
echo "  AI_UPSCALER_BIN=$PWD/scripts/fsrcnn-upscaler.sh"
echo "  # AI_UPSCALER_GPU tidak dipakai (CPU murni)"
echo
