<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useVidBoost } from '~/composables/useVidBoost'
import type { AiModel } from '~/utils/vidboost'

interface CatalogFile { filename: string; url: string }
interface CatalogModel extends Omit<AiModel, 'vram'> {
  sizeMb: number
  files: CatalogFile[]
}

const { settings } = useVidBoost()
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const catalog = ref<CatalogModel[]>([])
const installed = ref<string[]>([])
const folder = ref('')
const busyFile = ref('')
const error = ref('')
const loading = ref(false)

const activeModel = computed(() => settings.upscale.aiModel)

async function invokeCmd<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

async function refresh() {
  if (!isTauri) return
  loading.value = true
  try {
    const [list, files, path] = await Promise.all([
      invokeCmd<CatalogModel[]>('model_catalog'),
      invokeCmd<string[]>('installed_models'),
      invokeCmd<string>('model_folder'),
    ])
    catalog.value = list
    installed.value = files
    folder.value = path
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : 'Gagal memuat katalog model.'
  } finally {
    loading.value = false
  }
}

function hasFile(filename: string) {
  return installed.value.includes(filename)
}

async function download(file: CatalogFile) {
  busyFile.value = file.filename
  error.value = ''
  try {
    await invokeCmd<string>('download_model', { filename: file.filename, url: file.url })
    await refresh()
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : 'Gagal mengunduh model.'
  } finally {
    busyFile.value = ''
  }
}

async function remove(filename: string) {
  try {
    await invokeCmd<void>('remove_model', { filename })
    await refresh()
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : 'Gagal menghapus model.'
  }
}

function useModel(model: CatalogModel) {
  settings.upscale.aiModel = model.id
  settings.upscale.enabled = true
  settings.upscale.ai = true
  void navigateTo('/')
}

onMounted(refresh)
</script>

<template>
  <div class="min-h-screen bg-ink-950 pb-10 text-zinc-100">
    <header class="sticky top-0 z-30 border-b border-line bg-ink-950">
      <div class="mx-auto flex max-w-4xl items-center gap-3 px-4 py-3 sm:px-6">
        <button type="button" class="clay-btn min-h-[40px] rounded-xl border border-line bg-ink-900 px-3 text-[12px]" @click="navigateTo('/')">
          ← Kembali
        </button>
        <h1 class="text-[14px] font-semibold">Model AI</h1>
        <span class="ml-auto font-mono text-[10px] text-dim">{{ isTauri ? 'Desktop · lokal' : 'Web · server' }}</span>
      </div>
    </header>

    <main class="mx-auto max-w-4xl space-y-4 px-4 pt-5 sm:px-6">
      <div class="clay-panel !rounded-2xl px-4 py-3">
        <p class="text-[12px] leading-snug text-zinc-200">
          Di aplikasi desktop, model diunduh ke device sendiri. Pilih model, lalu lihat kelebihan & kekurangannya sebelum dipakai.
        </p>
        <p v-if="folder" class="mt-2 break-all font-mono text-[10px] text-dim">Folder model: {{ folder }}</p>
        <p v-if="!isTauri" class="mt-2 text-[11px] text-dim">
          Menu unduh hanya aktif di build Tauri. Di web, model disiapkan oleh server.
        </p>
      </div>

      <div v-if="error" role="alert" class="rounded-xl border border-red-500/30 bg-red-500/5 px-3 py-2.5 text-[12px] text-red-300">
        {{ error }}
      </div>

      <p v-if="loading" class="text-[12px] text-dim">Memuat katalog…</p>

      <section
        v-for="model in catalog"
        :key="model.id"
        class="clay-panel !rounded-2xl p-4"
        :class="activeModel === model.id ? 'border-accent/40' : ''"
      >
        <div class="mb-3 flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 class="text-[14px] font-semibold">{{ model.name }}</h2>
            <p class="mt-0.5 font-mono text-[10px] text-dim">
              {{ model.runtime }} · {{ model.sizeMb }} MB · {{ model.ram }} · {{ model.speed }} · kualitas {{ model.quality }}
            </p>
          </div>
          <span v-if="model.recommended" class="clay-chip rounded-full bg-accent/10 px-2 py-1 font-mono text-[10px] text-accent">Rekomendasi</span>
        </div>

        <p class="mb-3 text-[12px] leading-snug text-zinc-300">{{ model.note }}</p>

        <div class="mb-3 grid gap-2 sm:grid-cols-2">
          <div class="clay-inset rounded-xl bg-ink-900 px-3 py-2">
            <p class="mb-1 text-[10px] font-semibold text-accent">Kelebihan</p>
            <p v-for="(item, i) in model.pros" :key="i" class="text-[11px] leading-snug text-zinc-300">+ {{ item }}</p>
          </div>
          <div class="clay-inset rounded-xl bg-ink-900 px-3 py-2">
            <p class="mb-1 text-[10px] font-semibold text-red-300">Kekurangan</p>
            <p v-for="(item, i) in model.cons" :key="i" class="text-[11px] leading-snug text-zinc-300">− {{ item }}</p>
          </div>
        </div>

        <div v-if="model.files.length" class="mb-3 space-y-2">
          <div v-for="file in model.files" :key="file.filename" class="flex items-center gap-2">
            <span class="min-w-0 flex-1 truncate font-mono text-[10px] text-dim">{{ file.filename }}</span>
            <button
              v-if="!hasFile(file.filename)"
              type="button"
              class="clay-btn shrink-0 rounded-lg border border-accent/40 px-2.5 py-1.5 text-[11px] text-accent disabled:opacity-50"
              :disabled="busyFile === file.filename"
              @click="download(file)"
            >{{ busyFile === file.filename ? '…' : 'Unduh' }}</button>
            <button
              v-else
              type="button"
              class="clay-btn shrink-0 rounded-lg border border-line bg-ink-900 px-2.5 py-1.5 text-[11px] text-dim"
              @click="remove(file.filename)"
            >Hapus</button>
          </div>
        </div>

        <p v-else class="mb-3 text-[11px] text-dim">Model Vulkan perlu binary eksternal (lihat docs/MODELS.md).</p>

        <button type="button" class="clay-btn clay-accent-btn w-full rounded-xl py-2.5 text-[12px] font-semibold" @click="useModel(model)">
          {{ activeModel === model.id ? 'Sedang dipakai · buka editor' : 'Pakai model ini' }}
        </button>
      </section>
    </main>
  </div>
</template>
