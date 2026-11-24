<script setup lang="ts">
import RangeField from '~/components/RangeField.vue'
import ToggleSwitch from '~/components/ToggleSwitch.vue'
import { useVidBoost } from '~/composables/useVidBoost'

const { settings, resetColor } = useVidBoost()
</script>

<template>
  <div class="space-y-4">
    <ToggleSwitch v-model="settings.color.enabled" label="Aktifkan koreksi warna" />
    <div class="space-y-4">
      <RangeField v-model="settings.color.brightness" label="Brightness" :min="-1" :max="1" :step="0.05" :disabled="!settings.color.enabled" />
      <RangeField v-model="settings.color.contrast" label="Contrast" :min="0" :max="3" :step="0.05" :disabled="!settings.color.enabled" />
      <RangeField v-model="settings.color.saturation" label="Saturation" :min="0" :max="3" :step="0.05" :disabled="!settings.color.enabled" />
      <RangeField v-model="settings.color.gamma" label="Gamma" :min="0.1" :max="3" :step="0.05" :disabled="!settings.color.enabled" />
    </div>
    <button type="button" class="clay-btn rounded-xl border border-line bg-ink-900 px-3 py-2 text-[12px]" @click="resetColor">
      Reset ke netral
    </button>
  </div>
</template>
