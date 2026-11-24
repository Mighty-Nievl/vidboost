#!/usr/bin/env bash
ROOT="$HOME/vidboost"
for name in monitor backend caddy tunnel; do
  if [ -f "$ROOT/logs/$name.pid" ]; then
    kill "$(cat "$ROOT/logs/$name.pid")" 2>/dev/null || true
    rm -f "$ROOT/logs/$name.pid"
    echo "$name stopped"
  fi
done
