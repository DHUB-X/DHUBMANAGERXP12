import { healthDetailMessages } from '@/i18n/healthDetails'

const enPlugin = healthDetailMessages.en.findings.addons_binary_integrity
const idPlugin = healthDetailMessages.id.findings.addons_binary_integrity

// Check-specific wording shared by the UI and English exports.
export const DOCTOR_FINDINGS: Record<string, { en: [string, string]; id: [string, string] }> = {
  ...Object.fromEntries(
    (
      [
        'installation_structure',
        'installation_stock_aircraft',
        'installation_airfoils',
        'installation_asset_access',
        'installation_write_access',
        'installation_permission_access',
        'system_vulkan_runtime',
        'scenery_files',
        'installation_scan_access',
      ] as const
    ).map((key) => {
      const en = healthDetailMessages.en.findings[key]
      const id = healthDetailMessages.id.findings[key]
      return [
        key.replace('_', '.'),
        { en: [en.title, en.description], id: [id.title, id.description] },
      ]
    }),
  ),
  'system.defender_exclusion': {
    en: [
      'Windows Defender exclusion',
      'The X-Plane folder is not excluded from Windows Defender real-time scans. Antivirus false positives may quarantine or delete aircraft DLLs, SASL plugins, or gauge scripts.',
    ],
    id: [
      'Pengecualian Windows Defender',
      'Folder X-Plane belum dikecualikan dari pemindaian Windows Defender. Deteksi false positive antivirus dapat menghapus atau mengkarantina file DLL pesawat, plugin SASL, atau skrip gauge.',
    ],
  },
  'navdata.bundled': {
    en: [
      healthDetailMessages.en.findings.navdata_bundled.title,
      healthDetailMessages.en.findings.navdata_bundled.description,
    ],
    id: [
      healthDetailMessages.id.findings.navdata_bundled.title,
      healthDetailMessages.id.findings.navdata_bundled.description,
    ],
  },
  'navdata.supplemental': {
    en: [
      healthDetailMessages.en.findings.navdata_supplemental.title,
      healthDetailMessages.en.findings.navdata_supplemental.description,
    ],
    id: [
      healthDetailMessages.id.findings.navdata_supplemental.title,
      healthDetailMessages.id.findings.navdata_supplemental.description,
    ],
  },
  'navdata.scan_access': {
    en: [
      healthDetailMessages.en.findings.navdata_scan_access.title,
      healthDetailMessages.en.findings.navdata_scan_access.description,
    ],
    id: [
      healthDetailMessages.id.findings.navdata_scan_access.title,
      healthDetailMessages.id.findings.navdata_scan_access.description,
    ],
  },
  'scenery.registration': {
    en: [
      'Scenery registration file',
      'scenery_packs.ini is missing. X-Plane may not have generated it yet. Synchronize the scenery list to create it.',
    ],
    id: [
      'File registrasi scenery',
      'scenery_packs.ini tidak ditemukan. X-Plane mungkin belum membuatnya. Sinkronkan daftar scenery untuk membuatnya.',
    ],
  },
  'scenery.invalid_entries': {
    en: [
      healthDetailMessages.en.findings.scenery_invalid_entries.title,
      healthDetailMessages.en.findings.scenery_invalid_entries.description,
    ],
    id: [
      healthDetailMessages.id.findings.scenery_invalid_entries.title,
      healthDetailMessages.id.findings.scenery_invalid_entries.description,
    ],
  },
  'scenery.missing_paths': {
    en: [
      'Missing scenery paths',
      'Enabled scenery registrations point to missing folders. Check the listed paths and reconnect external drives before changing the scenery list.',
    ],
    id: [
      'Path scenery hilang',
      'Entri scenery yang aktif mengarah ke folder yang tidak ada. Periksa path yang tercantum dan pastikan drive eksternal terhubung sebelum mengubah daftar scenery.',
    ],
  },
  'scenery.duplicate_paths': {
    en: [
      'Duplicate scenery registrations',
      'The same enabled scenery folder is registered more than once. Review the listed line numbers in scenery_packs.ini.',
    ],
    id: [
      'Registrasi scenery duplikat',
      'Folder scenery aktif yang sama terdaftar lebih dari sekali. Periksa nomor baris yang tercantum di scenery_packs.ini.',
    ],
  },
  'scenery.broken_links': {
    en: [
      'Broken scenery links',
      'Scenery links or shortcuts have unavailable targets. Reconnect the target disk or correct the link before rescanning.',
    ],
    id: [
      'Tautan scenery rusak',
      'Tautan simbolik atau shortcut scenery mengarah ke target yang tidak tersedia. Hubungkan kembali disk target atau perbaiki tautan lalu pindai ulang.',
    ],
  },
  'navdata.empty_files': {
    en: [
      'Empty navigation data files',
      'Custom navigation files contain zero bytes. Restore a complete navigation data package; empty files cannot supply routes or waypoints.',
    ],
    id: [
      'File data navigasi kosong',
      'File navigasi kustom berukuran nol byte. Pasang ulang paket data navigasi yang lengkap; file kosong tidak dapat menyediakan rute atau waypoint.',
    ],
  },
  'addons.binary_integrity': {
    en: [enPlugin.title, enPlugin.description],
    id: [idPlugin.title, idPlugin.description],
  },
  'stability.log_freshness': {
    en: [
      'Log age',
      'Log.txt is over seven days old. Its findings describe an older session; reproduce the problem and scan again using the resulting log.',
    ],
    id: [
      'Usia log',
      'Log.txt berusia lebih dari tujuh hari. Temuan di dalamnya berasal dari sesi lama; ulangi masalah lalu pindai ulang menggunakan log terbaru.',
    ],
  },
  'updates.gateway_unavailable': {
    en: [
      'Gateway airports not checked',
      'Gateway did not return an update result for every airport. The listed airports are unverified; retry the full diagnosis later.',
    ],
    id: [
      'Bandara Gateway belum diperiksa',
      'Gateway tidak mengembalikan status pembaruan untuk semua bandara. Bandara yang tercantum belum terverifikasi; coba lagi diagnosa penuh nanti.',
    ],
  },
}

