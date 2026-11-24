#!/usr/bin/env bash
# Menjaga backend, Caddy, dan tunnel tetap hidup. Dipanggil start.sh.
set -u
ROOT="${VIDBOOST_ROOT:-$HOME/vidboost}"
while true; do
  bash "$ROOT/start.sh" >/dev/null 2>&1
  sleep 30
done
