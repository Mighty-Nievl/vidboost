#!/usr/bin/env bash
# Deploy backend ke Fly.io + frontend ke Cloudflare Pages.
# Butuh: fly auth login & wrangler login sudah dilakukan sekali.
set -euo pipefail

export FLYCTL_INSTALL="${FLYCTL_INSTALL:-$HOME/.fly}"
export PATH="$FLYCTL_INSTALL/bin:$HOME/.bun/bin:$PATH"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP="${FLY_APP:-vidboost-api}"
PAGES="${CF_PAGES_PROJECT:-vidboost}"
REGION="${FLY_REGION:-sin}"

echo "==> Cek login"
fly auth whoami
wrangler whoami

echo "==> Pastikan app & volume Fly ada"
fly apps create "$APP" 2>/dev/null || true
fly volumes create vidboost_data --region "$REGION" --size 5 2>/dev/null || true

echo "==> Deploy backend ke Fly.io"
fly deploy

BACKEND_URL="https://$APP.fly.dev"
echo "==> Backend: $BACKEND_URL"

echo "==> Build frontend Pages dengan API base $BACKEND_URL"
NUXT_PUBLIC_API_BASE="$BACKEND_URL" bun run generate

echo "==> Deploy frontend ke Cloudflare Pages ($PAGES)"
wrangler pages deploy .output/public --project-name "$PAGES"

echo
echo "Selesai. Cek:"
echo "  curl $BACKEND_URL/api/status"
echo "  Buka URL Pages di HP."
