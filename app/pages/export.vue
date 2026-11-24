<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import AppHeader from '~/components/AppHeader.vue'
import ExportPanel from '~/components/ExportPanel.vue'
import ProcessingOverlay from '~/components/ProcessingOverlay.vue'
import VideoPreview from '~/components/VideoPreview.vue'
import { useTheme } from '~/composables/useTheme'
import { useVidBoost } from '~/composables/useVidBoost'

const { initVidBoost, disposeVidBoost, job } = useVidBoost()
const { initTheme } = useTheme()

async function goBack() {
  await navigateTo('/')
}

onMounted(async () => {
  initTheme()
  await initVidBoost()
})

onBeforeUnmount(() => disposeVidBoost())
</script>

<template>
  <div class="min-h-screen bg-ink-950 pb-10 text-zinc-100">
    <AppHeader />

    <main class="mx-auto max-w-4xl space-y-5 px-4 pt-4 sm:px-6">
      <div class="flex items-center gap-3">
        <button type="button" class="clay-btn min-h-[40px] rounded-xl border border-line bg-ink-900 px-3 text-[12px]" @click="goBack">
          ← Kembali
        </button>
        <div>
          <h2 class="text-[14px] font-semibold">Export</h2>
          <p class="hidden text-[11px] text-dim sm:block">Atur output akhir video kamu</p>
        </div>
      </div>

      <div v-if="!job" class="clay-panel animate-rise px-6 py-12 text-center">
        <p class="text-[13px]">Belum ada video untuk diexport.</p>
        <p class="mt-1 text-[11px] text-dim">Pilih video dulu di halaman utama.</p>
        <button type="button" class="clay-btn clay-accent-btn mt-5 min-h-[44px] rounded-xl px-5 text-[12px] font-semibold" @click="goBack">
          Pilih video
        </button>
      </div>

      <template v-else>
        <VideoPreview />
        <section class="clay-panel animate-rise px-4 py-4">
          <ExportPanel />
        </section>
      </template>
    </main>

    <ProcessingOverlay />
  </div>
</template>
