#!/usr/bin/env bash
# Smoke test end-to-end terhadap backend yang sudah jalan di :8080.
set -euo pipefail

API="${API:-http://127.0.0.1:8080}"
VIDEO="${1:-sample.mp4}"

echo "==> status"
curl -fsS "$API/api/status"; echo

echo "==> upload $VIDEO"
JOB="$(curl -fsS -F "file=@$VIDEO" "$API/api/video/upload")"
echo "$JOB"
ID="$(printf '%s' "$JOB" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
[ -n "$ID" ] || { echo "Gagal ambil job id" >&2; exit 1; }

echo "==> process"
curl -fsS -H 'Content-Type: application/json' \
  -d "{\"id\":\"$ID\",\"settings\":{\"color\":{\"enabled\":true,\"brightness\":0.03,\"contrast\":1.1,\"saturation\":1.15,\"gamma\":1},\"detail\":{\"enabled\":true,\"sharpen\":30,\"denoise\":5},\"upscale\":{\"enabled\":true,\"factor\":2,\"ai\":false},\"geometry\":{\"rotate\":0},\"watermark\":{\"mode\":\"text\",\"text\":\"VidBoost\",\"fontSize\":36,\"color\":\"#ffffff\",\"position\":\"bottom-right\"},\"output\":{\"format\":\"mp4\",\"codec\":\"h264\",\"crf\":23,\"preset\":\"fast\",\"audio\":true}}}" \
  "$API/api/video/process"; echo

echo "==> polling"
for i in $(seq 1 120); do
  RES="$(curl -fsS "$API/api/video/job?id=$ID")"
  STATUS="$(printf '%s' "$RES" | sed -n 's/.*"status"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
  PROG="$(printf '%s' "$RES" | sed -n 's/.*"progress"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)"
  printf '\r  %s %s%%' "$STATUS" "${PROG:-0}"
  if [ "$STATUS" = "done" ] || [ "$STATUS" = "error" ]; then echo; break; fi
  sleep 1
done

echo "==> job terakhir"
echo "$RES"

if [ "${STATUS:-}" = "done" ]; then
  echo "==> unduh 1 KB pertama"
  curl -fsS -r 0-1023 -D - "$API/api/video/download?id=$ID" -o /tmp/vidboost-smoke.bin | head -20
  ls -l /tmp/vidboost-smoke.bin
  echo "Selesai ✓"
else
  echo "Encode gagal ✗" >&2
  exit 1
fi
