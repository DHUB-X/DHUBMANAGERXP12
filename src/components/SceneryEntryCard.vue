<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app'
import { useModalStore } from '@/stores/modal'
import { useLockStore } from '@/stores/lock'
import { useSceneryStore } from '@/stores/scenery'
import ToggleSwitch from '@/components/ToggleSwitch.vue'
import type { SceneryManagerEntry } from '@/types'
import { SceneryCategory, getErrorMessage, parseApiError } from '@/types'
import { useContextMenu } from '@/composables/useContextMenu'
import type { ContextMenuItem } from '@/composables/useContextMenu'

const GLOBAL_AIRPORTS_ENTRY_NAME = '*GLOBAL_AIRPORTS*'

const props = withDefaults(
  defineProps<{
    entry: SceneryManagerEntry
    index: number
    totalCount: number
    disableReorder?: boolean
    disableMoveDown?: boolean
    flattenBusy?: boolean
    selected?: boolean
    showCheckbox?: boolean
  }>(),
  {
    disableReorder: false,
    disableMoveDown: false,
    flattenBusy: false,
    selected: false,
    showCheckbox: false,
  },
)

const emit = defineEmits<{
  (e: 'toggle-enabled', folderName: string): void
  (e: 'move-up', folderName: string): void
  (e: 'move-down', folderName: string): void
  (e: 'show-missing-libs', entry: SceneryManagerEntry): void
  (e: 'show-duplicate-tiles', entry: SceneryManagerEntry): void
  (e: 'show-delete-confirm', entry: SceneryManagerEntry): void
  (e: 'update', folderName: string): void
  (e: 'toggle-flatten', entry: SceneryManagerEntry): void
  (e: 'open-flatten-page', entry: SceneryManagerEntry): void
  (e: 'toggle-select', folderName: string): void
  (e: 'assign-custom-group', payload: { folderName: string; groupId: string }): void
  (e: 'remove-custom-group', payload: { folderName: string; groupId: string }): void
  (e: 'manage-storage', entry: SceneryManagerEntry): void
}>()

const { t } = useI18n()
const appStore = useAppStore()
const modalStore = useModalStore()
const lockStore = useLockStore()
const sceneryStore = useSceneryStore()
const contextMenu = useContextMenu()

// Category display config
const categoryConfig = computed(() => {
  const cat =
    props.entry.category === SceneryCategory.FixedHighPriority &&
    props.entry.originalCategory &&
    props.entry.originalCategory !== SceneryCategory.FixedHighPriority
      ? props.entry.originalCategory
      : props.entry.category

  switch (cat) {
    case SceneryCategory.FixedHighPriority:
      return {
        label: 'SAM',
        color: 'text-purple-700 dark:text-purple-300',
        bgColor: 'bg-purple-100 dark:bg-purple-900/30',
      }
    case SceneryCategory.Airport:
      return {
        label: t('sceneryManager.categoryAirport'),
        color: 'text-blue-700 dark:text-blue-300',
        bgColor: 'bg-blue-100 dark:bg-blue-900/30',
      }
    case SceneryCategory.DefaultAirport:
      return {
        label: t('sceneryManager.categoryDefaultAirport'),
        color: 'text-gray-600 dark:text-gray-400',
        bgColor: 'bg-gray-100 dark:bg-gray-800/50',
      }
    case SceneryCategory.Library:
      return {
        label: t('sceneryManager.categoryLibrary'),
        color: 'text-green-700 dark:text-green-300',
        bgColor: 'bg-green-100 dark:bg-green-900/30',
      }
    case SceneryCategory.Overlay:
      return {
        label: t('sceneryManager.categoryOverlay'),
        color: 'text-yellow-700 dark:text-yellow-300',
        bgColor: 'bg-yellow-100 dark:bg-yellow-900/30',
      }
    case SceneryCategory.AirportMesh:
      return {
        label: t('sceneryManager.categoryAirportMesh'),
        color: 'text-cyan-700 dark:text-cyan-300',
        bgColor: 'bg-cyan-100 dark:bg-cyan-900/30',
      }
    case SceneryCategory.Mesh:
      return {
        label: t('sceneryManager.categoryMesh'),
        color: 'text-red-700 dark:text-red-300',
        bgColor: 'bg-red-100 dark:bg-red-900/30',
      }
    case SceneryCategory.Other:
      return {
        label: t('sceneryManager.categoryOther'),
        color: 'text-gray-600 dark:text-gray-400',
        bgColor: 'bg-gray-100 dark:bg-gray-800/50',
      }
    case SceneryCategory.Unrecognized:
      return {
        label: t('sceneryManager.categoryUnrecognized'),
        color: 'text-red-600 dark:text-red-400',
        bgColor: 'bg-red-100 dark:bg-red-900/40',
      }
    default:
      return {
        label: t('sceneryManager.categoryOther'),
        color: 'text-gray-600 dark:text-gray-400',
        bgColor: 'bg-gray-100 dark:bg-gray-800/50',
      }
  }
})

