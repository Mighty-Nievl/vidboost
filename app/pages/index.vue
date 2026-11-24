<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import AppHeader from '~/components/AppHeader.vue'
import BottomNav from '~/components/BottomNav.vue'
import ColorPanel from '~/components/ColorPanel.vue'
import CropPanel from '~/components/CropPanel.vue'
import DetailPanel from '~/components/DetailPanel.vue'
import ExportPanel from '~/components/ExportPanel.vue'
import SourcePanel from '~/components/SourcePanel.vue'
import TextPanel from '~/components/TextPanel.vue'
import UpscalePanel from '~/components/UpscalePanel.vue'
import VideoPreview from '~/components/VideoPreview.vue'
import { useTheme } from '~/composables/useTheme'
import { useVidBoost } from '~/composables/useVidBoost'
import type { TabId } from '~/utils/vidboost'

const { initVidBoost, disposeVidBoost } = useVidBoost()
const { initTheme } = useTheme()

const isMobile = ref(false)
const activeTab = ref<TabId>('source')
const sheetOpen = ref(true)
const sheetExpanded = ref(true)

const panelComponents = {
  source: SourcePanel,
  color: ColorPanel,
  detail: DetailPanel,
  upscale: UpscalePanel,
  crop: CropPanel,
  text: TextPanel,
  export: ExportPanel,
}

const activeComponent = computed(() => panelComponents[activeTab.value])
const activeTitle = computed(() => ({
  source: 'Source',
  color: 'Warna',
  detail: 'Detail',
  upscale: 'Upscale',
  crop: 'Crop & rotate',
  text: 'Teks',
  export: 'Export',
})[activeTab.value])

function onViewportChange() {
  isMobile.value = window.matchMedia('(max-width: 1023px)').matches
  if (isMobile.value && activeTab.value === 'export') activeTab.value = 'source'
}

function selectTab(id: TabId) {
  if (activeTab.value === id) {
    if (isMobile.value) sheetOpen.value = !sheetOpen.value
    return
  }
  activeTab.value = id
  sheetOpen.value = true
  sheetExpanded.value = true
}

onMounted(async () => {
  initTheme()
  onViewportChange()
  window.addEventListener('resize', onViewportChange)
  await initVidBoost()
})

onBeforeUnmount(() => {
  disposeVidBoost()
  window.removeEventListener('resize', onViewportChange)
})
</script>

<template>
  <div class="min-h-screen bg-ink-950 pb-[9.5rem] text-zinc-100 lg:pb-6">
    <AppHeader />

    <main class="mx-auto max-w-[1600px] px-4 pt-4 sm:px-6">
      <div class="grid grid-cols-1 gap-5 lg:grid-cols-[minmax(0,1fr)_380px]">
        <div class="order-1 -mx-4 min-w-0 self-start sticky top-[3.5rem] z-20 bg-ink-950/95 px-4 pb-3 backdrop-blur-sm sm:-mx-6 sm:px-6 lg:static lg:z-auto lg:mx-0 lg:bg-transparent lg:p-0 lg:backdrop-blur-none lg:self-start">
          <VideoPreview />
        </div>

        <div class="order-2 min-w-0 lg:flex lg:gap-3 lg:self-start">
          <BottomNav :active="activeTab" :on-select="selectTab" />

          <Transition name="sheet">
            <section
              v-show="sheetOpen || !isMobile"
              class="clay-panel fixed inset-x-0 bottom-[58px] z-30 overflow-y-auto px-4 pb-6 pt-2 lg:static lg:z-auto lg:max-h-[calc(100vh_-_5.5rem)] lg:w-full lg:flex-1 lg:overflow-y-auto lg:px-4 lg:pb-4 lg:pt-3"
              :class="[
                isMobile ? (sheetExpanded ? 'max-h-[38vh]' : 'max-h-[108px]') : '',
                isMobile && !sheetExpanded ? 'overflow-hidden' : '',
              ]"
              :aria-label="`Panel ${activeTitle}`"
            >
              <button
                v-if="isMobile"
                type="button"
                class="clay-inset mx-auto mb-2 flex h-5 w-12 items-center justify-center rounded-full bg-ink-900"
                :aria-label="sheetExpanded ? 'Ciutkan panel' : 'Buka panel'"
                @click="sheetExpanded = !sheetExpanded"
              >
                <span class="h-1 w-7 rounded-full bg-ink-700" />
              </button>

              <div class="mb-4 flex items-center justify-between gap-3">
                <h2 class="text-[13px] font-medium">{{ activeTitle }}</h2>
                <button v-if="isMobile" type="button" class="clay-btn min-h-[36px] rounded-xl px-2 text-[11px] text-dim" @click="sheetOpen = false">
                  Tutup
                </button>
              </div>

              <Transition name="tab" mode="out-in">
                <div v-if="sheetExpanded || !isMobile" :key="activeTab" class="space-y-5">
                  <component :is="activeComponent" />
                </div>
              </Transition>
            </section>
          </Transition>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
.tab-enter-active,
.tab-leave-active {
  transition: opacity 180ms ease, transform 260ms cubic-bezier(0.34, 1.56, 0.64, 1);
}
.tab-enter-from { opacity: 0; transform: translateY(14px) scale(0.98); }
.tab-leave-to { opacity: 0; transform: translateY(-8px) scale(0.99); }

.sheet-enter-active,
.sheet-leave-active {
  transition: transform 320ms cubic-bezier(0.34, 1.56, 0.64, 1), opacity 200ms ease;
}
.sheet-enter-from,
.sheet-leave-to { transform: translateY(32px); opacity: 0; }

.notice-enter-active,
.notice-leave-active {
  transition: transform 360ms cubic-bezier(0.34, 1.56, 0.64, 1), opacity 220ms ease;
}
.notice-enter-from,
.notice-leave-to { transform: translateY(-16px) scale(0.97); opacity: 0; }
</style>
