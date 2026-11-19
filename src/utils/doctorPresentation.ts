import type { DoctorCheckResult } from '@/types/doctor'
import en from '@/i18n/en'
import id from '@/i18n/id'
import { doctorAdvice } from '@/i18n/doctorAdvice'
import { getDoctorAdvice } from './doctorAdvice'
import { DOCTOR_FINDINGS } from './doctorFindings'
import { CHECK_NAME_KEYS, LEGACY_TEXT_ROOTS } from './doctorCatalog'

function message(
  locale: string,
  key: string,
  params: Record<string, string | number> = {},
): string {
  let value: unknown = locale === 'id' ? id : en
  for (const part of key.split('.')) {
    if (!value || typeof value !== 'object') return ''
    value = (value as Record<string, unknown>)[part]
  }
  return typeof value === 'string'
    ? value.replace(/\{(\w+)\}/g, (token, name: string) => String(params[name] ?? token))
    : ''
}

export function suggestDoctorCheck(
  check: DoctorCheckResult,
  locale: string,
  checks: readonly DoctorCheckResult[] = [],
  translateLegacy = (key: string, params: Record<string, string | number>) =>
    message(locale, key, params),
): string {
  const advice = getDoctorAdvice(check, checks)
  if (advice) return advice.map((key) => doctorAdvice[key][locale === 'id' ? 1 : 0]).join(' ')
  if (!['info', 'warning', 'critical'].includes(check.outcome)) return ''
  const root = check.id.startsWith('log.')
    ? `doctor.checks.log.${check.id.slice(4)}`
    : LEGACY_TEXT_ROOTS[check.id]
  return root ? translateLegacy(`${root}.suggestion`, check.params ?? {}) : ''
}

export function describeDoctorCheck(check: DoctorCheckResult, locale = 'en') {
  const params = check.params ?? {}
  const indonesian = locale === 'id'
  const name = CHECK_NAME_KEYS[check.id]
  const root = check.id.startsWith('log.')
    ? `doctor.checks.log.${check.id.slice(4)}`
    : LEGACY_TEXT_ROOTS[check.id]
  const finding = DOCTOR_FINDINGS[check.id]?.[indonesian ? 'id' : 'en']
  const title =
    finding?.[0] ||
    (name && message(locale, `doctor.center.checks.${name}`, params)) ||
    (root && message(locale, `${root}.title`, params)) ||
    check.id
  const generic = message(locale, `doctor.center.checkDescriptions.${check.outcome}`)
  if (check.id === 'addons.binary_integrity') {
    const root = 'healthDetails.findings.addons_binary_integrity'
    if (params.requiresRescan || check.outcome === 'pass') {
      const key = params.requiresRescan ? 'outdated' : 'scope'
      return { title, description: message(locale, `${root}.${key}`) }
    }
  }
  if (check.id === 'stability.last_session') {
    const descriptions: Record<string, string> = indonesian
      ? {
          crash:
            'Log.txt memuat penanda crash X-Plane secara eksplisit. Catatan ini saja belum tentu menentukan akar masalah.',
          normal: 'Log.txt memuat penanda penutupan normal X-Plane.',
          unknown:
            'Tidak ada penanda crash atau keluar normal yang tercatat. Hasil akhir sesi tidak dapat dipastikan dari log ini.',
          running: 'X-Plane sedang berjalan; log sesi saat ini masih ditulis.',
        }
      : {
          crash:
            'Log.txt contains an explicit X-Plane crash marker. This alone does not establish the root cause.',
          normal: 'Log.txt contains the normal X-Plane shutdown marker.',
          unknown:
            'No explicit crash or normal shutdown marker was recorded. The session outcome cannot be determined from this log.',
          running: 'X-Plane is running; the current log is still being written.',
        }
    return {
      title,
      description: descriptions[String(params.sessionEnd)] ?? descriptions.unknown!,
    }
  }
  if (check.id === 'updates.addons' && 'checked' in params) {
    const summary = message(locale, 'healthDetails.coverage.addonUpdates', params)
    return {
      title,
      description: params.manual
        ? `${summary} ${message(locale, 'healthDetails.coverage.manualUpdates', params)}`
        : summary,
    }
  }
  if (check.id === 'updates.gateway' && check.outcome === 'notApplicable') {
    return {
      title,
      description: message(locale, 'healthDetails.coverage.gatewayNoManaged'),
    }
  }
  if (
    check.id === 'updates.csl' &&
    ['pass', 'info'].includes(check.outcome) &&
    'checked' in params
  ) {
    return {
      title,
      description: message(locale, 'healthDetails.coverage.cslUpdates', params),
    }
  }
  if (check.id === 'navdata.cycles' && Number(params.future) > 0) {
    return {
      title,
      description: message(locale, 'healthDetails.coverage.navFuture'),
    }
  }
  if (check.id === 'navdata.cycles' && Number(params.conflicts) > 0) {
    return {
      title,
      description: message(locale, 'healthDetails.coverage.navConflict'),
    }
  }
  if (check.id === 'installation.scan_access' && check.outcome === 'unavailable' && finding) {
    return { title, description: finding[1] }
  }
  if (['unavailable', 'notApplicable', 'cancelled'].includes(check.outcome)) {
    return { title, description: generic }
  }
  if (finding) {
    return { title, description: check.outcome === 'pass' ? generic : finding[1] }
  }
  if (check.id === 'updates.gateway') {
    return {
      title,
      description: indonesian
        ? `Bandara terpasang: ${params.installed ?? 0}; berhasil diperiksa: ${params.checked ?? 0}; pembaruan tersedia: ${params.count ?? 0}.`
        : `Installed airports: ${params.installed ?? 0}; successfully checked: ${params.checked ?? 0}; updates available: ${params.count ?? 0}.`,
    }
  }
  if (check.id === 'system.injectors') {
    return {
      title,
      description:
        check.outcome === 'pass'
          ? generic
          : indonesian
            ? 'DLL proxy dan konfigurasi ReShade terdeteksi di folder root X-Plane. Keberadaan file belum tentu berarti sedang aktif dimuat.'
            : 'Proxy DLLs and ReShade configuration are present in the X-Plane root. Presence does not prove they were loaded.',
    }
  }
  const hasFinding = check.outcome !== 'pass'
  return {
    title,
    description: (hasFinding && root && message(locale, `${root}.description`, params)) || generic,
  }
}

export function formatDoctorDuration(milliseconds: number): string {
  const safeMilliseconds = Math.max(0, Math.round(milliseconds))
  if (safeMilliseconds < 1_000) return `${safeMilliseconds} ms`

  const totalSeconds = Math.round(safeMilliseconds / 1_000)
  if (totalSeconds < 60) return `${(safeMilliseconds / 1_000).toFixed(1)} s`

  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${minutes}m ${seconds}s`
}
