#!/usr/bin/env python3
"""Buat CNAME vidboost.nievl.site -> mighty-nievl.github.io di Cloudflare."""
import json
import os
import urllib.error
import urllib.request

TOKEN = os.environ.get("CLOUDFLARE_API_TOKEN", "")
if not TOKEN:
    raise SystemExit("CLOUDFLARE_API_TOKEN kosong")

BASE = "https://api.cloudflare.com/client/v4"
HEADERS = {
    "Authorization": f"Bearer {TOKEN}",
    "Content-Type": "application/json",
}


def api(path, method="GET", payload=None):
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(BASE + path, data=data, headers=HEADERS, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as exc:
        body = exc.read().decode()
        raise SystemExit(f"HTTP {exc.code}: {body[:400]}")


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
