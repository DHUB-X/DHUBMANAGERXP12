<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="show" class="fixed inset-0 z-[1200] flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div
          class="absolute inset-0 bg-black/50 backdrop-blur-sm"
          @click="handleClose"
        ></div>

        <!-- Modal Box -->
        <div
          class="relative w-full max-w-lg bg-white dark:bg-gray-800 rounded-xl shadow-2xl border border-gray-200 dark:border-gray-700 p-6 flex flex-col"
          @click.stop
        >
          <!-- Header -->
          <div class="flex items-start justify-between gap-4 mb-4">
            <div class="min-w-0">
              <h2 class="text-base font-semibold text-gray-900 dark:text-white">
                {{ t('multiDrive.title') }}
              </h2>
              <p class="mt-0.5 text-xs text-gray-500 dark:text-gray-400 font-mono truncate" :title="folderName">
                {{ folderName }}
              </p>
            </div>
            <button
              type="button"
              @click="handleClose"
              :disabled="isProcessing"
              class="flex-shrink-0 p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-40"
              :title="t('common.close')"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Loading state -->
          <div v-if="isLoading" class="py-10 flex flex-col items-center justify-center space-y-2">
            <div class="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
            <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('common.loading') }}</p>
          </div>

          <!-- Error state -->
          <div v-else-if="errorMessage" class="mb-4 p-3 rounded-lg bg-red-50 dark:bg-red-950/40 border border-red-200 dark:border-red-900/50 text-xs text-red-600 dark:text-red-300">
            {{ errorMessage }}
          </div>

          <!-- Content Body -->
          <div v-if="!isLoading && storageInfo" class="space-y-4">
            <!-- Case 1: ALREADY LINKED -->
            <template v-if="storageInfo.isLinked">
              <!-- Meta summary -->
              <div class="rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50/80 dark:bg-gray-900/40 divide-y divide-gray-200 dark:divide-gray-700/60 text-xs">
                <div class="px-3.5 py-2.5 flex items-center justify-between">
                  <span class="text-gray-500 dark:text-gray-400">{{ t('multiDrive.storageStatus') }}</span>
                  <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                    {{ t('multiDrive.linkedStatus') }}
                  </span>
                </div>
                <div class="px-3.5 py-2.5 flex items-center justify-between">
                  <span class="text-gray-500 dark:text-gray-400">{{ t('multiDrive.folderSize') }}</span>
                  <span class="font-medium text-gray-900 dark:text-gray-100">
                    {{ storageInfo.sizeFormatted }}
                    <span class="text-emerald-600 dark:text-emerald-400 ml-1">({{ t('multiDrive.savedOnSSD', { size: storageInfo.sizeFormatted }) }})</span>
                  </span>
                </div>
                <div class="px-3.5 py-2.5 space-y-1.5">
                  <div class="flex items-center justify-between">
                    <span class="text-gray-500 dark:text-gray-400">{{ t('multiDrive.targetPath') }}</span>
                    <button
                      type="button"
                      @click="handleOpenTargetFolder"
                      class="inline-flex items-center gap-1 text-[11px] font-medium text-blue-600 dark:text-blue-400 hover:underline"
                    >
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                      </svg>
                      {{ t('multiDrive.openTargetFolder') }}
                    </button>
                  </div>
                  <div class="font-mono text-[11px] text-gray-700 dark:text-gray-300 break-all bg-white dark:bg-gray-800 px-2.5 py-1.5 rounded border border-gray-200 dark:border-gray-700 select-all">
                    {{ storageInfo.linkTarget }}
                  </div>
                </div>
              </div>

              <!-- Action buttons -->
              <div class="flex items-center gap-2 pt-1">
                <button
                  type="button"
                  :disabled="isProcessing"
                  @click="handleRestoreToSSD"
                  class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center justify-center gap-1.5 disabled:opacity-50"
                >
                  <svg class="w-3.5 h-3.5 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                  </svg>
                  <span>{{ t('multiDrive.restoreToSSD') }}</span>
                </button>

                <button
                  type="button"
                  :disabled="isProcessing"
                  @click="handleUnlinkOnly"
                  class="px-3 py-2 text-xs font-medium rounded-lg border border-red-200 dark:border-red-900/50 bg-red-50/60 dark:bg-red-950/30 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-900/40 transition-colors flex items-center justify-center gap-1.5 disabled:opacity-50"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                  <span>{{ t('multiDrive.unlinkOnly') }}</span>
                </button>
              </div>

              <!-- Note -->
              <p class="text-[11px] text-gray-500 dark:text-gray-400 leading-relaxed">
                {{ t('multiDrive.alreadyLinkedNote') }}
              </p>
            </template>

            <!-- Case 2: NOT LINKED (Move to Secondary Drive) -->
            <template v-else>
              <!-- Info summary -->
              <div class="flex items-center justify-between text-xs px-3.5 py-2 rounded-lg bg-gray-50/80 dark:bg-gray-900/40 border border-gray-200 dark:border-gray-700">
                <span class="text-gray-500 dark:text-gray-400">{{ t('multiDrive.folderSize') }}: <strong class="text-gray-900 dark:text-gray-100 font-medium">{{ storageInfo.sizeFormatted }}</strong></span>
                <span class="text-gray-500 dark:text-gray-400">{{ t('multiDrive.storageStatus') }}: <strong class="text-gray-900 dark:text-gray-100 font-medium">{{ t('multiDrive.primaryStatus') }}</strong></span>
              </div>

              <!-- Drive Selection -->
              <div>
                <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                  {{ t('multiDrive.selectTargetDrive') }}
                </label>
                <div class="grid grid-cols-2 gap-2">
                  <button
                    v-for="drive in secondaryDrives"
                    :key="drive.letter"
                    type="button"
                    @click="selectedDrive = drive"
                    :class="[
                      'p-2.5 rounded-lg border text-left transition flex items-center justify-between',
                      selectedDrive?.letter === drive.letter
                        ? 'border-blue-500 bg-blue-50/50 dark:bg-blue-950/30 text-gray-900 dark:text-white ring-1 ring-blue-500'
                        : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900/60 hover:border-gray-300 dark:hover:border-gray-600 text-gray-700 dark:text-gray-300'
                    ]"
                  >
                    <div>
                      <div class="text-xs font-semibold font-mono">{{ drive.letter }} {{ drive.label }}</div>
                      <div class="text-[11px] text-gray-500 dark:text-gray-400 mt-0.5">
                        {{ drive.freeFormatted }} free
                      </div>
                    </div>
                    <div
                      v-if="selectedDrive?.letter === drive.letter"
                      class="w-2 h-2 rounded-full bg-blue-600 dark:bg-blue-400"
                    ></div>
                  </button>
                </div>

                <div v-if="secondaryDrives.length === 0" class="p-2.5 bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/40 rounded-lg text-xs text-amber-800 dark:text-amber-300 mt-2">
                  {{ t('multiDrive.noSecondaryDrives') }}
                </div>
              </div>

              <!-- Target Directory & Browse -->
              <div v-if="selectedDrive" class="space-y-1.5">
                <label class="block text-xs font-medium text-gray-700 dark:text-gray-300">
                  {{ t('multiDrive.selectTargetFolder') }}
                </label>
                <div class="flex items-center gap-2">
                  <input
                    v-model="customFolderPath"
                    type="text"
                    :placeholder="defaultStorageFolder"
                    class="flex-1 px-3 py-1.5 text-xs bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-900 dark:text-gray-100 font-mono focus:border-blue-500 focus:outline-none"
                  />
                  <button
                    type="button"
                    @click="handleBrowseFolder"
                    class="px-3 py-1.5 text-xs font-medium bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700 rounded-lg transition"
                  >
                    {{ t('common.browse') }}
                  </button>
                </div>
                <div class="text-[11px] text-gray-500 dark:text-gray-400 font-mono truncate">
                  {{ t('multiDrive.previewDestination') }} <span class="text-gray-700 dark:text-gray-300">{{ finalDestinationPreview }}</span>
                </div>
              </div>

              <!-- Brief hint -->
              <p class="text-[11px] text-gray-500 dark:text-gray-400 leading-relaxed border-t border-gray-100 dark:border-gray-700/60 pt-2.5">
                {{ t('multiDrive.howItWorksDesc') }}
              </p>
            </template>
          </div>

          <!-- Progress overlay -->
          <div
            v-if="isProcessing"
            class="p-6 flex flex-col items-center justify-center space-y-2 bg-gray-50 dark:bg-gray-900/90 rounded-lg border border-gray-200 dark:border-gray-700 my-2"
          >
            <div class="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
            <div class="text-xs font-semibold text-gray-900 dark:text-white">{{ processingTitle }}</div>
            <div class="text-[11px] text-gray-500 dark:text-gray-400 text-center max-w-sm">{{ processingSubtitle }}</div>
          </div>

          <!-- Clean Footer -->
          <div class="flex items-center justify-end gap-2 pt-4 mt-3 border-t border-gray-200 dark:border-gray-700">
            <button
              type="button"
              :disabled="isProcessing"
              @click="handleClose"
              class="px-3.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-40"
            >
              {{ t('common.close') }}
            </button>
            <button
              v-if="!storageInfo?.isLinked"
              type="button"
              :disabled="isProcessing || !effectiveDestinationFolder"
              @click="handleMoveToDrive"
              class="px-4 py-1.5 text-xs font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {{ t('multiDrive.moveAndLinkButton') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { open as openDirectoryDialog } from '@tauri-apps/plugin-dialog'