const hasMissingDeps = computed(() => props.entry.missingLibraries.length > 0)
const hasDuplicateTiles = computed(
  () => props.entry.duplicateTiles && props.entry.duplicateTiles.length > 0,
)
const hasDuplicateAirports = computed(
  () => props.entry.duplicateAirports && props.entry.duplicateAirports.length > 0,
)
const hasDuplicates = computed(() => hasDuplicateTiles.value || hasDuplicateAirports.value)
const canOpenUpdater = computed(() => {
  const updateUrl = (props.entry.updateUrl || '').trim().toLowerCase()
  return !!updateUrl && !updateUrl.startsWith('x-updater:')
})
const duplicatesCount = computed(() => {
  const all = new Set<string>()
  if (props.entry.duplicateTiles) {
    for (const p of props.entry.duplicateTiles) all.add(p)
  }
  if (props.entry.duplicateAirports) {
    for (const p of props.entry.duplicateAirports) all.add(p)
  }
  return all.size
})
const isGlobalAirportsEntry = computed(() => props.entry.folderName === GLOBAL_AIRPORTS_ENTRY_NAME)
const isFirst = computed(() => props.index === 0)
const isLast = computed(() => props.index === props.totalCount - 1)
const canToggleFlatten = computed(
  () => props.entry.flattenAvailable && !!props.entry.airportId && !isGlobalAirportsEntry.value,
)
const canOpenFlattenPage = computed(
  () => !!props.entry.airportId && !isGlobalAirportsEntry.value && !canToggleFlatten.value,
)
const assignedCustomGroup = computed(() =>
  sceneryStore.customGroups.find((group) =>
    group.manualFolderNames.includes(props.entry.folderName),
  ),
)

// Lock state
const NOISE_WORDS = new Set([
  'XP11', 'XP12', 'MESH', 'INTL', 'CITY', 'ROAD', 'ROADS', 'SFD', 'AERO', 'TAIM',
  'PACK', 'APTS', 'DEMO', 'SCEN', 'PART', 'FLAT', 'FREE', 'BASE', 'FULL',
  'AIRP', 'PARK', 'HIGH', 'WIDE', 'DEPO', 'APRO', 'TERM'
])

const isSpecialAirport = computed(() => {
  const name = props.entry.folderName.toLowerCase().trim()
  return (
    name.startsWith('x-plane airports') ||
    name.startsWith('xplane airports') ||
    name.startsWith('x-plane landmarks') ||
    name.startsWith('xplane landmarks') ||
    name.startsWith('aep overlay') ||
    name.startsWith('aep_overlay') ||
    name.startsWith('aep-overlay') ||
    name === 'global airports' ||
    name === GLOBAL_AIRPORTS_ENTRY_NAME.toLowerCase()
  )
})

const isNonAirportLayer = (folderName: string) => {
  const lower = folderName.toLowerCase()
  return (
    lower.includes('_city') ||
    lower.includes('-city') ||
    lower.endsWith(' city') ||
    lower.includes('landmark') ||
    lower.includes('overlay') ||
    lower.includes('phototerrain') ||
    lower.includes('mesh') ||
    lower.includes('roads') ||
    lower.endsWith('_lib') ||
    lower.endsWith('-lib') ||
    lower.includes('library')
  )
}

const displayIcao = computed(() => {
  if (isSpecialAirport.value) return null
  if (props.entry.category !== 'Airport') return null
  if (isNonAirportLayer(props.entry.folderName)) return null

  // Priority 1: 4-letter alphabetic ICAO directly from folder name
  const tokens = props.entry.folderName.toUpperCase().split(/[^A-Z0-9]+/).filter(Boolean)
  for (const token of tokens) {
    if (token.length === 4 && /^[A-Z]{4}$/.test(token) && !NOISE_WORDS.has(token)) {
      return token
    }
  }

  // Priority 2: airportId from apt.dat (must be 4 alphabetic letters, no heliport codes like DC04 or EX16)
  if (props.entry.airportId) {
    const trimmed = props.entry.airportId.trim().toUpperCase()
    if (trimmed.length === 4 && /^[A-Z]{4}$/.test(trimmed) && !NOISE_WORDS.has(trimmed)) {
      return trimmed
    }
  }

  return null
})

