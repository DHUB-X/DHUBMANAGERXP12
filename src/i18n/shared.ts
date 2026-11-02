export const LOCALE_STORAGE_KEY = 'DHUB.locale'

export const SUPPORTED_LOCALES = ['en', 'id'] as const

export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number]

export const LOCALE_OPTIONS: Array<{ value: SupportedLocale; label: string }> = [
  { value: 'en', label: 'English' },
  { value: 'id', label: 'Indonesia' },
]

function matchLocale(input?: string | null): SupportedLocale | null {
  const locale = input?.trim().toLowerCase()

  if (!locale) return null
  if (locale.startsWith('id') || locale.startsWith('in')) return 'id'
  if (locale.startsWith('en')) return 'en'

  return null
}

export function normalizeLocale(input?: string | null): SupportedLocale {
  return matchLocale(input) ?? 'en'
}

export function getStoredLocale(): SupportedLocale | null {
  if (typeof window === 'undefined') return null

  const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY)
  return matchLocale(stored)
}

export function getSystemLocale(): SupportedLocale {
  if (typeof navigator === 'undefined') return 'en'

  const candidates = [
    ...(Array.isArray(navigator.languages) ? navigator.languages : []),
    navigator.language,
  ]

  for (const candidate of candidates) {
    const matched = matchLocale(candidate)
    if (matched) return matched
  }

  return 'en'
}

export function getInitialLocale(): SupportedLocale {
  return getStoredLocale() ?? 'en'
}

export function persistLocale(locale: SupportedLocale) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(LOCALE_STORAGE_KEY, locale)
}
