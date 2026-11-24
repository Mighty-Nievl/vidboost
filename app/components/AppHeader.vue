<script setup lang="ts">
import { ref } from 'vue'
import { useTheme } from '~/composables/useTheme'
import { useVidBoost } from '~/composables/useVidBoost'

const { theme, toggleTheme } = useTheme()
const { ffmpegMissing, statusLoading, hasFile, persistForExport, resetAll } = useVidBoost()

const resetArmed = ref(false)
let resetTimer: ReturnType<typeof setTimeout> | null = null

function onReset() {
  if (resetArmed.value) {
    resetArmed.value = false
    if (resetTimer !== null) {
      clearTimeout(resetTimer)
      resetTimer = null
    }
    resetAll()
    return
  }
  resetArmed.value = true
  if (resetTimer !== null) clearTimeout(resetTimer)
  resetTimer = setTimeout(() => {
    resetArmed.value = false
    resetTimer = null
  }, 4000)
}

async function goToExport() {
  persistForExport()
  await navigateTo('/export')
}
</script>

<template>
  <header class="sticky top-0 z-30 border-b border-line bg-ink-950">
    <div class="mx-auto flex max-w-[1600px] items-center gap-3 px-4 py-3 sm:px-6">
      <div class="clay-btn flex h-8 w-8 shrink-0 items-center justify-center rounded-[10px] bg-accent text-ink-950">
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
          <rect x="3" y="4" width="18" height="16" rx="2" />
          <path d="M7 4v16M17 4v16M3 12h18" />
        </svg>
      </div>

      <div class="min-w-0">
        <h1 class="text-[14px] font-semibold leading-tight tracking-tight">VidBoost</h1>
        <p class="hidden text-[11px] leading-tight text-dim sm:block">Local video enhancer · powered by ffmpeg</p>
      </div>

      <div class="ml-auto flex items-center gap-2">
        <span
          v-if="statusLoading"
          class="hidden rounded-md border border-line bg-ink-900 px-2 py-1 font-mono text-[10px] text-dim sm:inline-flex"
        >checking…</span>
        <span
          v-else
          class="hidden rounded-md border px-2 py-1 font-mono text-[10px] sm:inline-flex"
          :class="ffmpegMissing ? 'border-red-500/30 text-red-300' : 'border-accent/40 text-accent'"
        >{{ ffmpegMissing ? 'FFmpeg missing' : 'FFmpeg ready' }}</span>

        <button
          type="button"
          class="clay-btn flex h-10 w-10 items-center justify-center rounded-xl border border-line bg-ink-900 text-dim transition-colors hover:text-accent"
          :aria-label="theme === 'clay-light' ? 'Aktifkan dark mode' : 'Aktifkan light mode'"
          :title="theme === 'clay-light' ? 'Dark mode' : 'Light mode'"
          @click="toggleTheme"
        >
          <svg v-if="theme === 'clay-light'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true">
            <path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" />
          </svg>
          <svg v-else class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4" />
          </svg>
        </button>

        <button
          type="button"
          class="clay-btn clay-accent-btn min-h-[40px] rounded-xl px-3 text-[12px] font-semibold lg:hidden"
          @click="goToExport"
        >Export</button>

        <button
          v-if="hasFile"
          type="button"
          class="clay-btn min-h-[40px] rounded-xl border border-line bg-ink-900 px-3 text-[12px] text-zinc-200 transition-colors hover:border-accent/50"
          :class="resetArmed ? 'border-red-500/60 text-red-300' : ''"
          @click="onReset"
        >{{ resetArmed ? 'Yakin reset?' : 'Reset' }}</button>
      </div>
    </div>
  </header>
</template>
