<script setup lang="ts">
import OptionChips from '~/components/OptionChips.vue'
import RangeField from '~/components/RangeField.vue'
import ToggleSwitch from '~/components/ToggleSwitch.vue'
import { useVidBoost } from '~/composables/useVidBoost'
import { rotateOptions } from '~/utils/vidboost'

const { settings } = useVidBoost()
</script>

<template>
  <div class="space-y-5">
    <OptionChips
      :model-value="String(settings.geometry.rotate)"
      label="Rotasi"
      :options="rotateOptions"
      :columns="4"
      @update:model-value="settings.geometry.rotate = Number($event)"
    />

    <div class="border-t border-line pt-4">
      <ToggleSwitch v-model="settings.geometry.cropEnabled" label="Aktifkan crop" />
      <div v-if="settings.geometry.cropEnabled" class="mt-4 space-y-4">
        <RangeField v-model="settings.geometry.cropX" label="X (kiri)" :min="0" :max="90" />
        <RangeField v-model="settings.geometry.cropY" label="Y (atas)" :min="0" :max="90" />
        <RangeField v-model="settings.geometry.cropW" label="Lebar" :min="10" :max="100" unit="%" />
        <RangeField v-model="settings.geometry.cropH" label="Tinggi" :min="10" :max="100" unit="%" />
      </div>
    </div>
  </div>
</template>
