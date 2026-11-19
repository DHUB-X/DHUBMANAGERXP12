<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
      @click.self="$emit('close')"
    >
      <div
        class="w-full max-w-2xl max-h-[90vh] flex flex-col bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700/60 rounded-2xl shadow-2xl overflow-hidden text-gray-900 dark:text-gray-100"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-5 py-4 border-b border-gray-200 dark:border-gray-800">
          <div class="min-w-0">
            <h2 class="text-base font-bold truncate">{{ $t('engineMod.title') }}</h2>
            <p class="text-xs text-gray-500 dark:text-gray-400 truncate mt-0.5">
              {{ aircraftName }} ({{ aircraftFolder }})
            </p>
          </div>
          <button
            type="button"
            class="p-1.5 rounded-lg text-gray-400 hover:text-gray-700 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800"
            :disabled="applying"
            @click="$emit('close')"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-y-auto p-5 space-y-4">
          <!-- Step 1: Pick Mod Source (Folder or Archive) -->
          <div class="space-y-2">
            <label class="text-xs font-semibold text-gray-700 dark:text-gray-300 block">
              {{ $t('engineMod.sourceLabel') }}
            </label>
            <div class="flex flex-wrap gap-2">
              <button
                type="button"
                class="px-3 py-1.5 rounded-lg text-xs font-medium bg-indigo-600 hover:bg-indigo-700 text-white transition-colors disabled:opacity-50"
                :disabled="inspecting || applying"
                @click="pickSource(true)"
              >
                {{ $t('engineMod.pickFolder') }}
              </button>
              <button
                type="button"
                class="px-3 py-1.5 rounded-lg text-xs font-medium bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-800 dark:text-gray-200 border border-gray-200 dark:border-gray-700 transition-colors disabled:opacity-50"
                :disabled="inspecting || applying"
                @click="pickSource(false)"
              >
                {{ $t('engineMod.pickArchive') }}
              </button>
            </div>
            <p v-if="sourcePath" class="text-[11px] font-mono text-gray-500 dark:text-gray-400 break-all bg-gray-50 dark:bg-gray-800/60 px-2.5 py-1.5 rounded-lg border border-gray-200 dark:border-gray-700/50">
              {{ sourcePath }}
            </p>
            <p v-else class="text-xs text-gray-500 dark:text-gray-400">
              {{ $t('engineMod.sourceHint') }}
            </p>
          </div>

          <div v-if="inspecting" class="text-xs text-indigo-600 dark:text-indigo-400 flex items-center gap-2">
            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
            </svg>
            {{ $t('engineMod.inspecting') }}
          </div>

          <div
            v-if="errorMsg"
            class="p-3 rounded-xl bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/30 text-xs text-red-700 dark:text-red-300"
          >
            {{ errorMsg }}
          </div>

          <!-- Inspection Details -->
          <template v-if="inspection && !inspecting">
            <!-- Preset selection -->
            <div class="p-3.5 rounded-xl bg-indigo-50/60 dark:bg-indigo-500/10 border border-indigo-200 dark:border-indigo-500/30 space-y-2">
              <div class="flex items-center justify-between gap-2 flex-wrap">
                <span class="text-xs font-semibold text-indigo-900 dark:text-indigo-200">
                  {{ $t('engineMod.presetLabel') }}
                </span>
                <select
                  v-model="selectedPreset"
                  :disabled="applying"
                  class="rounded-lg border border-indigo-300 dark:border-indigo-500/40 bg-white dark:bg-gray-900 px-2.5 py-1 text-xs font-medium text-gray-800 dark:text-gray-100"
                >
                  <option value="ff767_mango_pw4000">Mango Studios FF 767 PW4000 Mod</option>
                  <option value="ff767_mango_ge_cf6">Mango Studios FF 767 GE CF6 Mod</option>
                </select>
              </div>
              <div class="text-[11px] text-indigo-800 dark:text-indigo-200/90 space-y-0.5">
                <p>
                  <strong>{{ $t('engineMod.objectsAdded') }}:</strong>
                  {{ presetObjectsPreview.join(', ') }}
                </p>
                <p>
                  <strong>{{ $t('engineMod.hideDataref') }}:</strong>
                  <code class="font-mono">{{ presetHideDataref }}</code>
                  · <strong>{{ $t('engineMod.clearsDefault') }}:</strong>
                  <code class="font-mono">{{ selectedPreset === 'ff767_mango_pw4000' ? 'PW' : 'GE' }}</code>
                </p>
              </div>
            </div>

            <!-- ACF Variants List -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <h3 class="text-xs font-semibold text-gray-700 dark:text-gray-300">
                  {{ $t('engineMod.acfVariantsTitle') }} ({{ selectedAcfFiles.length }}/{{ inspection.acfVariants.length }})
                </h3>
              </div>
              <div v-if="inspection.acfVariants.length === 0" class="text-xs text-amber-600 dark:text-amber-400">
                {{ $t('engineMod.noAcfFound') }}
              </div>
              <div v-else class="space-y-1.5 max-h-48 overflow-y-auto pr-1">
                <label
                  v-for="v in inspection.acfVariants"
                  :key="v.fileName"
                  class="flex items-center justify-between gap-3 p-2.5 rounded-xl border text-xs cursor-pointer transition-colors"
                  :class="
                    isVariantSupported(v.variant)
                      ? 'bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700/60 hover:border-indigo-400'
                      : 'bg-gray-50/40 dark:bg-gray-900/40 border-gray-200/50 dark:border-gray-800 opacity-55 cursor-not-allowed'
                  "
                >
                  <div class="flex items-center gap-2.5 min-w-0">
                    <input
                      v-model="selectedAcfFiles"
                      type="checkbox"
                      :value="v.fileName"
                      :disabled="!isVariantSupported(v.variant) || applying"
                      class="w-4 h-4 accent-indigo-600"
                    />
                    <div class="min-w-0">
                      <div class="font-mono font-medium truncate">{{ v.fileName }}</div>
                      <div class="text-[11px] text-gray-500 dark:text-gray-400">
                        {{ variantLabel(v.variant) }}
                        <span v-if="coordsForVariant(v.variant)">
                          · ({{ coordsForVariant(v.variant) }})
                        </span>
                      </div>
                    </div>
                  </div>
                  <span
                    v-if="v.hasDefaultEngineObj"
                    class="px-2 py-0.5 rounded-md text-[10px] font-medium bg-emerald-100 dark:bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 shrink-0"
                  >
                    {{ $t('engineMod.defaultEngineDetected') }}
                  </span>
                </label>
              </div>
            </div>

            <!-- Options -->
            <div class="space-y-2 pt-1 border-t border-gray-200 dark:border-gray-800">
              <label v-if="inspection.liveriesSubpath" class="flex items-center gap-2 cursor-pointer text-xs">
                <input v-model="installLiveries" type="checkbox" :disabled="applying" class="w-4 h-4 accent-indigo-600" />
                <span>{{ $t('engineMod.installLiveries') }}</span>
              </label>
              <label v-if="inspection.compatibleObjectsSubpath" class="flex items-center gap-2 cursor-pointer text-xs">
                <input v-model="installCompatibleObjects" type="checkbox" :disabled="applying" class="w-4 h-4 accent-indigo-600" />
                <span>
                  {{ $t('engineMod.installCompatibleObjects', { name: inspection.compatibleObjectsLabel || 'Compatible Objects' }) }}
                </span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer text-xs">
                <input v-model="backupEnabled" type="checkbox" :disabled="applying" class="w-4 h-4 accent-indigo-600" />
                <span class="font-medium">{{ $t('engineMod.backupLabel') }}</span>
              </label>
              <p v-if="backupEnabled" class="text-[11px] text-gray-500 dark:text-gray-400 ml-6">
                {{ $t('engineMod.backupHint') }}
              </p>
            </div>
          </template>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-end gap-2 px-5 py-3.5 border-t border-gray-200 dark:border-gray-800">
          <button
            type="button"
            class="px-3.5 py-2 rounded-xl text-xs font-medium bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200"
            :disabled="applying"
            @click="$emit('close')"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            type="button"
            class="px-4 py-2 rounded-xl text-xs font-semibold bg-indigo-600 hover:bg-indigo-700 text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
            :disabled="!canApply"
            @click="handleApply"
          >
            <svg v-if="applying" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
            </svg>
            <span>{{ applying ? $t('engineMod.applying') : $t('engineMod.applyBtn') }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openFileDialog } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'