const isItemLocked = computed(() => lockStore.isLocked('scenery', props.entry.folderName))

function handleToggleLock() {
  lockStore.toggleLock('scenery', props.entry.folderName)
}

async function handleDoubleClick() {
  if (props.showCheckbox) return
  if (isGlobalAirportsEntry.value) return

  if (!appStore.xplanePath) {
    modalStore.showError(t('sceneryManager.noXplanePath'))
    return
  }

  try {
    await invoke('open_scenery_folder', {
      xplanePath: appStore.xplanePath,
      folderName: props.entry.folderName,
    })
  } catch (error) {
    const apiError = parseApiError(error)
    if (apiError?.code === 'not_found') {
      await sceneryStore.loadData()
      modalStore.showError(t('sceneryManager.stalePathMessage'))
      return
    }
    modalStore.showError(t('sceneryManager.openFolderFailed') + ': ' + getErrorMessage(error))
  }
}

function handleClick(event: Event) {
  if (isGlobalAirportsEntry.value) {
    return
  }

  const target = event.target as HTMLElement
  const isInteractiveClick = Boolean(
    target.closest('button') ||
    target.closest('input') ||
    target.closest('select') ||
    target.closest('textarea') ||
    target.closest('a') ||
    target.closest('.drag-handle'),
  )

  if (props.showCheckbox) {
    if (!isInteractiveClick) {
      event.stopPropagation()
      emit('toggle-select', props.entry.folderName)
    }
    return
  }

  // Don't trigger if clicking on interactive elements
  if (isInteractiveClick) {
    return
  }

  // If has missing libraries, show modal on single click
  if (hasMissingDeps.value) {
    event.stopPropagation()
    emit('show-missing-libs', props.entry)
  } else if (hasDuplicates.value) {
    // If has duplicates (but no missing deps), show that modal
    event.stopPropagation()
    emit('show-duplicate-tiles', props.entry)
  }
}

