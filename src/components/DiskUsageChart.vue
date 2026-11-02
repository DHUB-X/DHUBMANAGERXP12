<template>
  <div class="relative shrink-0 flex items-center justify-center" :style="{ width: size + 'px', height: size + 'px' }">
    <svg :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`" class="rotate-[-90deg]">
      <!-- Background circle -->
      <circle
        :cx="center"
        :cy="center"
        :r="radius"
        fill="none"
        :stroke="isDark ? '#1e293b' : '#f1f5f9'"
        :stroke-width="strokeWidth"
      />
      <!-- Segments -->
      <circle
        v-for="(seg, i) in segments"
        :key="i"
        :cx="center"
        :cy="center"
        :r="radius"
        fill="none"
        :stroke="seg.color"
        :stroke-width="strokeWidth"
        :stroke-dasharray="`${seg.length} ${circumference - seg.length}`"
        :stroke-dashoffset="-(seg.offset || 0)"
        stroke-linecap="round"
        class="transition-all duration-500 ease-out cursor-pointer hover:opacity-80"
        @mouseenter="hovered = i"
        @mouseleave="hovered = -1"
      />
    </svg>
    <!-- Center text -->
    <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none select-none text-center px-2">
      <span
        class="font-extrabold text-slate-900 dark:text-white tracking-tight leading-tight transition-all duration-200"
        :class="size < 160 ? 'text-base' : 'text-xl'"
      >
        {{ hovered >= 0 ? formatSize(categories[hovered].bytes) : formatSize(totalBytes) }}
      </span>
      <span class="text-[10px] font-semibold text-slate-400 dark:text-slate-400 uppercase tracking-wider mt-0.5">
        {{ hovered >= 0 ? categories[hovered].name : $t('diskUsage.total') }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

useI18n()

const props = defineProps<{
  categories: { name: string; bytes: number; color: string }[]
  totalBytes: number
  size?: number
  isDark?: boolean
}>()

const hovered = ref(-1)
const size = computed(() => props.size ?? 160)
const strokeWidth = computed(() => (size.value < 150 ? 10 : 14))
const center = computed(() => size.value / 2)
const radius = computed(() => (size.value - strokeWidth.value) / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)

interface Segment {
  color: string
  length: number
  offset: number
}

const segments = computed<Segment[]>(() => {
  if (props.totalBytes === 0) return []
  const segs: Segment[] = []
  let offset = 0
  for (const cat of props.categories) {
    const ratio = cat.bytes / props.totalBytes
    const length = ratio * circumference.value
    segs.push({ color: cat.color, length, offset })
    offset += length
  }
  return segs
})

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`
  return `${(bytes / 1073741824).toFixed(1)} GB`
}
</script>
