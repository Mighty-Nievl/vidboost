<script setup lang="ts">
import { computed } from 'vue'

interface ChipOption {
  value: string
  label?: string
  hint?: string
}

const props = withDefaults(defineProps<{
  options: Array<string | ChipOption>
  modelValue: string
  label?: string
  disabled?: boolean
  columns?: number
}>(), {
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const normalized = computed<ChipOption[]>(() =>
  props.options.map((option) =>
    typeof option === 'string' ? { value: option, label: option } : option,
  ),
)

const gridColumns = computed(() => props.columns ?? Math.min(normalized.value.length, 4))

function isActive(value: string): boolean {
  return value === props.modelValue
}
</script>

<template>
  <div>
    <p v-if="label" class="mb-1.5 text-[12px] text-zinc-300">{{ label }}</p>
    <div class="grid gap-1.5" :style="{ gridTemplateColumns: `repeat(${gridColumns}, minmax(0, 1fr))` }">
      <button
        v-for="(option, index) in normalized"
        :key="option.value"
        type="button"
        class="clay-chip animate-chip-in min-h-[40px] border px-2 py-2 text-[11px] disabled:cursor-not-allowed disabled:opacity-40"
        :class="isActive(option.value)
          ? 'border-accent/50 bg-accent/15 text-accent'
          : 'border-line bg-ink-900 text-dim hover:text-zinc-100'"
        :style="{ animationDelay: `${index * 35}ms` }"
        :aria-pressed="isActive(option.value)"
        :disabled="disabled"
        @click="emit('update:modelValue', option.value)"
      >
        <span class="block truncate">{{ option.label ?? option.value }}</span>
        <span v-if="option.hint" class="mt-0.5 block truncate text-[9px] opacity-70">{{ option.hint }}</span>
      </button>
    </div>
  </div>
</template>
