<script setup lang="ts">
import { ref } from 'vue'
import { useVidBoost } from '~/composables/useVidBoost'
import { formatDuration, formatSize } from '~/utils/vidboost'

const {
  job, busy, meta, ffmpegMissing, isDragging, uploadVideo,
} = useVidBoost()

const fileInput = ref<HTMLInputElement | null>(null)

function pickFile() {
  fileInput.value?.click()
}

function onChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) void uploadVideo(file)
  input.value = ''
}

function onDrop(event: DragEvent) {
  isDragging.value = false
  const file = event.dataTransfer?.files?.[0]
  if (file) void uploadVideo(file)
}
</script>

<template>
  <div class="space-y-3">
    <input ref="fileInput" type="file" accept="video/*" class="hidden" @change="onChange">

    <div
      v-if="!job"
      class="clay-inset rounded-2xl border border-dashed border-line bg-ink-900/60 px-4 py-8 text-center"
      :class="isDragging ? 'border-accent/60 bg-accent/5' : ''"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="onDrop"
    >
      <button type="button" class="clay-btn clay-accent-btn min-h-[44px] rounded-xl px-4 py-2 text-[12px] font-semibold" @click="pickFile">
        Pilih file video
      </button>
      <p class="mt-2 text-[11px] text-dim">atau tarik file ke sini · maks 400 MB</p>
    </div>

    <div v-else class="space-y-3">
      <div class="flex items-start gap-3">
        <div class="min-w-0 flex-1">
          <p class="truncate text-[13px]">{{ job.fileName }}</p>
          <p class="mt-0.5 font-mono text-[11px] text-dim">{{ formatSize(job.size) }}</p>
        </div>
        <button type="button" class="clay-btn min-h-[40px] rounded-xl border border-line bg-ink-900 px-3 text-[12px]" :disabled="busy" @click="pickFile">
          Ganti file
        </button>
      </div>
      <dl class="grid grid-cols-2 gap-x-4 gap-y-2 border-t border-line pt-3 text-[11px]">
        <div><dt class="text-dim">Resolusi</dt><dd class="mt-0.5 font-mono">{{ meta ? `${meta.width}x${meta.height}` : '-' }}</dd></div>
        <div><dt class="text-dim">Durasi</dt><dd class="mt-0.5 font-mono">{{ meta ? formatDuration(meta.duration) : '-' }}</dd></div>
        <div><dt class="text-dim">FPS</dt><dd class="mt-0.5 font-mono">{{ meta ? meta.fps : '-' }}</dd></div>
        <div><dt class="text-dim">Codec</dt><dd class="mt-0.5 truncate font-mono">{{ meta?.codec || '-' }}</dd></div>
      </dl>
    </div>

    <div v-if="ffmpegMissing" role="alert" class="rounded-xl border border-red-500/30 bg-red-500/5 px-3 py-2.5 text-[12px] text-red-300">
      FFmpeg tidak ditemukan di server. Cek environment FFMPEG_PATH.
    </div>
  </div>
</template>
