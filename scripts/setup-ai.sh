#!/usr/bin/env bash
# Pasang realesrgan-ncnn-vulkan (portable, termasuk model) untuk VidBoost.
# Aman dijalankan ulang: file lama akan ditimpa.
set -euo pipefail

DEST="${VIDBOOST_AI_DIR:-$HOME/.local/share/vidboost-ai}"
ARCHIVE="$DEST/realesrgan-ncnn-vulkan.zip"
BASE_URL="https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0"
BIN_NAME="realesrgan-ncnn-vulkan"

echo "==> Folder tujuan: $DEST"
mkdir -p "$DEST"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl tidak ditemukan. Install curl dulu ya." >&2
  exit 1
fi
if ! command -v unzip >/dev/null 2>&1; then
  echo "unzip tidak ditemukan. Install unzip dulu ya." >&2
  exit 1
fi

case "$(uname -s)" in
  Linux)
    URL="$BASE_URL/realesrgan-ncnn-vulkan-20220424-ubuntu.zip"
    BIN_NAME="realesrgan-ncnn-vulkan"
    ;;
  Darwin)
    URL="$BASE_URL/realesrgan-ncnn-vulkan-20220424-macos.zip"
    BIN_NAME="realesrgan-ncnn-vulkan"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    URL="$BASE_URL/realesrgan-ncnn-vulkan-20220424-windows.zip"
    BIN_NAME="realesrgan-ncnn-vulkan.exe"
    ;;
  *)
    echo "OS tidak dikenali: $(uname -s). Download manual dari $BASE_URL" >&2
    exit 1
    ;;
esac

echo "==> Mengunduh $URL"
curl -fL --retry 3 --progress-bar "$URL" -o "$ARCHIVE"

echo "==> Extract ke $DEST"
unzip -oq "$ARCHIVE" -d "$DEST"

# Binary bisa berada di root zip atau di subfolder.
BIN="$(find "$DEST" -maxdepth 3 -type f -name "$BIN_NAME" -print -quit || true)"
if [ -z "$BIN" ]; then
  echo "Binary tidak ketemu setelah extract. Cek isi $DEST" >&2
  exit 1
fi
chmod +x "$BIN"

echo
echo "Selesai. Tambahkan ke .env:"
echo
echo "  AI_UPSCALER_BIN=$BIN"
echo
echo "Lalu cek badge 'AI ready' di header VidBoost."
