#!/usr/bin/env bash
# Jalankan backend Rust + frontend Nuxt sekaligus (development).
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cleanup() {
  kill "${BACKEND_PID:-}" "${FRONTEND_PID:-}" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "==> Menjalankan backend Rust di :8080"
(cd server-rs && cargo run) &
BACKEND_PID=$!

echo "==> Menjalankan frontend Nuxt di :3000 (atau port bebas berikutnya)"
bun run dev &
FRONTEND_PID=$!

wait