function handleContextMenu(event: MouseEvent) {
  const menuItems: ContextMenuItem[] = []

  menuItems.push({
    id: 'toggle-enabled',
    label: props.entry.enabled ? t('contextMenu.disable') : t('contextMenu.enable'),
    icon: props.entry.enabled
      ? '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"/></svg>'
      : '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/></svg>',
  })

  if (!isGlobalAirportsEntry.value) {
    menuItems.push({
      id: 'open-folder',
      label: t('contextMenu.openFolder'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"/></svg>',
    })
  }

  if (!isGlobalAirportsEntry.value && assignedCustomGroup.value) {
    menuItems.push({
      id: `remove-custom-group:${assignedCustomGroup.value.id}`,
      label: t('sceneryManager.removeFromGroup'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24"><path stroke-width="2" d="M4 5.75A1.75 1.75 0 015.75 4h4.1l1.55 1.75h6.85A1.75 1.75 0 0120 7.5V9"/><rect x="4" y="8.25" width="16" height="10.75" rx="2" stroke-width="2"/><path stroke-width="2" d="M14 13.5h4"/></svg>',
      dividerAfter: true,
    })
  } else if (!isGlobalAirportsEntry.value && sceneryStore.customGroups.length > 0) {
    menuItems.push({
      id: 'assign-custom-group',
      label: t('sceneryManager.assignToGroup'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24"><path stroke-width="2" d="M4 5.75A1.75 1.75 0 015.75 4h4.1l1.55 1.75h6.85A1.75 1.75 0 0120 7.5V9"/><rect x="4" y="8.25" width="16" height="10.75" rx="2" stroke-width="2"/><circle cx="15.25" cy="12" r="1.25" stroke-width="1.8"/><circle cx="15.25" cy="15" r="1.25" stroke-width="1.8"/></svg>',
      children: sceneryStore.customGroups.map((group) => ({
        id: `assign-custom-group:${group.id}`,
        label: group.name,
      })),
    })
    menuItems[menuItems.length - 1].dividerAfter = true
  }

    if (!isGlobalAirportsEntry.value) {
    menuItems.push({
      id: 'manage-storage',
      label: props.entry.isSymlink ? `🔗 ${t('multiDrive.menuManage')}` : `💽 ${t('multiDrive.menuMove')}`,
      icon: '<svg class="w-4 h-4 text-indigo-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"/></svg>',
    })
    if (props.entry.isSymlink) {
      menuItems.push({
        id: 'open-target-folder',
        label: `📂 ${t('multiDrive.openTargetFolder')}`,
        icon: '<svg class="w-4 h-4 text-indigo-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/></svg>',
      })
    }
  }

  if (canOpenUpdater.value && !isGlobalAirportsEntry.value) {
    menuItems.push({
      id: 'update',
      label: t('management.startUpdate'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/></svg>',
    })
  }

  if (canToggleFlatten.value) {
    menuItems.push({
      id: 'toggle-flatten',
      label: props.entry.flattened ? t('airportFlatten.disable') : t('airportFlatten.enable'),
      icon: props.entry.flattened
        ? '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M7 7h10M7 17h10"/></svg>'
        : '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5v14"/></svg>',
      disabled: props.flattenBusy,
    })
  } else if (canOpenFlattenPage.value) {
    menuItems.push({
      id: 'open-flatten-page',
      label: t('airportFlatten.openPage'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M13 5l7 7-7 7"/></svg>',
    })
  }

  if (!props.disableReorder) {
    menuItems.push({
      id: 'move-up',
      label: t('sceneryManager.moveUp'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>',
      disabled: isFirst.value,
    })
    menuItems.push({
      id: 'move-down',
      label: t('sceneryManager.moveDown'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>',
      disabled: isLast.value || props.disableMoveDown,
    })
  }

  if (hasMissingDeps.value) {
    menuItems.push({
      id: 'show-missing-libs',
      label: t('sceneryManager.missingLibraries'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>',
    })
  }

  if (hasDuplicates.value) {
    menuItems.push({
      id: 'show-duplicate-tiles',
      label: t('sceneryManager.duplicates'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2h-2M8 7H6a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2v-2"/></svg>',
    })
  }

  // Divider before lock/delete
  if (menuItems.length > 0) {
    menuItems[menuItems.length - 1].dividerAfter = true
  }

  if (!isGlobalAirportsEntry.value) {
    menuItems.push({
      id: 'toggle-lock',
      label: isItemLocked.value ? t('management.unlock') : t('management.lock'),
      icon: isItemLocked.value
        ? '<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"/></svg>'
        : '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"/></svg>',
    })

    menuItems.push({
      id: 'delete',
      label: t('sceneryManager.delete'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>',
      danger: true,
    })
  }

  contextMenu.show(event, menuItems, async (id: string) => {
    if (id.startsWith('assign-custom-group:')) {
      emit('assign-custom-group', {
        folderName: props.entry.folderName,
        groupId: id.slice('assign-custom-group:'.length),
      })
      return
    }

    if (id.startsWith('remove-custom-group:')) {
      emit('remove-custom-group', {
        folderName: props.entry.folderName,
        groupId: id.slice('remove-custom-group:'.length),
      })
      return
    }

    switch (id) {
      case 'toggle-enabled':
        emit('toggle-enabled', props.entry.folderName)
        break
      case 'open-folder':
        handleDoubleClick()
        break
      case 'manage-storage':
        emit('manage-storage', props.entry)
        break
      case 'open-target-folder':
        try {
          await invoke('open_target_storage_folder', {
            xplanePath: appStore.xplanePath,
            folderName: props.entry.folderName
          })
        } catch (e) {
          console.error(e)
        }
        break
      case 'update':
        emit('update', props.entry.folderName)
        break
      case 'toggle-flatten':
        emit('toggle-flatten', props.entry)
        break
      case 'open-flatten-page':
        emit('open-flatten-page', props.entry)
        break
      case 'move-up':
        emit('move-up', props.entry.folderName)
        break
      case 'move-down':
        emit('move-down', props.entry.folderName)
        break
      case 'show-missing-libs':
        emit('show-missing-libs', props.entry)
        break
      case 'show-duplicate-tiles':
        emit('show-duplicate-tiles', props.entry)
        break
      case 'toggle-lock':
        handleToggleLock()
        break
      case 'delete':
        emit('show-delete-confirm', props.entry)
        break
    }
  })
}
</script>

<template>
  <div
    class="flex items-center gap-2.5 p-2.5 rounded-xl border transition-all hover:bg-gray-50 dark:hover:bg-slate-800/50 hover:dark:border-sky-500/30"
    :class="[
      entry.category === 'Unrecognized'
        ? entry.enabled
          ? 'bg-red-50/50 dark:bg-red-900/10 border-red-300 dark:border-red-700'
          : 'bg-red-50/30 dark:bg-red-900/5 border-red-300 dark:border-red-700'
        : entry.enabled
          ? 'bg-white dark:bg-slate-900/70 border-gray-200 dark:border-slate-800'
          : 'bg-gray-50 dark:bg-slate-950/40 border-gray-200/50 dark:border-slate-800/50 opacity-60',
      selected
        ? 'ring-2 ring-blue-400 dark:ring-blue-500 border-blue-300 dark:border-blue-600'
        : '',
      hasMissingDeps || hasDuplicates ? 'cursor-pointer' : '',
      showCheckbox ? 'cursor-pointer' : '',
    ]"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @contextmenu.prevent="handleContextMenu"
  >
    <!-- Selection checkbox -->
    <button
      v-if="showCheckbox"
      class="flex-shrink-0 w-4 h-4 rounded border-2 transition-all duration-150 flex items-center justify-center"
      :class="
        selected
          ? 'bg-blue-500 border-blue-500'
          : 'border-gray-300 dark:border-slate-600 hover:border-sky-400 dark:hover:border-sky-400'
      "
      :title="selected ? t('management.selectedCount', { count: 1 }) : t('management.selectAll')"
      @click.stop="emit('toggle-select', entry.folderName)"
    >
      <svg
        v-if="selected"
        class="w-3 h-3 text-white"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
      </svg>
    </button>

    <!-- Drag handle -->
    <div
      v-if="!props.disableReorder && !props.showCheckbox"
      class="cursor-grab active:cursor-grabbing text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 drag-handle select-none"
    >
      <svg
        class="w-4 h-4 pointer-events-none"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16" />
      </svg>
    </div>

    <!-- Enable/Disable toggle -->
    <ToggleSwitch
      :model-value="entry.enabled"
      :disabled="false"
      active-class="bg-blue-500"
      inactive-class="bg-gray-300 dark:bg-gray-600"
      :aria-label="entry.enabled ? t('contextMenu.disable') : t('contextMenu.enable')"
      @update:model-value="emit('toggle-enabled', entry.folderName)"
    />

    <!-- Folder name with optional [ ICAO ] -->
    <div
      class="flex-1 min-w-0 flex items-center text-sm font-medium text-gray-900 dark:text-gray-100 truncate"
      :title="displayIcao ? `[ ${displayIcao} ] ${entry.folderName}` : entry.folderName"
    >
      <span
        v-if="displayIcao"
        class="inline-block w-[78px] font-semibold text-sky-600 dark:text-sky-400 shrink-0 text-left select-all"
      >[ {{ displayIcao }} ]</span>
      <span class="truncate">{{ entry.folderName }}</span>
    </div>

    <!-- Missing dependencies warning (before category badge) -->
    <div
      v-if="hasMissingDeps"
      class="flex-shrink-0 flex items-center gap-0.5 px-1.5 py-0.5 rounded text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 cursor-pointer"
      :title="t('sceneryManager.clickToViewMissingLibs')"
      @click.stop="emit('show-missing-libs', entry)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
        />
      </svg>
      <span class="text-[10px] font-medium">{{ entry.missingLibraries.length }}</span>
    </div>

    <!-- Duplicate warning badge -->
    <div
      v-if="hasDuplicates"
      class="flex-shrink-0 flex items-center gap-0.5 px-1.5 py-0.5 rounded text-orange-600 dark:text-orange-400 bg-orange-50 dark:bg-orange-900/20 cursor-pointer"
      :title="t('sceneryManager.clickToViewDuplicates')"
      @click.stop="emit('show-duplicate-tiles', entry)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2h-2M8 7H6a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2v-2"
        />
      </svg>
      <span class="text-[10px] font-medium">{{ duplicatesCount }}</span>
    </div>

    <!-- Category badge -->
    <span
      class="flex-shrink-0 px-1.5 py-0.5 rounded text-[10px] font-medium"
      :class="[categoryConfig.color, categoryConfig.bgColor]"
    >
      {{ categoryConfig.label }}
    </span>

    <!-- Geo info badge (continent) -->
    <span
      v-if="entry.continent"
      class="flex-shrink-0 px-1.5 py-0.5 rounded text-[10px] font-medium text-gray-600 dark:text-gray-400 bg-gray-100 dark:bg-gray-700"
      :title="entry.continent"
    >
      {{ entry.continent }}
    </span>

    <!-- Multi-Drive Junction / Symlink badge -->
    <span
      v-if="entry.isSymlink"
      class="flex-shrink-0 px-1.5 py-0.5 rounded text-[10px] font-semibold flex items-center gap-1 text-indigo-700 dark:text-indigo-300 bg-indigo-100 dark:bg-indigo-950/60 border border-indigo-200 dark:border-indigo-800"
      :title="entry.linkTarget ? `${t('multiDrive.targetPath')}: ${entry.linkTarget}` : t('multiDrive.badgeLinked')"
    >
      <svg class="w-3 h-3 text-indigo-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"></path>
      </svg>
      <span>{{ entry.linkTarget ? (entry.linkTarget.match(/^[A-Za-z]:/)?.[0] || 'Linked') : 'Linked' }}</span>
    </span>

    <button
      v-if="canOpenUpdater && !isGlobalAirportsEntry"
      class="flex-shrink-0 px-1.5 py-0.5 rounded text-[10px] font-medium text-white transition-colors flex items-center gap-1"
      :class="
        entry.hasUpdate
          ? 'bg-emerald-500 hover:bg-emerald-600 ring-2 ring-emerald-300 dark:ring-emerald-700'
          : 'bg-emerald-500 hover:bg-emerald-600'
      "
      :title="
        entry.hasUpdate && entry.latestVersion
          ? `${entry.version || ''} → ${entry.latestVersion}`
          : t('management.startUpdate')
      "
      @click.stop="emit('update', entry.folderName)"
    >
      <span
        v-if="entry.hasUpdate"
        class="inline-block w-1.5 h-1.5 rounded-full bg-white"
        aria-hidden="true"
      ></span>
      <span>{{ entry.hasUpdate ? t('management.hasUpdate') : t('management.startUpdate') }}</span>
    </button>

    <!-- Move buttons -->
    <div v-if="!props.disableReorder" class="flex-shrink-0 flex gap-0.5">
      <button
        :disabled="isFirst"
        class="p-0.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        :title="t('sceneryManager.moveUp')"
        @click="emit('move-up', entry.folderName)"
      >
        <svg
          class="w-3.5 h-3.5 text-gray-600 dark:text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
        </svg>
      </button>
      <button
        :disabled="isLast || props.disableMoveDown"
        class="p-0.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        :title="t('sceneryManager.moveDown')"
        @click="emit('move-down', entry.folderName)"
      >
        <svg
          class="w-3.5 h-3.5 text-gray-600 dark:text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>
    </div>

    <!-- Multi-Drive Storage button -->
    <button
      v-if="!isGlobalAirportsEntry"
      class="flex-shrink-0 p-0.5 rounded text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 dark:hover:bg-indigo-950/40 transition-colors"
      :class="entry.isSymlink ? 'text-indigo-600 dark:text-indigo-400' : ''"
      :title="entry.isSymlink ? t('multiDrive.menuManage') : t('multiDrive.menuMove')"
      @click.stop="emit('manage-storage', entry)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"></path>
      </svg>
    </button>

    <!-- Lock button -->
    <button
      v-if="!isGlobalAirportsEntry"
      class="flex-shrink-0 p-0.5 rounded transition-colors"
      :class="
        isItemLocked
          ? 'text-amber-500 dark:text-amber-400 hover:bg-amber-100 dark:hover:bg-amber-900/30'
          : 'text-gray-400 dark:text-gray-500 hover:text-amber-500 dark:hover:text-amber-400 hover:bg-amber-100 dark:hover:bg-amber-900/30'
      "
      :title="isItemLocked ? t('management.unlock') : t('management.lock')"
      @click.stop="handleToggleLock"
    >
      <!-- Locked icon (solid) -->
      <svg v-if="isItemLocked" class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
        <path
          d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"
        />
      </svg>
      <!-- Unlocked icon (outline) -->
      <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"
        />
      </svg>
    </button>

    <!-- Delete button -->
    <button
      v-if="!isGlobalAirportsEntry"
      class="flex-shrink-0 p-0.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors"
      :title="t('sceneryManager.delete')"
      @click.stop="emit('show-delete-confirm', entry)"
    >
      <svg
        class="w-3.5 h-3.5 text-gray-400 hover:text-red-500 dark:text-gray-500 dark:hover:text-red-400"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
        />
      </svg>
    </button>
  </div>
</template>
