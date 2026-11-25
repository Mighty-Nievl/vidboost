<script setup lang="ts">
import TabIcon from '~/components/TabIcon.vue'
import type { TabId } from '~/utils/vidboost'

defineProps<{
  active: TabId
  onSelect: (id: TabId) => void
}>()

const tabs: Array<{ id: TabId, label: string }> = [
  { id: 'source', label: 'Source' },
  { id: 'color', label: 'Warna' },
  { id: 'detail', label: 'Detail' },
  { id: 'upscale', label: 'Upscale' },
  { id: 'crop', label: 'Crop' },
  { id: 'text', label: 'Teks' },
  { id: 'export', label: 'Export' },
]
</script>

<template>
  <nav
    class="clay-panel fixed inset-x-0 bottom-[env(safe-area-inset-bottom)] z-40 flex !rounded-none border-x-0 border-b-0 px-1 pt-1 lg:static lg:bottom-auto lg:z-auto lg:w-[68px] lg:shrink-0 lg:flex-col lg:gap-1 lg:border lg:p-2"
    aria-label="Kategori pengaturan"
  >
    <button
      v-for="tab in tabs"
      :key="tab.id"
      type="button"
      class="relative flex min-h-[50px] flex-1 flex-col items-center justify-center gap-1 rounded-xl px-1 text-[10px] transition-all duration-200 ease-spring lg:min-h-[58px] lg:flex-none"
      :class="[
        active === tab.id ? 'bg-accent/15 text-accent' : 'text-dim hover:text-zinc-100',
        tab.id === 'export' ? 'hidden lg:flex' : '',
      ]"
      :aria-current="active === tab.id ? 'page' : undefined"
      @click="onSelect(tab.id)"
    >
      <TabIcon :name="tab.id" class="h-5 w-5 transition-transform duration-200 ease-spring" :class="active === tab.id ? 'scale-110' : ''" />
      <span class="truncate">{{ tab.label }}</span>
      <span v-if="active === tab.id" class="animate-rise absolute bottom-1 h-0.5 w-6 rounded-full bg-accent" aria-hidden="true" />
    </button>
  </nav>
</template>
