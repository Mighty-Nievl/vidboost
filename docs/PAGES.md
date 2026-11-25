# Custom domain — vidboost.nievl.site

Landing page di-deploy dari folder `landing/` lewat `.github/workflows/pages.yml`.

## DNS Cloudflare

Akun: `rezalhbramantara@gmail.com`, zone: `nievl.site`.

Buat record:

| Type | Name | Value | Proxy |
|---|---|---|---|
| CNAME | `vidboost` | `mighty-nievl.github.io` | **DNS only** (abu-abu, bukan oranye) |

> GitHub Pages tidak mendukung Cloudflare proxy (orange cloud) untuk domain kustom — harus DNS only.

> Gotcha: `wrangler login` (device login) **tidak punya scope DNS edit** — daftar OAuth-nya hanya `zone:read`, `pages:write`, dll. Untuk membuat record DNS lewat API wajib pakai **API Token** dengan template *Edit zone DNS*. Jangan buang waktu dengan device login untuk urusan DNS.

## GitHub

File `landing/CNAME` berisi `vidboost.nievl.site` dan ikut ter-deploy bersama artifact Pages.

Di **Settings → Pages → Custom domain**, isi:

```
vidboost.nievl.site
```

Centang **Enforce HTTPS** setelah DNS terverifikasi.

## Verifikasi

```bash
dig +short vidboost.nievl.site
# harus mengarah ke IP GitHub Pages

curl -I https://vidboost.nievl.site/
# HTTP 200
```

Jika masih 404, tunggu propagasi DNS (biasanya 1–10 menit), lalu jalankan ulang workflow **Deploy Landing Page**.
