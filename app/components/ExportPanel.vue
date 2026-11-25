<script setup lang="ts">
import { ref } from 'vue'
import OptionChips from '~/components/OptionChips.vue'
import ProcessLog from '~/components/ProcessLog.vue'
import RangeField from '~/components/RangeField.vue'
import ToggleSwitch from '~/components/ToggleSwitch.vue'
import { useVidBoost } from '~/composables/useVidBoost'

const {
  settings, formats, codecOptions, presetOptions, chips,
  job, meta, busy, isProcessing, isDone, progressPct,
  startProcess, cancelJob, downloadUrl, encodeWarnings,
  logEntries, elapsedSeconds, etaSeconds, formatElapsed, nativeMobile,
} = useVidBoost()

const showWarnings = ref(false)

function useVp9() {
  settings.output.format = 'webm'
  settings.output.codec = 'vp9'
  showWarnings.value = false
}

function lowerFactor() {
  const m = meta.value
  if (!m) return
  const safe = Math.sqrt(9_200_000 / (m.width * m.height))
  settings.upscale.factor = Math.max(1, Math.floor(safe * 10) / 10)
  showWarnings.value = false
}
</script>

<template>
  <div class="space-y-5">
    <div v-if="encodeWarnings.length" class="relative">
      <button
        type="button"
        class="clay-btn flex w-full items-center gap-2 rounded-xl border border-accent/40 bg-accent/10 px-3 py-2.5 text-left text-[12px] text-accent"
        :aria-expanded="showWarnings"
        @click="showWarnings = !showWarnings"
      >
        <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
          <path d="M12 3L2 20h20L12 3z" />
          <path d="M12 9v5M12 17h.01" />
        </svg>
        <span class="flex-1">{{ encodeWarnings.length }} peringatan encode — ketuk untuk lihat</span>
        <span class="font-mono text-[10px]">{{ showWarnings ? '−' : '+' }}</span>
      </button>

      <div v-if="showWarnings" class="clay-panel absolute left-0 right-0 top-11 z-30 space-y-3 p-4">
        <p v-for="(warning, index) in encodeWarnings" :key="index" class="text-[11px] leading-snug text-zinc-200">
          <span class="mr-1 text-accent">•</span>{{ warning }}
        </p>
        <div class="flex flex-wrap gap-2 border-t border-line pt-3">
          <button type="button" class="clay-btn rounded-xl border border-accent/40 px-3 py-2 text-[11px] text-accent" @click="useVp9">
            Ganti ke WebM + VP9
          </button>
          <button
            v-if="settings.upscale.enabled && !settings.upscale.ai"
            type="button"
            class="clay-btn rounded-xl border border-line bg-ink-900 px-3 py-2 text-[11px]"
            @click="lowerFactor"
          >Turunkan faktor</button>
          <button type="button" class="clay-btn rounded-xl px-3 py-2 text-[11px] text-dim" @click="showWarnings = false">Mengerti</button>
        </div>
      </div>
    </div>

    <OptionChips v-model="settings.output.format" label="Format" :options="formats" :columns="4" />
    <OptionChips v-model="settings.output.codec" label="Codec" :options="codecOptions" :columns="5" />
    <OptionChips v-model="settings.output.preset" label="Preset" :options="presetOptions" :columns="5" />
    <RangeField v-model="settings.output.crf" label="CRF" :min="18" :max="34" hint="Lebih rendah berarti kualitas lebih tinggi dan ukuran lebih besar." />

    <div class="border-t border-line pt-4">
      <ToggleSwitch v-model="settings.output.audio" label="Sertakan audio" />
    </div>

    <div v-if="chips.length" class="flex flex-wrap gap-1.5 border-t border-line pt-4">
      <span v-for="chip in chips" :key="chip" class="clay-chip rounded-lg border border-line bg-ink-900 px-2 py-1 font-mono text-[10px] text-dim">{{ chip }}</span>
    </div>

    <button
      type="button"
      class="clay-btn clay-accent-btn w-full px-4 py-3 text-[13px] font-semibold disabled:cursor-not-allowed disabled:bg-ink-800 disabled:text-dim disabled:shadow-none"
      :disabled="!job || busy || isProcessing || nativeMobile"
      @click="startProcess"
    >{{ isProcessing ? 'Memproses...' : 'Enhance video' }}</button>

    <div v-if="job && (isProcessing || isDone)" class="space-y-2">
      <div class="flex items-center gap-2">
        <span
          v-if="isProcessing"
          class="h-2 w-2 shrink-0 animate-pulse-soft rounded-full bg-accent"
          aria-hidden="true"
        />
        <span class="min-w-0 flex-1 truncate text-[11px] text-zinc-300">
          {{ isDone ? 'Selesai' : (job.stage || 'Menyiapkan') }}
        </span>
        <span class="clay-chip shrink-0 rounded-full bg-ink-900 px-2 py-0.5 font-mono text-[10px] tabular-nums text-dim">
          {{ formatElapsed(elapsedSeconds) }}
        </span>
        <span v-if="isProcessing" class="clay-chip shrink-0 rounded-full bg-ink-900 px-2 py-0.5 font-mono text-[10px] tabular-nums text-accent">
          Sisa {{ etaSeconds === null ? '—' : formatElapsed(etaSeconds) }}
        </span>
        <span class="clay-chip shrink-0 rounded-full bg-accent/10 px-2 py-0.5 font-mono text-[10px] tabular-nums text-accent">
          {{ progressPct }}%
        </span>
        <button
          v-if="isProcessing"
          type="button"
          class="clay-btn shrink-0 rounded-full border border-red-500/40 bg-ink-900 px-2.5 py-1 text-[10px] text-red-300"
          @click="cancelJob"
        >Batal</button>
      </div>
      <div
        class="clay-inset h-2 w-full overflow-hidden rounded-full bg-ink-900"
        role="progressbar"
        :aria-valuenow="progressPct"
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <div
          class="animate-bar-glow h-full rounded-full bg-accent transition-[width] duration-300"
          :style="{ width: `${progressPct}%` }"
        />
      </div>
    </div>

    <p v-if="nativeMobile" class="clay-inset rounded-xl bg-ink-900 px-3 py-2.5 text-[11px] text-dim">
      Export belum tersedia di build Android ini. Engine lokal sedang dikerjakan.
    </p>

    <ProcessLog v-if="logEntries.length || isProcessing || isDone" :entries="logEntries" />

    <div v-if="isDone" class="animate-rise flex flex-wrap items-center justify-between gap-3 rounded-xl border border-accent/30 bg-accent/5 px-3 py-2.5">
      <span class="text-[12px] text-accent">Selesai. Video siap diunduh.</span>
      <a :href="downloadUrl" class="clay-btn rounded-xl border border-accent/40 px-3 py-1.5 text-[12px] font-medium text-accent">Download hasil</a>
    </div>
  </div>
</template>
