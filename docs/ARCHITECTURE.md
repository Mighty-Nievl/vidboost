# Arsitektur VidBoost

## Prinsip

- **Satu sumber kebenaran untuk state.** State video, settings, dan status API hidup di satu composable singleton, bukan di halaman.
- **Halaman hanya komposisi.** `app/pages/*.vue` tidak boleh berisi logika bisnis; mereka merakit komponen.
- **Komponen hanya tampilan + input.** Komponen memanggil aksi dari composable, tidak menaruh logika upload/encode.
- **Tema lewat token.** Komponen tidak memilih warna; mereka memakai `bg-ink-*`, `text-zinc-*`, `text-accent` yang dipetakan ke variabel CSS tema.

## Lapisan

```text
app/pages/            routing + komposisi halaman
app/components/       presentational components (props/slot + aksi composable)
app/composables/      state, aksi, dan efek samping (fetch, polling, localStorage)
app/utils/            tipe, konstanta, default, formatter (murni, tanpa state)
app/assets/css/       design tokens + utility clay + animasi
server-rs/            API, job queue, ffmpeg, adapter AI
```

## Frontend

### `app/utils/vidboost.ts`

Tempatnya tipe (`Settings`, `Job`, `AiModel`, `TabId`), `createDefaultSettings()`, opsi chip, kunci localStorage, dan formatter. Tidak bergantung pada Vue.

### `app/composables/useVidBoost.ts`

Singleton state + aksi:

- State: `settings`, `job`, `status`, `models`, `busy`, `error`, `previewMode`, `split`.
- Derived: `hasFile`, `isProcessing`, `isDone`, `chips`, URL preview/download, opsi codec yang valid per format.
- Aksi: `initVidBoost()`, `uploadVideo()`, `uploadLogo()`, `startProcess()`, `resetAll()`, `persistForExport()`.
- Efek: polling job tiap 800 ms, auto-dismiss notifikasi 9 detik, simpan settings ke localStorage.

Halaman memanggil `initVidBoost()` sekali; guard `initialized` mencegah fetch ganda.

### `app/composables/useTheme.ts`

State tema (`clay-light` / `clay-dark`), `toggleTheme()`, `initTheme()`, dan persistensi di localStorage. Tema diterapkan sebagai `data-theme` pada `<html>`.

> Ada juga script inline di `nuxt.config.ts` yang membaca tema sebelum hydration supaya tidak ada flash.

### Komponen

| Komponen | Tanggung jawab |
|---|---|
| `AppHeader` | Logo, badge FFmpeg, toggle tema, tombol Export (mobile), Reset dua klik |
| `VideoPreview` | Player sebelum/sesudah/split, notifikasi, shimmer, progress overlay |
| `BottomNav` | Tab kategori (mobile bottom bar / desktop rail) |
| `SourcePanel` … `ExportPanel` | Kontrol per tab |
| `OptionChips` | Pilihan berbentuk chip, pengganti `<select>` |
| `RangeField`, `ToggleSwitch` | Kontrol dasar dengan state disabled/fokus |

### Halaman

- `/` (`index.vue`): grid preview + panel, nav bawah, transisi tab & sheet.
- `/export` (`export.vue`): preview + panel export penuh. Job & settings dibaca dari localStorage lewat composable yang sama.

## Styling: claymorphism

Token didefinisikan dua kali di `main.css`:

- `:root[data-theme="clay-light"]` → pastel terang, shadow putih lembut.
- `:root[data-theme="clay-dark"]` → abu gelap, shadow hitam (tanpa halo putih).

Utility:

- `.clay-panel` — kartu dengan `border-radius: 20px` + `box-shadow: var(--shadow-out)`.
- `.clay-inset` — efek tertekan untuk track/wadah.
- `.clay-btn` — tombol dengan shadow luar, berubah ke shadow dalam saat ditekan.
- `.clay-chip` — pilihan pill; state aktif memakai shadow dalam.

Semua animasi otomatis dinonaktifkan oleh `@media (prefers-reduced-motion: reduce)`.

## Backend

Rust + Axum. `server-rs/src/`:

- `routes.rs` — handler HTTP, multipart streaming, range streaming, `DefaultBodyLimit` 500 MB.
- `ffmpeg.rs` — probe, filter chain, encode, parser progres, pipeline AI (frame → upscale → encode ulang).
- `jobs.rs` — registry + persistensi atomik `jobs.json`.
- `state.rs` — konfigurasi runtime, resolusi `DATA_DIR`, pencarian font.
- `ai.rs` — daftar model yang benar-benar tersedia + deteksi binary.

## Aturan menjaga kebersihan

1. Jangan menaruh `$fetch` di komponen — taruh di composable.
2. Jangan menambah warna hardcoded di komponen — pakai token tema atau utility clay.
3. Jangan menyimpan state job/settings di `ref` lokal halaman — pakai `useVidBoost()`.
4. Kontrol baru sebaiknya menjadi komponen dengan `v-model`, bukan markup inline panjang.
5. Perubahan filter ffmpeg harus tetap lewat `base_filters()` agar mode normal & AI konsisten.
