#!/usr/bin/env python3
"""Buat CNAME vidboost.nievl.site memakai OAuth token dari `wrangler login`.

Token dibaca dari ~/.config/.wrangler/config/default.toml dan tidak pernah
dicetak ke layar.
"""
import json
import os
import re
import urllib.error
import urllib.request
from pathlib import Path

CONFIG = Path.home() / ".config/.wrangler/config/default.toml"
text = CONFIG.read_text()
match = re.search(r'^oauth_token\s*=\s*"([^"]+)"', text, re.MULTILINE)
if not match:
    raise SystemExit("Token Wrangler tidak ditemukan. Jalankan `wrangler login` dulu.")
token = match.group(1)

BASE = "https://api.cloudflare.com/client/v4"
HEADERS = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}


def api(path, method="GET", payload=None):
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(BASE + path, data=data, headers=HEADERS, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as exc:
        body = exc.read().decode()
        raise SystemExit(f"HTTP {exc.code}: {body[:400]}")


me = api("/user")
print("akun:", (me.get("result") or {}).get("email", "?"))

zones = api("/zones?name=nievl.site")
results = zones.get("result") or []
if not results:
    raise SystemExit("Zone nievl.site tidak ditemukan di akun ini")
zone_id = results[0]["id"]
print("zone:", zone_id)

existing = api(f"/zones/{zone_id}/dns_records?name=vidboost.nievl.site&type=CNAME")
if existing.get("result"):
    print("record sudah ada:", existing["result"][0]["content"])
else:
    created = api(
        f"/zones/{zone_id}/dns_records",
        method="POST",
        payload={
            "type": "CNAME",
            "name": "vidboost.nievl.site",
            "content": "mighty-nievl.github.io",
            "ttl": 1,
            "proxied": False,
        },
    )
    rec = created.get("result") or {}
    print("dibuat:", rec.get("name"), "->", rec.get("content"), "proxied:", rec.get("proxied"))
