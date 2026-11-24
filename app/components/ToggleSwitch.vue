<script setup lang="ts">
defineProps<{
  label: string
  modelValue: boolean
  hint?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function onChange(event: Event) {
  emit('update:modelValue', (event.target as HTMLInputElement).checked)
}
</script>

<template>
  <label
    class="flex items-start justify-between gap-4"
    :class="disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'"
  >
    <span class="min-w-0">
      <span class="block text-[12px] text-zinc-200">{{ label }}</span>
      <span v-if="hint" class="mt-0.5 block text-[11px] leading-snug text-dim">{{ hint }}</span>
    </span>
    <input
      type="checkbox"
      class="peer sr-only"
      :checked="modelValue"
      :disabled="disabled"
      @change="onChange"
    >
    <span
      class="relative mt-px h-6 w-11 shrink-0 rounded-full border transition-all duration-200 ease-spring peer-focus-visible:ring-2 peer-focus-visible:ring-accent/70 peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-ink-900 sm:h-5 sm:w-9"
      :class="modelValue ? 'border-accent bg-accent' : 'clay-inset border-line bg-ink-900'"
    >
      <span
        class="absolute left-[2px] top-[2px] h-[18px] w-[18px] rounded-full transition-transform duration-200 ease-spring sm:h-[14px] sm:w-[14px]"
        :class="modelValue ? 'translate-x-5 bg-ink-900 sm:translate-x-4' : 'bg-dim'"
      />
    </span>
  </label>
</template>
