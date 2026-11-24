<script setup lang="ts">
import { ref } from 'vue'
import OptionChips from '~/components/OptionChips.vue'
import RangeField from '~/components/RangeField.vue'
import { useVidBoost } from '~/composables/useVidBoost'
import { positionOptions, watermarkModes } from '~/utils/vidboost'

const { settings, hasFile, busy, uploadLogo } = useVidBoost()
const logoInput = ref<HTMLInputElement | null>(null)

function pickLogo() {
  logoInput.value?.click()
}

function onChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) void uploadLogo(file)
  input.value = ''
}
</script>

<template>
  <div class="space-y-5">
    <input ref="logoInput" type="file" accept="image/*" class="hidden" @change="onChange">

    <OptionChips v-model="settings.watermark.mode" label="Jenis watermark" :options="watermarkModes" :columns="3" />

    <div v-if="settings.watermark.mode === 'text'" class="space-y-4">
      <div>
        <label for="wm-text" class="mb-1.5 block text-[12px] text-zinc-300">Teks</label>
        <input id="wm-text" v-model="settings.watermark.text" type="text" class="clay-inset w-full rounded-xl border border-line bg-ink-900 px-3 py-2.5 text-[12px]" placeholder="Contoh: © VidBoost">
      </div>
      <RangeField v-model="settings.watermark.fontSize" label="Ukuran font" :min="24" :max="120" />
      <div>
        <label for="wm-color" class="mb-1.5 block text-[12px] text-zinc-300">Warna</label>
        <input id="wm-color" v-model="settings.watermark.color" type="color" class="clay-btn h-10 w-full rounded-xl border border-line bg-ink-900 p-1">
      </div>
      <OptionChips v-model="settings.watermark.position" label="Posisi" :options="positionOptions" :columns="3" />
    </div>

    <div v-else-if="settings.watermark.mode === 'image'" class="space-y-4">
      <button type="button" class="clay-btn w-full rounded-xl border border-line bg-ink-900 px-3 py-2.5 text-[12px]" :disabled="!hasFile || busy" @click="pickLogo">
        {{ settings.watermark.logoPath ? 'Ganti logo' : 'Pilih logo (PNG/JPG)' }}
      </button>
      <p v-if="settings.watermark.logoPath" class="truncate font-mono text-[10px] text-dim">{{ settings.watermark.logoPath }}</p>
      <RangeField v-model="settings.watermark.scale" label="Ukuran logo" :min="5" :max="60" unit="%" />
      <RangeField v-model="settings.watermark.opacity" label="Opacity" :min="0" :max="1" :step="0.05" />
      <OptionChips v-model="settings.watermark.position" label="Posisi" :options="positionOptions" :columns="3" />
    </div>

    <p v-else class="text-[11px] text-dim">Tidak ada watermark diterapkan.</p>
  </div>
</template>