export const HEALTH_RECOVERY_COPY = {
  en: {
    title: 'Repairs & backups',
    noMatches: 'No matching checks',
    trySearch: 'Change the search or switch to All.',
    backupError: 'Could not read repair backups. Retry the check.',
    empty: 'No repair backups.',
    restore: 'Restore',
    confirm: 'Continue',
    restoreBody:
      'Restore this backup? Current files will be saved in another backup before replacement. Close X-Plane first.',
    backupNote: 'Each repair saves the original files locally. Restore them here.',
    englishReport: 'Reports are exported in English. Original log evidence is retained.',
    disable_injectors: 'Disable injector DLLs',
    rebuild_shader_cache: 'Rebuild shader cache',
    reset_preferences: 'Reset preferences',
    disable_injectorsBody:
      'Move detected ReShade proxy DLLs into a backup. The injector will no longer load. Other DLLs, shaders and configuration files are preserved.',
    rebuild_shader_cacheBody:
      'Back up the renderer shader cache. X-Plane will regenerate it; the next launch or flight may take longer.',
    reset_preferencesBody:
      'Back up and reset the entire Output/preferences folder, including graphics, controls and joystick assignments. X-Plane will generate defaults. Restore the backup here to undo this.',
  },
  id: {
    title: 'Perbaikan & Cadangan',
    noMatches: 'Tidak ada pemeriksaan yang cocok',
    trySearch: 'Ubah kata kunci pencarian atau alihkan ke Semua.',
    backupError: 'Tidak dapat membaca cadangan perbaikan. Coba pindai ulang.',
    empty: 'Belum ada cadangan perbaikan.',
    restore: 'Pulihkan',
    confirm: 'Lanjutkan',
    restoreBody:
      'Pulihkan cadangan ini? File saat ini akan disimpan ke cadangan lain sebelum ditimpa. Tutup X-Plane terlebih dahulu.',
    backupNote: 'Setiap perbaikan menyimpan file asli secara lokal. Pulihkan di sini.',
    englishReport: 'Laporan diekspor dalam bahasa Inggris. Bukti log asli tetap dipertahankan.',
    disable_injectors: 'Nonaktifkan DLL injector',
    rebuild_shader_cache: 'Bangun ulang shader cache',
    reset_preferences: 'Reset preferensi',
    disable_injectorsBody:
      'Pindahkan DLL proxy ReShade yang terdeteksi ke cadangan agar injector tidak lagi dimuat. DLL lain, shader, dan konfigurasi tetap aman.',
    rebuild_shader_cacheBody:
      'Cadangkan shader cache renderer. X-Plane akan membuatnya ulang; pemuatan pertama berikutnya mungkin sedikit lebih lama.',
    reset_preferencesBody:
      'Cadangkan dan reset seluruh folder Output/preferences, termasuk pengaturan grafis, kontrol, dan joystick. X-Plane akan membuat setelan default. Pulihkan cadangan di sini untuk membatalkan.',
  },
}