import { useToastStore } from '@/stores/toast'
import type { DriveInfo, SceneryStorageDetail, MoveSceneryResult, RestoreSceneryResult } from '@/types'

const props = defineProps<{
  show: boolean
  folderName: string
  xplanePath: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'updated'): void
}>()

const { t } = useI18n()
const toastStore = useToastStore()

const isLoading = ref(false)
const isProcessing = ref(false)
const errorMessage = ref<string | null>(null)

const processingTitle = ref('')
const processingSubtitle = ref('')

const storageInfo = ref<SceneryStorageDetail | null>(null)
const availableDrives = ref<DriveInfo[]>([])
const selectedDrive = ref<DriveInfo | null>(null)

const customFolderPath = ref('')

const secondaryDrives = computed(() => {
  return availableDrives.value.filter((d) => !d.isSystemOrGameDrive)
})

const defaultStorageFolder = computed(() => {
  if (!selectedDrive.value) return ''
  const letter = selectedDrive.value.letter
  return `${letter}\\X-Plane_Custom_Scenery_Storage`
})

const effectiveDestinationFolder = computed(() => {
  if (customFolderPath.value.trim()) {
    return customFolderPath.value.trim()
  }
  return defaultStorageFolder.value
})

const finalDestinationPreview = computed(() => {
  if (!effectiveDestinationFolder.value) return ''
  const base = effectiveDestinationFolder.value.replace(/[/\\]+$/, '')
  return `${base}\\${props.folderName}`
})

