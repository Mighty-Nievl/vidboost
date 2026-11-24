<script setup lang="ts">
import { computed, useId } from 'vue'

const props = defineProps<{
  label: string
  modelValue: number
  min: number
  max: number
  step?: number
  unit?: string
  hint?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const id = useId()

const display = computed(() => {
  const step = props.step ?? 1
  const decimals = step < 1 ? (String(step).split('.')[1]?.length ?? 0) : 0
  const raw = props.modelValue.toFixed(decimals)
  return raw.includes('.') ? raw.replace(/0+$/, '').replace(/\.$/, '') : raw
})

function onInput(event: Event) {
  emit('update:modelValue', Number((event.target as HTMLInputElement).value))
}
</script>

<template>
  <div class="space-y-1.5">
    <div class="flex items-baseline justify-between gap-3">
      <label :for="id" class="text-[12px] text-zinc-300">{{ label }}</label>
      <span class="font-mono text-[12px] tabular-nums text-accent">
        {{ display }}<span v-if="unit" class="ml-0.5 text-dim">{{ unit }}</span>
      </span>
    </div>
    <input
      :id="id"
      type="range"
      :min="min"
      :max="max"
      :step="step ?? 1"
      :value="modelValue"
      :disabled="disabled"
      class="w-full"
      @input="onInput"
    >
    <p v-if="hint" class="text-[11px] leading-snug text-dim">{{ hint }}</p>
  </div>
</template>