import { getErrorMessage } from '@/types'

type EngineModPresetId = 'ff767_mango_pw4000' | 'ff767_mango_ge_cf6'
type AircraftVariantKind = 'v200_er_sf' | 'v300_er_f' | 'v400_er' | 'unknown'

interface DetectedAcfVariant {
  fileName: string
  variant: AircraftVariantKind
  supportedByPreset: boolean
  hasDefaultEngineObj: boolean
  defaultEngineObjName: string | null
  coordinates: [number, number, number] | null
}

interface EngineModInspection {
  sourcePath: string
  isArchive: boolean
  detectedPreset: EngineModPresetId | null
  presetName: string | null
  mainModObjectsSubpath: string | null
  liveriesSubpath: string | null
  compatibleObjectsSubpath: string | null
  compatibleObjectsLabel: string | null
  acfVariants: DetectedAcfVariant[]
  objectsToAdd: string[]
  clearEngineKeyword: string | null
  hideDataref: string | null
}

export interface EngineModApplyResult {
  copiedFiles: number
  patchedAcfFiles: string[]
  backupSessionId: string | null
}

const props = defineProps<{
  xplanePath: string
  aircraftFolder: string
  aircraftName: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'applied', result: EngineModApplyResult): void
}>()

const { t } = useI18n()