watch(
  () => props.show,
  async (newVal) => {
    if (newVal && props.folderName && props.xplanePath) {
      await loadStorageData()
    } else {
      errorMessage.value = null
      isProcessing.value = false
    }
  }
)

async function loadStorageData() {
  isLoading.value = true
  errorMessage.value = null
  try {
    const [info, drives] = await Promise.all([
      invoke<SceneryStorageDetail>('get_scenery_storage_info', {
        xplanePath: props.xplanePath,
        folderName: props.folderName
      }),
      invoke<DriveInfo[]>('get_available_drives', {
        xplanePath: props.xplanePath
      })
    ])

    storageInfo.value = info
    availableDrives.value = drives

    const eligible = drives.filter((d) => !d.isSystemOrGameDrive)
    if (eligible.length > 0) {
      selectedDrive.value = eligible[0]
    } else {
      selectedDrive.value = null
    }
    customFolderPath.value = ''
  } catch (err: any) {
    errorMessage.value = typeof err === 'string' ? err : err?.message || 'Error loading scenery status.'
  } finally {
    isLoading.value = false
  }
}

async function handleOpenTargetFolder() {
  try {
    await invoke('open_target_storage_folder', {
      xplanePath: props.xplanePath,
      folderName: props.folderName
    })
  } catch (err: any) {
    toastStore.error(err?.message || 'Failed to open target folder.')
  }
}

async function handleBrowseFolder() {
  try {
    const selected = await openDirectoryDialog({
      directory: true,
      multiple: false,
      title: t('multiDrive.browseFolder')
    })
    if (selected && typeof selected === 'string') {
      customFolderPath.value = selected
    }
  } catch (err) {
    console.error('Directory browse failed', err)
  }
}

async function handleMoveToDrive() {
  const target = effectiveDestinationFolder.value
  if (!target) return

  isProcessing.value = true
  processingTitle.value = t('multiDrive.movingProgress', { folder: props.folderName })
  processingSubtitle.value = t('multiDrive.movingSubtitle')
  errorMessage.value = null

  try {
    const result = await invoke<MoveSceneryResult>('move_scenery_to_drive', {
      xplanePath: props.xplanePath,
      folderName: props.folderName,
      targetDestination: target
    })

    toastStore.success(result.message || 'Scenery moved successfully!')
    emit('updated')
    await loadStorageData()
  } catch (err: any) {
    const msg = typeof err === 'string' ? err : err?.message || 'Failed to move scenery.'
    errorMessage.value = msg
    toastStore.error(msg)
  } finally {
    isProcessing.value = false
  }
}

async function handleRestoreToSSD() {
  isProcessing.value = true
  processingTitle.value = t('multiDrive.restoringProgress', { folder: props.folderName })
  processingSubtitle.value = t('multiDrive.restoringSubtitle')
  errorMessage.value = null

  try {
    const result = await invoke<RestoreSceneryResult>('restore_scenery_to_ssd', {
      xplanePath: props.xplanePath,
      folderName: props.folderName
    })

    toastStore.success(result.message || 'Scenery restored successfully!')
    emit('updated')
    await loadStorageData()
  } catch (err: any) {
    const msg = typeof err === 'string' ? err : err?.message || 'Failed to restore scenery.'
    errorMessage.value = msg
    toastStore.error(msg)
  } finally {
    isProcessing.value = false
  }
}

async function handleUnlinkOnly() {
  if (!confirm(t('multiDrive.confirmUnlink', { folder: props.folderName }))) {
    return
  }

  isProcessing.value = true
  processingTitle.value = t('multiDrive.unlinkingProgress')
  processingSubtitle.value = t('multiDrive.unlinkingSubtitle')
  errorMessage.value = null

  try {
    const msg = await invoke<string>('unlink_scenery_junction', {
      xplanePath: props.xplanePath,
      folderName: props.folderName
    })

    toastStore.info(msg || 'Junction unlinked.')
    emit('updated')
    handleClose()
  } catch (err: any) {
    const msg = typeof err === 'string' ? err : err?.message || 'Failed to unlink.'
    errorMessage.value = msg
    toastStore.error(msg)
  } finally {
    isProcessing.value = false
  }
}

function handleClose() {
  if (!isProcessing.value) {
    emit('close')
  }
}
</script>
