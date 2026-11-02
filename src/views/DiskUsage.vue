<template>
  <div class="h-full flex flex-col px-6 pt-4 pb-6 overflow-y-auto select-none">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-6">
      <div>
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-xl bg-blue-500/10 dark:bg-blue-500/20 flex items-center justify-center text-blue-500">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
            </svg>
          </div>
          <h1 class="text-xl font-bold text-gray-900 dark:text-white tracking-tight">
            {{ $t('diskUsage.title') }}
          </h1>
        </div>
        <p v-if="store.report" class="text-xs text-gray-500 dark:text-gray-400 mt-1 pl-10.5">
          {{ $t('diskUsage.scannedIn', { ms: store.report.scanDurationMs }) }}
        </p>
      </div>

      <div class="flex items-center gap-2.5 self-start sm:self-auto">
        <router-link
          to="/disk-usage/output-cleanup"
          class="text-xs sm:text-sm px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-800 bg-white/70 dark:bg-slate-900/60 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 hover:border-slate-300 dark:hover:border-slate-700 transition-all shadow-xs flex items-center gap-1.5 font-medium"
        >
          <svg class="w-3.5 h-3.5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          {{ $t('diskUsage.outputCleanup.title') }}
        </router-link>

        <button
          class="text-xs sm:text-sm px-4.5 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 text-white font-semibold shadow-xs shadow-blue-500/20 active:scale-95 transition-all disabled:opacity-50 flex items-center gap-2"
          :disabled="store.isScanning"
          @click="scanAll"
        >
          <div
            v-if="store.isScanning"
            class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"
          ></div>
          <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ store.isScanning ? $t('diskUsage.scanning') : $t('diskUsage.scan') }}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div v-if="store.report" class="flex-1 flex flex-col gap-6">
      <!-- Modern Hero Dashboard Card (Donut Chart + Category Metric Grid) -->
      <div
        class="rounded-2xl border border-slate-200 dark:border-slate-800/80 bg-white/70 dark:bg-slate-900/60 backdrop-blur-md p-6 shadow-sm dark:shadow-slate-950/40"
      >
        <div class="flex flex-col lg:flex-row items-center gap-8">
          <!-- Left: Donut Chart -->
          <div class="flex flex-col items-center justify-center shrink-0">
            <DiskUsageChart
              :categories="chartData"
              :total-bytes="store.report.totalBytes"
              :size="170"
              :is-dark="isDark"
            />
          </div>

          <!-- Right: Category Metrics Grid -->
          <div class="flex-1 w-full grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3">
            <div
              v-for="cat in store.report.categories"
              :key="cat.category"
              class="rounded-xl border border-slate-200/70 dark:border-slate-800/60 bg-slate-50/70 dark:bg-slate-800/30 p-3.5 flex flex-col justify-between hover:border-slate-300 dark:hover:border-slate-700/80 transition-all duration-200"
            >
              <div class="flex items-center justify-between gap-2 mb-2.5">
                <div class="flex items-center gap-2 min-w-0">
                  <div
                    class="w-2.5 h-2.5 rounded-full shrink-0 shadow-xs"
                    :style="{ backgroundColor: categoryColor(cat.category) }"
                  ></div>
                  <span class="text-xs font-semibold text-slate-800 dark:text-slate-200 truncate">
                    {{ diskCategoryLabel(cat.category) }}
                  </span>
                  <span class="text-[10px] text-slate-400 dark:text-slate-400 bg-slate-200/60 dark:bg-slate-700/50 px-1.5 py-0.5 rounded-md font-mono">
                    {{ cat.itemCount }}
                  </span>
                </div>
                <span class="text-xs font-bold text-slate-900 dark:text-white tabular-nums">
                  {{ formatSize(cat.totalBytes) }}
                </span>
              </div>

              <!-- Mini Progress Bar & Percentage -->
              <div class="flex items-center gap-2.5">
                <div class="flex-1 h-1.5 bg-slate-200/80 dark:bg-slate-700/60 rounded-full overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-500"
                    :style="{
                      width: store.report.totalBytes > 0 ? `${(cat.totalBytes / store.report.totalBytes) * 100}%` : '0%',
                      backgroundColor: categoryColor(cat.category),
                    }"
                  ></div>
                </div>
                <span class="text-[10px] font-semibold text-slate-500 dark:text-slate-400 tabular-nums shrink-0">
                  {{ store.report.totalBytes > 0 ? ((cat.totalBytes / store.report.totalBytes) * 100).toFixed(1) : 0 }}%
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modern Category Details Accordion -->
      <div class="space-y-3">
        <details
          v-for="cat in store.report.categories"
          :key="cat.category"
          class="group rounded-xl border border-slate-200 dark:border-slate-800/80 bg-white/70 dark:bg-slate-900/60 overflow-hidden shadow-xs transition-all duration-200 hover:border-slate-300 dark:hover:border-slate-700/80"
        >
          <summary
            class="flex items-center justify-between px-5 py-3.5 cursor-pointer hover:bg-slate-50/80 dark:hover:bg-slate-800/40 transition-colors list-none select-none"
          >
            <div class="flex items-center gap-3 min-w-0">
              <div
                class="w-3 h-3 rounded-md shrink-0 shadow-xs"
                :style="{ backgroundColor: categoryColor(cat.category) }"
              ></div>
              <span class="font-semibold text-sm text-slate-800 dark:text-slate-100">
                {{ diskCategoryLabel(cat.category) }}
              </span>
              <span class="text-xs text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-full border border-slate-200/60 dark:border-slate-700/40 font-mono">
                {{ cat.itemCount }} {{ $t('diskUsage.items') }}
              </span>
            </div>

            <div class="flex items-center gap-4">
              <div class="flex items-center gap-2.5">
                <span class="text-xs font-semibold text-slate-400 dark:text-slate-400 tabular-nums">
                  {{ store.report.totalBytes > 0 ? ((cat.totalBytes / store.report.totalBytes) * 100).toFixed(1) : 0 }}%
                </span>
                <span class="text-sm font-bold text-slate-900 dark:text-white font-mono">
                  {{ formatSize(cat.totalBytes) }}
                </span>
              </div>

              <svg
                class="w-4 h-4 text-slate-400 transition-transform duration-200 group-open:rotate-180"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
              </svg>
            </div>
          </summary>

          <div class="border-t border-slate-100 dark:border-slate-800/80 divide-y divide-slate-100 dark:divide-slate-800/40 bg-slate-50/40 dark:bg-slate-950/20 max-h-96 overflow-y-auto">
            <div
              v-for="item in cat.items"
              :key="item.folderName"
              class="flex items-center gap-3 px-5 py-2.5 hover:bg-slate-100/60 dark:hover:bg-slate-800/30 transition-colors"
            >
              <div class="flex-1 min-w-0">
                <p class="text-xs font-medium text-slate-800 dark:text-slate-200 truncate">
                  {{ item.displayName }}
                </p>
                <div class="flex items-center gap-2.5 mt-1">
                  <div
                    class="flex-1 max-w-xs h-1.5 bg-slate-200/70 dark:bg-slate-800 rounded-full overflow-hidden"
                  >
                    <div
                      class="h-full rounded-full transition-all duration-300"
                      :style="{
                        width:
                          cat.totalBytes > 0 ? `${(item.sizeBytes / cat.totalBytes) * 100}%` : '0%',
                        backgroundColor: categoryColor(cat.category),
                      }"
                    ></div>
                  </div>
                  <span class="text-[11px] text-slate-500 dark:text-slate-400 font-mono shrink-0">
                    {{ formatSize(item.sizeBytes) }}
                  </span>
                </div>
              </div>
              <button
                v-if="item.folderName"
                class="p-1.5 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-200/60 dark:hover:bg-slate-800 transition-colors"
                :title="$t('diskUsage.openFolder')"
                @click="openFolder(item.itemType, item.folderName)"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                  />
                </svg>
              </button>
            </div>
          </div>
        </details>
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="!store.isScanning"
      class="flex-1 flex flex-col items-center justify-center text-slate-400 dark:text-slate-500 py-16"
    >
      <div class="w-16 h-16 rounded-2xl bg-slate-100 dark:bg-slate-800/50 flex items-center justify-center mb-3 text-slate-400">
        <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
          />
        </svg>
      </div>
      <p class="text-sm font-semibold text-slate-700 dark:text-slate-300">{{ $t('diskUsage.empty') }}</p>
      <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">{{ $t('diskUsage.emptyHint') }}</p>
    </div>

    <!-- Scanning overlay -->
    <div
      v-if="store.isScanning && !store.report"
      class="flex-1 flex flex-col items-center justify-center py-16"
    >
      <div
        class="w-9 h-9 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mb-3.5"
      ></div>
      <p class="text-sm font-medium text-slate-600 dark:text-slate-300">{{ $t('diskUsage.scanning') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useDiskUsageStore } from '@/stores/diskUsage'
import { useAppStore } from '@/stores/app'
import { useThemeStore } from '@/stores/theme'
import DiskUsageChart from '@/components/DiskUsageChart.vue'

const { t } = useI18n()

const store = useDiskUsageStore()
const appStore = useAppStore()
const themeStore = useThemeStore()

const isDark = computed(() => themeStore.isDark)

const COLORS: Record<string, string> = {
  Aircraft: '#3b82f6',
  Plugins: '#8b5cf6',
  Scenery: '#10b981',
  Navdata: '#f59e0b',
  Screenshots: '#ef4444',
}

function categoryColor(name: string): string {
  return COLORS[name] || '#6b7280'
}

const DISK_CATEGORY_KEY_MAP: Record<string, string> = {
  aircraft: 'diskUsage.categoryAircraft',
  plugin: 'diskUsage.categoryPlugins',
  plugins: 'diskUsage.categoryPlugins',
  scenery: 'diskUsage.categoryScenery',
  navdata: 'diskUsage.categoryNavdata',
  screenshot: 'diskUsage.categoryScreenshots',
  screenshots: 'diskUsage.categoryScreenshots',
}

function diskCategoryLabel(category: string): string {
  const key = DISK_CATEGORY_KEY_MAP[category.trim().toLowerCase()]
  return key ? t(key) : category
}

const chartData = computed(() => {
  if (!store.report) return []
  return store.report.categories.map((c) => ({
    name: diskCategoryLabel(c.category),
    bytes: c.totalBytes,
    color: categoryColor(c.category),
  }))
})

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`
  return `${(bytes / 1073741824).toFixed(2)} GB`
}

async function scanAll() {
  if (!appStore.xplanePath) return
  await store.scan()
}

async function openFolder(itemType: string, folderName: string) {
  try {
    await invoke('open_management_folder', {
      xplanePath: appStore.xplanePath,
      itemType,
      folderName,
    })
  } catch {
    // silently ignore
  }
}

onMounted(() => {
  if (!appStore.xplanePath || store.isScanning) return
  void scanAll()
})
</script>
