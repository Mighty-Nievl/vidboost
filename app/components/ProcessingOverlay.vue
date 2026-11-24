<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import ProcessLog from '~/components/ProcessLog.vue'
import { useVidBoost } from '~/composables/useVidBoost'

const {
  job, isProcessing, isDone, progressPct, elapsedSeconds, etaSeconds,
  formatElapsed, logEntries, cancelJob, effectiveUpscaleFactor, settings,
} = useVidBoost()

const dismissed = ref(false)
const circumference = 2 * Math.PI * 52
const dashOffset = computed(() => circumference * (1 - progressPct.value / 100))

watch(isProcessing, (value) => {
  if (value) dismissed.value = false
})

function onKey(event: KeyboardEvent) {
  if (event.key === 'Escape' && isProcessing.value) void cancelJob()
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <Transition name="overlay">
    <div
      v-if="!dismissed && (isProcessing || isDone)"
      class="fixed inset-0 z-[60] flex items-center justify-center bg-ink-950/80 px-4 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      :aria-label="isProcessing ? 'Memproses video' : 'Export selesai'"
    >
      <section class="clay-panel w-full max-w-md !rounded-3xl p-5">
        <div class="mb-4 text-center">
          <p class="text-[10px] font-mono uppercase tracking-widest text-dim">
            {{ isProcessing ? 'Exporting' : 'Selesai' }}
          </p>
          <h2 class="mt-1 text-[15px] font-semibold">
            {{ isProcessing ? 'Memproses video' : 'Video siap diunduh' }}
          </h2>
        </div>

        <div class="relative mx-auto mb-4 h-32 w-32">
          <svg class="h-full w-full -rotate-90" viewBox="0 0 120 120" aria-hidden="true">
            <circle cx="60" cy="60" r="52" fill="none" stroke="var(--vb-surface-3)" stroke-width="9" />
            <circle
              cx="60" cy="60" r="52" fill="none" stroke="var(--vb-accent)" stroke-width="9"
              stroke-linecap="round" :stroke-dasharray="circumference" :stroke-dashoffset="dashOffset"
              class="transition-[stroke-dashoffset] duration-500 ease-out"
            />
          </svg>
          <div class="absolute inset-0 flex flex-col items-center justify-center">
            <span class="font-mono text-2xl font-semibold tabular-nums text-accent">{{ progressPct }}%</span>
            <span class="font-mono text-[10px] text-dim">
              {{ isProcessing ? 'Sisa ' + (etaSeconds === null ? '—' : formatElapsed(etaSeconds)) : '100%' }}
            </span>
          </div>
        </div>

        <p class="mb-2 truncate text-center text-[11px] text-zinc-300">
          {{ isDone ? 'Selesai' : (job?.stage || 'Menyiapkan…') }}
        </p>
        <p v-if="settings.upscale.enabled" class="mb-4 text-center font-mono text-[10px] text-dim">
          Faktor upscale efektif
          <span class="text-accent">{{ Number(effectiveUpscaleFactor.toFixed(2)) }}x</span>
          <span v-if="settings.upscale.ai" class="text-zinc-300"> · {{ settings.upscale.aiModel }}</span>
        </p>
        <div v-else class="mb-4" />

        <div class="mb-4 flex items-center justify-center gap-4 font-mono text-[11px]">
          <span class="text-dim">Berjalan <span class="text-zinc-200">{{ formatElapsed(elapsedSeconds) }}</span></span>
          <span class="text-dim">Sisa <span class="text-accent">{{ etaSeconds === null ? '—' : formatElapsed(etaSeconds) }}</span></span>
        </div>

        <ProcessLog v-if="logEntries.length" :entries="logEntries" />

        <div class="mt-4 flex gap-2">
          <button
            v-if="isProcessing"
            type="button"
            class="clay-btn flex-1 rounded-xl border border-red-500/40 bg-ink-900 py-2.5 text-[12px] text-red-300"
            @click="cancelJob"
          >Batal</button>
          <button
            v-else
            type="button"
            class="clay-btn clay-accent-btn flex-1 rounded-xl py-2.5 text-[12px] font-semibold"
            @click="dismissed = true"
          >Lihat hasil</button>
        </div>
      </section>
    </div>
  </Transition>
</template>

<style>
.overlay-enter-active,
.overlay-leave-active { transition: opacity 220ms ease; }
.overlay-enter-from,
.overlay-leave-to { opacity: 0; }
</style>
