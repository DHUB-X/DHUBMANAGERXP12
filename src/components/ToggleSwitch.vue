<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue: boolean
    disabled?: boolean
    ariaLabel?: string
    size?: 'sm' | 'md' | 'lg'
    activeClass?: string
    inactiveClass?: string
    stopPropagation?: boolean
  }>(),
  {
    disabled: false,
    ariaLabel: '',
    size: 'md',
    activeClass: 'bg-blue-600 dark:bg-blue-600',
    inactiveClass: 'bg-slate-700/80 dark:bg-slate-800',
    stopPropagation: true,
  },
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const sizeConfig = computed(() => {
  switch (props.size) {
    case 'sm':
      return {
        track: 'h-5 w-10 p-0.5 rounded-[5px]',
        knob: 'h-4 w-4 rounded-[3px]',
        on: 'translate-x-5',
        iconSlot: 'w-5 h-4',
        iconSvg: 'w-2.5 h-2.5',
      }
    case 'lg':
      return {
        track: 'h-7 w-14 p-0.5 rounded-[7px]',
        knob: 'h-6 w-6 rounded-[5px]',
        on: 'translate-x-7',
        iconSlot: 'w-7 h-6',
        iconSvg: 'w-3.5 h-3.5',
      }
    default:
      return {
        track: 'h-6 w-12 p-0.5 rounded-[6px]',
        knob: 'h-5 w-5 rounded-[4px]',
        on: 'translate-x-6',
        iconSlot: 'w-6 h-5',
        iconSvg: 'w-3 h-3',
      }
  }
})

function handleToggle(event: MouseEvent) {
  if (props.stopPropagation) {
    event.stopPropagation()
  }
  if (props.disabled) return
  emit('update:modelValue', !props.modelValue)
}
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="modelValue"
    :aria-label="ariaLabel || undefined"
    class="relative inline-flex shrink-0 items-center select-none overflow-hidden transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-400/50 shadow-[inset_0_2px_4px_rgba(0,0,0,0.35)]"
    :class="[sizeConfig.track, modelValue ? activeClass : inactiveClass]"
    :disabled="disabled"
    @click="handleToggle"
  >
    <!-- Left Icon (Checkmark ✓ visible when ON) -->
    <span
      class="absolute left-0.5 top-0.5 flex items-center justify-center transition-all duration-200 pointer-events-none"
      :class="[
        sizeConfig.iconSlot,
        modelValue ? 'opacity-100 scale-100' : 'opacity-0 scale-75',
      ]"
    >
      <svg
        :class="sizeConfig.iconSvg"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="2.6"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="text-white/95 drop-shadow-[0_1px_1px_rgba(0,0,0,0.3)]"
      >
        <path d="M3.5 8.5L6.5 11.5L12.5 4.5" />
      </svg>
    </span>

    <!-- Right Icon (Cross ✕ visible when OFF) -->
    <span
      class="absolute right-0.5 top-0.5 flex items-center justify-center transition-all duration-200 pointer-events-none"
      :class="[
        sizeConfig.iconSlot,
        !modelValue ? 'opacity-100 scale-100' : 'opacity-0 scale-75',
      ]"
    >
      <svg
        :class="sizeConfig.iconSvg"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="text-rose-300 dark:text-rose-300/90 drop-shadow-[0_1px_1px_rgba(0,0,0,0.3)]"
      >
        <path d="M4.5 4.5L11.5 11.5M11.5 4.5L4.5 11.5" />
      </svg>
    </span>

    <!-- 3D Squarish Knob (matching reference image) -->
    <span
      class="relative z-10 bg-gradient-to-b from-white via-slate-100 to-slate-200 border border-white/90 shadow-[0_2px_4px_rgba(0,0,0,0.28),inset_0_1px_0_#FFFFFF] transition-transform duration-200 ease-out"
      :class="[sizeConfig.knob, modelValue ? sizeConfig.on : 'translate-x-0']"
    />
  </button>
</template>
