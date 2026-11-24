<script setup lang="ts">
import { ref } from 'vue'
import RangeField from '~/components/RangeField.vue'
import { useVidBoost } from '~/composables/useVidBoost'
import type { PreviewMode } from '~/utils/vidboost'

const {
  job, busy, isProcessing, meta, chips, previewMode, split,
  sourceUrl, afterSource, thumbUrl, visibleError, dismissError, hasFile, cancelJob, encodeWarnings,
} = useVidBoost()

const beforeVideo = ref<HTMLVideoElement | null>(null)
const afterVideo = ref<HTMLVideoElement | null>(null)

const modes: Array<{ value: PreviewMode, label: string }> = [
  { value: 'before', label: 'Sebelum' },
  { value: 'after', label: 'Sesudah' },
  { value: 'split', label: 'Split' },
]

function syncPlay() {
  void afterVideo.value?.play().catch(() => {})
}

function syncPause() {
  afterVideo.value?.pause()
}

function syncTime() {
  const before = beforeVideo.value
  const after = afterVideo.value
  if (!before || !after) return
  if (Math.abs(before.currentTime - after.currentTime) > 0.15) {
    after.currentTime = before.currentTime
  }
}
</script>

<template>
  <div>
    <Transition name="notice">
      <div
        v-if="visibleError"
        role="alert"
        class="clay-panel relative mb-3 flex items-start gap-3 overflow-hidden px-3 py-2.5 text-[12px] leading-snug text-red-300"
      >
        <span class="min-w-0 flex-1">{{ visibleError }}</span>
        <button type="button" class="shrink-0 px-1 text-[13px] leading-none text-red-300/70" aria-label="Tutup pesan" @click="dismissError">×</button>
        <span class="animate-notice-timer absolute bottom-0 left-0 h-0.5 bg-accent/60" aria-hidden="true" />
      </div>
    </Transition>

    <div class="mb-3 flex flex-wrap items-center gap-2">
      <div class="clay-inset inline-flex rounded-2xl bg-ink-900 p-1">
        <button
          v-for="mode in modes"
          :key="mode.value"
          type="button"
          class="clay-chip min-h-[38px] rounded-xl px-3 font-mono text-[11px] transition-colors"
          :class="previewMode === mode.value ? 'bg-accent/15 text-accent' : 'text-dim hover:text-zinc-100'"
          :aria-pressed="previewMode === mode.value"
          @click="previewMode = mode.value"
        >{{ mode.label }}</button>
      </div>
      <span class="ml-auto truncate font-mono text-[10px] text-dim">{{ job ? job.fileName : 'idle' }}</span>
    </div>

    <div class="clay-panel relative aspect-video w-full overflow-hidden !rounded-2xl bg-black">
      <template v-if="!job">
        <div class="flex h-full w-full flex-col items-center justify-center gap-3 px-6 text-center">
          <svg class="h-8 w-8 text-ink-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
            <rect x="3" y="4" width="18" height="16" rx="2" />
            <path d="M7 4v16M17 4v16M3 12h18" />
          </svg>
          <p class="text-[13px]">Video kamu akan tampil di sini</p>
          <p class="text-[11px] text-dim">Pilih file di tab Source untuk mulai.</p>
        </div>
        <div v-if="busy" class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
          <div class="animate-shimmer h-full w-1/2 bg-gradient-to-r from-transparent via-accent/10 to-transparent" />
        </div>
      </template>

      <template v-else>
        <video
          v-if="previewMode !== 'after'"
          ref="beforeVideo"
          :src="sourceUrl"
          :poster="thumbUrl"
          controls loop preload="metadata" playsinline
          aria-label="Video sebelum enhancement"
          class="absolute inset-0 h-full w-full object-contain"
          @play="syncPlay" @pause="syncPause" @timeupdate="syncTime"
        />
        <video
          v-if="previewMode !== 'before'"
          ref="afterVideo"
          :src="afterSource"
          :poster="thumbUrl"
          controls loop preload="metadata" playsinline
          aria-label="Video hasil enhancement"
          class="absolute inset-0 h-full w-full object-contain"
          :class="previewMode === 'split' ? 'pointer-events-none' : ''"
          :style="previewMode === 'split' ? { clipPath: `inset(0 0 0 ${split}%)` } : undefined"
        />

        <div v-if="previewMode === 'split'" class="pointer-events-none absolute inset-x-0 top-0 flex justify-between p-2">
          <span class="rounded-lg border border-line bg-ink-950/80 px-1.5 py-0.5 font-mono text-[10px]">SEBELUM</span>
          <span class="rounded-lg border border-accent/40 bg-ink-950/80 px-1.5 py-0.5 font-mono text-[10px] text-accent">SESUDAH</span>
        </div>
        <div v-else class="pointer-events-none absolute left-2 top-2">
          <span
            class="rounded-lg border bg-ink-950/80 px-1.5 py-0.5 font-mono text-[10px]"
            :class="previewMode === 'after' ? 'border-accent/40 text-accent' : 'border-line'"
          >{{ previewMode === 'after' ? 'SESUDAH' : 'SEBELUM' }}</span>
        </div>

        <div v-if="isProcessing" role="status" class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 bg-ink-900/80 backdrop-blur-sm">
          <span class="h-7 w-7 animate-spin animate-pulse-soft rounded-full border-2 border-ink-700 border-t-accent" aria-hidden="true" />
          <span class="font-mono text-[11px]">{{ job.stage || 'Memproses' }} · {{ Math.round(job.progress) }}%</span>
          <button type="button" class="clay-btn rounded-xl border border-red-500/40 bg-ink-900 px-3 py-2 text-[11px] text-red-300" @click="cancelJob">
            Batal
          </button>
        </div>
      </template>
    </div>

    <div v-if="hasFile && previewMode === 'split'" class="clay-panel mt-3 !rounded-2xl px-4 py-3">
      <RangeField v-model="split" label="Posisi pembagi" :min="0" :max="100" :step="1" unit="%" />
    </div>

    <div class="mt-3">
      <div v-if="chips.length" class="flex flex-wrap gap-1.5">
        <span v-for="chip in chips" :key="chip" class="clay-chip rounded-lg border border-line bg-ink-900 px-2 py-1 font-mono text-[10px] text-dim">{{ chip }}</span>
        <span v-if="encodeWarnings.length" class="clay-chip rounded-lg border border-accent/40 px-2 py-1 font-mono text-[10px] text-accent">⚠ {{ encodeWarnings.length }} encode</span>
      </div>
      <p v-else class="text-[11px] text-dim">Pengaturan aktif akan tampil di sini.</p>
    </div>
  </div>
</template>
