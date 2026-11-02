import { createI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import en from './en'
import id from './id'
import { getInitialLocale, persistLocale, type SupportedLocale } from './shared'

const initialLocale = getInitialLocale()

function applyDocumentLocale(locale: SupportedLocale) {
  if (typeof document === 'undefined') return
  document.documentElement.lang = locale
  document.documentElement.dir = 'ltr'
  document.documentElement.dataset.localeDirection = 'ltr'
}

export const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'en',
  messages: {
    en,
    id,
  },
})

applyDocumentLocale(initialLocale)

// Non-blocking function to sync initial locale with backend (call in App.vue)
export function syncLocaleToBackend() {
  applyDocumentLocale(i18n.global.locale.value)
  invoke('set_log_locale', { locale: i18n.global.locale.value }).catch(() => {
    // Ignore errors during initialization
  })
}

// Helper function to sync locale with backend when user changes language
export async function setLocale(locale: SupportedLocale) {
  i18n.global.locale.value = locale
  applyDocumentLocale(locale)
  persistLocale(locale)
  try {
    await invoke('set_log_locale', { locale })
  } catch (e) {
    console.debug('Failed to set backend locale:', e)
  }
}

export default i18n
