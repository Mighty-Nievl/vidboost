<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ name: string }>()

const paths: Record<string, string[]> = {
  source: ['M3 4h18v16H3z', 'M3 12h18', 'M8 4v16'],
  color: ['M4 7h16', 'M4 17h16', 'M8 4v6', 'M16 14v6'],
  detail: ['M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z'],
  upscale: ['M4 9V4h5', 'M20 15v5h-5', 'M4 4l6 6', 'M20 20l-6-6'],
  crop: ['M6 2v16h16', 'M2 6h16v16'],
  text: ['M4 6h16', 'M12 6v14', 'M8 20h8'],
  export: ['M12 3v12', 'M8 11l4 4 4-4', 'M4 19h16'],
}

const shapes = computed(() => paths[props.name] ?? [])
</script>

<template>
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path v-for="(path, index) in shapes" :key="index" :d="path" />
  </svg>
</template>