const sourcePath = ref('')
const inspecting = ref(false)
const applying = ref(false)
const errorMsg = ref('')
const inspection = ref<EngineModInspection | null>(null)
const selectedPreset = ref<EngineModPresetId>('ff767_mango_pw4000')
const selectedAcfFiles = ref<string[]>([])
const installLiveries = ref(false)
const installCompatibleObjects = ref(false)
const backupEnabled = ref(true)

const presetObjectsPreview = computed(() =>
  selectedPreset.value === 'ff767_mango_pw4000'
    ? ['PW4000L.obj', 'PW4000R.obj', 'N1_fast.obj']
    : ['CF6.obj', 'Misc.obj', 'N1.obj'],
)

const presetHideDataref = computed(() =>
  selectedPreset.value === 'ff767_mango_pw4000'
    ? '1-sim/object/kill/PW'
    : '1-sim/object/kill/GE',
)

function isVariantSupported(variant: AircraftVariantKind): boolean {
  if (variant === 'v200_er_sf' || variant === 'v300_er_f') return true
  if (variant === 'v400_er') return selectedPreset.value === 'ff767_mango_ge_cf6'
  return false
}

function coordsForVariant(variant: AircraftVariantKind): string {
  if (variant === 'v200_er_sf') return '188.97 / 0.00 / 26.20'
  if (variant === 'v300_er_f') return '199.50 / 0.00 / 26.20'
  if (variant === 'v400_er' && selectedPreset.value === 'ff767_mango_ge_cf6') {
    return '210.89 / 0.00 / 26.21'
  }
  return ''
}

function variantLabel(variant: AircraftVariantKind): string {
  switch (variant) {
    case 'v200_er_sf':
      return '767-200ER / SF'
    case 'v300_er_f':
      return '767-300ER / F'
    case 'v400_er':
      return '767-400ER'
    default:
      return t('engineMod.unknownVariant')
  }
}

watch(selectedPreset, () => {
  if (!inspection.value) return
  selectedAcfFiles.value = inspection.value.acfVariants
    .filter((v) => isVariantSupported(v.variant))
    .map((v) => v.fileName)
})

const canApply = computed(
  () =>
    !inspecting.value &&
    !applying.value &&
    Boolean(inspection.value?.mainModObjectsSubpath) &&
    selectedAcfFiles.value.length > 0,
)

async function pickSource(directory: boolean) {
  const selected = await openFileDialog({
    multiple: false,
    directory,
    filters: directory ? undefined : [{ name: 'Archives', extensions: ['zip', '7z', 'rar'] }],
  })
  const picked = Array.isArray(selected) ? selected[0] : selected
  if (!picked) return
  sourcePath.value = picked
  errorMsg.value = ''
  inspecting.value = true
  try {
    const res = await invoke<EngineModInspection>('inspect_engine_mod', {
      sourcePath: picked,
      xplanePath: props.xplanePath,
      aircraftFolder: props.aircraftFolder,
    })
    inspection.value = res
    if (res.detectedPreset) {
      selectedPreset.value = res.detectedPreset
    }
    selectedAcfFiles.value = res.acfVariants
      .filter((v) => isVariantSupported(v.variant))
      .map((v) => v.fileName)
    installLiveries.value = false
    installCompatibleObjects.value = false
    if (!res.mainModObjectsSubpath) {
      errorMsg.value = t('engineMod.noObjectsFound')
    }
  } catch (e) {
    inspection.value = null
    errorMsg.value = getErrorMessage(e)
  } finally {
    inspecting.value = false
  }
}

async function handleApply() {
  if (!canApply.value || !inspection.value) return
  applying.value = true
  errorMsg.value = ''
  try {
    const result = await invoke<EngineModApplyResult>('apply_engine_mod', {
      request: {
        sourcePath: sourcePath.value,
        xplanePath: props.xplanePath,
        aircraftFolder: props.aircraftFolder,
        presetId: selectedPreset.value,
        selectedAcfFiles: selectedAcfFiles.value,
        installLiveries: installLiveries.value,
        installCompatibleObjects: installCompatibleObjects.value,
        backupEnabled: backupEnabled.value,
      },
    })
    emit('applied', result)
  } catch (e) {
    errorMsg.value = getErrorMessage(e)
  } finally {
    applying.value = false
  }
}
</script>
