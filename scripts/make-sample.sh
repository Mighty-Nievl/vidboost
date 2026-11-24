#!/usr/bin/env bash
# Bikin video uji 8 detik, 640x360, 30fps — buat smoke test VidBoost.
set -euo pipefail

OUT="${1:-sample.mp4}"

if ffmpeg -hide_banner -encoders 2>/dev/null | grep -q ' libx264 '; then
  VIDEO_ARGS=(-c:v libx264 -preset veryfast -crf 23 -pix_fmt yuv420p)
else
  VIDEO_ARGS=(-c:v libopenh264 -b:v 2M -pix_fmt yuv420p)
fi

ffmpeg -y -hide_banner -loglevel error \
  -f lavfi -i "testsrc2=size=640x360:rate=30:duration=8" \
  -f lavfi -i "sine=frequency=440:duration=8" \
  "${VIDEO_ARGS[@]}" \
  -c:a aac -shortest "$OUT"

echo "Sample dibuat: $OUT"
