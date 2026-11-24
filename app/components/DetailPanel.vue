<script setup lang="ts">
import RangeField from '~/components/RangeField.vue'
import ToggleSwitch from '~/components/ToggleSwitch.vue'
import { useVidBoost } from '~/composables/useVidBoost'

const { settings } = useVidBoost()
</script>

<template>
  <div class="space-y-4">
    <ToggleSwitch v-model="settings.detail.enabled" label="Aktifkan detail" />
    <RangeField v-model="settings.detail.sharpen" label="Sharpen" :min="0" :max="100" :disabled="!settings.detail.enabled" />
    <RangeField
      v-model="settings.detail.denoise"
      label="Denoise"
      :min="0"
      :max="100"
      :disabled="!settings.detail.enabled"
      hint="Denoise tinggi bisa menghilangkan detail halus."
    />
  </div>
</template>
