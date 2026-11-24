<script setup lang="ts">
import OptionChips from '~/components/OptionChips.vue'
import ToggleSwitch from '~/components/ToggleSwitch.vue'
import { useVidBoost } from '~/composables/useVidBoost'
import { factorOptions } from '~/utils/vidboost'

const { settings, aiAvailable, aiHeavy, aiBlockReason, aiChipOptions, effectiveUpscaleFactor } = useVidBoost()
</script>

<template>
  <div class="space-y-5">
    <ToggleSwitch v-model="settings.upscale.enabled" label="Aktifkan upscale" />

    <OptionChips
      :model-value="String(settings.upscale.factor)"
      label="Faktor"
      :options="factorOptions"
      :columns="4"
      :disabled="!settings.upscale.enabled"
      @update:model-value="settings.upscale.factor = Number($event)"
    />

    <p
      v-if="settings.upscale.enabled && !settings.upscale.ai && effectiveUpscaleFactor < settings.upscale.factor"
      class="clay-inset rounded-xl bg-ink-900 px-3 py-2 text-[11px] text-dim"
    >
      Faktor efektif <span class="font-mono text-accent">{{ Number(effectiveUpscaleFactor.toFixed(2)) }}x</span>
      — dibatasi agar video tetap bisa diputar di HP.
    </p>

    <div class="border-t border-line pt-4">
      <ToggleSwitch
        v-model="settings.upscale.ai"
        label="Gunakan AI lokal"
        :disabled="!aiAvailable || aiHeavy || !settings.upscale.enabled"
        :hint="aiHeavy ? aiBlockReason : 'Lebih tajam, tapi butuh binary AI dan ruang disk.'"
      />
      <div v-if="aiHeavy" role="status" class="clay-inset mt-3 rounded-xl bg-ink-900 px-3 py-2.5 text-[11px] leading-snug text-dim">
        {{ aiBlockReason }} Pakai Lanczos atau resize video dulu.
      </div>
      <div v-else-if="aiAvailable" class="mt-3">
        <OptionChips v-model="settings.upscale.aiModel" label="Model" :options="aiChipOptions" :columns="1" :disabled="!settings.upscale.ai" />
      </div>
      <div v-else class="clay-inset mt-3 rounded-xl bg-ink-900 px-3 py-2.5 text-[11px] leading-snug text-dim">
        AI belum aktif. Set <span class="font-mono">AI_UPSCALER_BIN</span> lalu restart server. Sementara pakai Lanczos.
      </div>
    </div>

    <button type="button" class="clay-btn w-full rounded-xl border border-line bg-ink-900 py-2.5 text-[12px]" @click="navigateTo('/models')">
      Kelola model AI →
    </button>
  </div>
</template>
