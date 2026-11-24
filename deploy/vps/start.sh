#!/usr/bin/env bash
set -euo pipefail
ROOT="$HOME/vidboost"
cd "$ROOT"
mkdir -p logs data
source env.sh

if ! pgrep -f 'vidboost-server' >/dev/null 2>&1; then
  nohup ./server-rs/target/release/vidboost-server > logs/backend.log 2>&1 &
  echo $! > logs/backend.pid
  echo "backend started"
else
  echo "backend already running"
fi

if ! pgrep -f 'caddy run' >/dev/null 2>&1; then
  nohup ./bin/caddy run --config Caddyfile > logs/caddy.log 2>&1 &
  echo $! > logs/caddy.pid
  echo "caddy started"
else
  echo "caddy already running"
fi

if ! pgrep -f 'cloudflared tunnel --url http://127.0.0.1:18080' >/dev/null 2>&1; then
  nohup ./bin/cloudflared tunnel --url http://127.0.0.1:18080 --no-autoupdate > logs/tunnel.log 2>&1 &
  echo $! > logs/tunnel.pid
  echo "tunnel started"
else
  echo "tunnel already running"
fi

find "$DATA_DIR/uploads" -type d -name '_ai' -mtime +1 -exec rm -rf {} + 2>/dev/null || true

if [ ! -f logs/monitor.pid ] || ! kill -0 "$(cat logs/monitor.pid)" 2>/dev/null; then
  nohup bash monitor.sh > logs/monitor.log 2>&1 &
  echo $! > logs/monitor.pid
  echo "monitor started"
fi
