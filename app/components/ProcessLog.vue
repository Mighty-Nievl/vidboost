<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import type { LogEntry } from '~/utils/vidboost'

const props = defineProps<{ entries: LogEntry[] }>()

const expanded = ref(true)
const body = ref<HTMLElement | null>(null)

watch(
  () => props.entries.length,
  async () => {
    await nextTick()
    if (body.value) body.value.scrollTop = body.value.scrollHeight
  },
)
</script>

<template>
  <section class="clay-panel overflow-hidden !rounded-2xl">
    <button
      type="button"
      class="flex w-full items-center justify-between gap-3 px-3 py-2.5 text-left"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span class="flex items-center gap-2 text-[12px] font-medium">
        Log proses
        <span class="clay-chip rounded-full px-2 py-0.5 font-mono text-[10px] text-dim">{{ entries.length }}</span>
      </span>
      <svg
        class="h-3.5 w-3.5 text-dim transition-transform duration-200"
        :class="expanded ? 'rotate-180' : ''"
        viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"
      >
        <path d="m6 9 6 6 6-6" />
      </svg>
    </button>

    <div v-if="expanded" ref="body" class="clay-inset max-h-[132px] overflow-y-auto px-3 py-2.5" aria-live="polite">
      <p v-if="!entries.length" class="font-mono text-[10px] text-dim">Belum ada log.</p>
      <div
        v-for="entry in entries"
        :key="entry.id"
        class="animate-rise flex gap-2 py-0.5 font-mono text-[10px] leading-snug"
        :class="entry.tone === 'error' ? 'text-red-300' : entry.tone === 'success' ? 'text-accent' : 'text-zinc-300'"
      >
        <span class="shrink-0 text-dim">{{ entry.time }}</span>
        <span class="min-w-0 flex-1 break-words">{{ entry.text }}</span>
      </div>
    </div>
  </section>
</template>
