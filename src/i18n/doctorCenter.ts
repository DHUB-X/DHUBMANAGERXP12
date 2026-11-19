interface DoctorCenterMessages {
  heading: string
  intro: string
  quickCheck: string
  fullCheck: string
  cancel: string
  quickHint: string
  fullHint: string
  localPhase: string
  networkPhase: string
  progress: string
  automaticQuick: string
  export: string
  exportMarkdown: string
  exportJson: string
  includeSensitive: string
  excludeSensitive: string
  status: {
    allClear: string
    informational: string
    attention: string
    critical: string
    incomplete: string
    stale: string
    staleDescription: string
    partialDescription: string
    cancelledDescription: string
    coverage: string
    checksCovered: string
    lastChecked: string
    duration: string
    currentResult: string
    viewingHistory: string
  }
  outcomes: {
    pass: string
    info: string
    warning: string
    critical: string
    unavailable: string
    notApplicable: string
    cancelled: string
  }
  filters: {
    all: string
    issues: string
  }
  sections: {
    installation: string
    stability: string
    scenery: string
    addons: string
    navdata: string
    performance: string
    storage: string
    system: string
    xfast: string
    updates: string
  }
  checks: {
    installationStructure: string
    xplaneRunning: string
    xplaneSpace: string
    installationLocation: string
    readonlyFiles: string
    graphicsInjectors: string
    defenderExclusion: string
    competingOrganizer: string
    systemInventory: string
    availableMemory: string
    appDataSpace: string
    appDataWritable: string
    database: string
    sceneryIndex: string
    logAvailable: string
    crashDump: string
    lastSession: string
    betaBuild: string
    globalAirports: string
    loadOrder: string
    sceneryDependencies: string
    sceneryOverlaps: string
    duplicateAirports: string
    flattenOverrides: string
    customNavdata: string
    navdataCycles: string
    cifp: string
    navdataIntegrity: string
    navdataConsistency: string
    recentFailures: string
    cleanableFiles: string
    addonInventory: string
    addonPlatform: string
    addonUpdates: string
    gatewayUpdates: string
    appUpdate: string
    cslUpdates: string
    diagnosticProbe: string
    generic: string
  }
  checkDescriptions: {
    pass: string
    info: string
    warning: string
    critical: string
    unavailable: string
    notApplicable: string
    cancelled: string
  }
  remediation: {
    apply: string
    open: string
    guide: string
    safe: string
    confirm: string
    destructive: string
    fixAllSafe: string
    fixingAll: string
    confirmTitle: string
    confirmBody: string
    automaticAvailable: string
    manualReview: string
  }
  evidence: {
    title: string
    show: string
    hide: string
    none: string
  }
  history: {
    title: string
    description: string
    empty: string
    backToCurrent: string
    quick: string
    full: string
  }
  privacy: {
    title: string
    redacted: string
    sensitive: string
    confirmTitle: string
    confirmBody: string
  }
  system: {
    title: string
    os: string
    cpu: string
    memory: string
    gpu: string
    xplane: string
    xplaneDisk: string
    appDataDisk: string
  }
  messages: {
    noPathTitle: string
    noPathDescription: string
    configurePath: string
    noResultTitle: string
    noResultDescription: string
    noIssues: string
    noIssuesDescription: string
    xplaneRunning: string
    exportSuccess: string
    exportFailed: string
    fixSuccess: string
    fixFailed: string
    batchResult: string
    historyOnly: string
  }
}

type DoctorCenterLocale = 'en' | 'id'

export const doctorCenterMessages: Record<DoctorCenterLocale, DoctorCenterMessages> = {
  en: {
    heading: 'Health',
    intro:
      'Inspect X-Plane, your system, addons, storage, updates, and DHUB Manager in one place.',
    quickCheck: 'Quick check',
    fullCheck: 'Full diagnosis',
    cancel: 'Cancel scan',
    quickHint: 'Local essentials · no network checks',
    fullHint: 'Deep local inspection · includes online update checks',
    localPhase: 'Inspecting this computer',
    networkPhase: 'Checking online services',
    progress: '{done} of {total} checks completed',
    automaticQuick: 'A quick check runs once when Health is first opened in this session.',
    export: 'Export report',
    exportMarkdown: 'Markdown report',
    exportJson: 'JSON data',
    includeSensitive: 'Include original paths',
    excludeSensitive: 'Redact private paths',
    status: {
      allClear: 'Healthy',
      informational: 'Information',
      attention: 'Warnings',
      critical: 'Critical',
      incomplete: 'Incomplete',
      stale: 'Health result is out of date',
      staleDescription: 'This result is over 24 hours old. Run a new check before relying on it.',
      partialDescription:
        'Some checks could not finish. Unavailable checks are never treated as passed.',
      cancelledDescription: 'This scan was cancelled, so its coverage is incomplete.',
      coverage: 'Diagnostic coverage',
      checksCovered: '{covered} of {eligible} applicable checks',
      lastChecked: 'Checked {time}',
      duration: '{duration} duration',
      currentResult: 'Current result',
      viewingHistory: 'Viewing a saved result',
    },
    outcomes: {
      pass: 'Passed',
      info: 'Information',
      warning: 'Warning',
      critical: 'Critical',
      unavailable: 'Unavailable',
      notApplicable: 'Not applicable',
      cancelled: 'Cancelled',
    },
    filters: {
      all: 'All',
      issues: 'Issues',
    },
    sections: {
      installation: 'Installation',
      stability: 'Stability & crashes',
      scenery: 'Scenery',
      addons: 'Aircraft & plugins',
      navdata: 'Navigation data',
      performance: 'Performance',
      storage: 'Storage',
      system: 'System environment',
      xfast: 'DHUB Manager',
      updates: 'Updates',
    },
    checks: {
      installationStructure: 'X-Plane installation structure',
      xplaneRunning: 'X-Plane runtime state',
      xplaneSpace: 'X-Plane disk space',
      installationLocation: 'Installation location',
      readonlyFiles: 'Addon file permissions',
      graphicsInjectors: 'Graphics injectors',
      defenderExclusion: 'Windows Defender exclusion',
      competingOrganizer: 'Other scenery organizers',
      systemInventory: 'System inventory',
      availableMemory: 'Available system memory',
      appDataSpace: 'DHUB data disk space',
      appDataWritable: 'DHUB data access',
      database: 'DHUB database integrity',
      sceneryIndex: 'Scenery index consistency',
      logAvailable: 'X-Plane log availability',
      crashDump: 'Crash dump analysis',
      lastSession: 'Last X-Plane session',
      betaBuild: 'X-Plane release channel',
      globalAirports: 'Global Airports status',
      loadOrder: 'Scenery load order',
      sceneryDependencies: 'Scenery library dependencies',
      sceneryOverlaps: 'Scenery tile overlaps',
      duplicateAirports: 'Duplicate airport definitions',
      flattenOverrides: 'Airport flatten overrides',
      customNavdata: 'Custom navigation data',
      navdataCycles: 'AIRAC cycle health',
      cifp: 'CIFP procedure data',
      navdataIntegrity: 'Navigation data integrity',
      navdataConsistency: 'Navigation cycle consistency',
      recentFailures: 'Recent install and update activity',
      cleanableFiles: 'Cleanable X-Plane output files',
      addonInventory: 'Addon inventory',
      addonPlatform: 'Plugin platform compatibility',
      addonUpdates: 'Addon updates',
      gatewayUpdates: 'Gateway scenery updates',
      appUpdate: 'DHUB Manager update',
      cslUpdates: 'CSL and ALTITUDE updates',
      diagnosticProbe: '{probe} check availability',
      generic: 'Diagnostic: {id}',
    },
    checkDescriptions: {
      pass: 'The check completed without finding a problem.',
      info: 'This does not block use, but may be useful to review.',
      warning: 'This can affect reliability or expected simulator behavior.',
      critical: 'This can prevent correct operation and should be addressed first.',
      unavailable: 'The check could not produce a trustworthy result.',
      notApplicable: 'This check does not apply to the current installation.',
      cancelled: 'The check stopped before a result was available.',
    },
    remediation: {
      apply: 'Apply fix',
      open: 'Open related area',
      guide: 'Review guidance',
      safe: 'Safe automatic fix',
      confirm: 'Confirmation required',
      destructive: 'Manual high-risk action',
      fixAllSafe: 'Fix all safe items',
      fixingAll: 'Applying safe fixes…',
      confirmTitle: 'Apply this repair?',
      confirmBody:
        'This repair changes local X-Plane or DHUB data. Review the evidence before continuing.',
      automaticAvailable: 'An automatic repair is available for this result.',
      manualReview: 'Review the evidence and related settings before making manual changes.',
    },
    evidence: {
      title: 'Evidence',
      show: 'Show evidence',
      hide: 'Hide evidence',
      none: 'No additional evidence was recorded.',
    },
    history: {
      title: 'Diagnostic history',
      description: 'The latest 20 completed runs are kept locally for this X-Plane installation.',
      empty: 'No saved health checks yet.',
      backToCurrent: 'Back to current result',
      quick: 'Quick',
      full: 'Full scan',
    },
    privacy: {
      title: 'Report privacy',
      redacted: 'Private paths are redacted by default.',
      sensitive: 'Original local paths will be included.',
      confirmTitle: 'Include private local paths?',
      confirmBody:
        'The exported report may reveal your user name, X-Plane location, and DHUB data path. Share it only with people you trust.',
    },
    system: {
      title: 'System snapshot',
      os: 'Operating system',
      cpu: 'Processor',
      memory: 'Memory',
      gpu: 'Graphics',
      xplane: 'X-Plane version',
      xplaneDisk: 'X-Plane disk',
      appDataDisk: 'DHUB data disk',
    },
    messages: {
      noPathTitle: 'Connect an X-Plane installation',
      noPathDescription:
        'Health needs a valid X-Plane path before it can diagnose the simulator and its addons.',
      configurePath: 'Configure X-Plane path',
      noResultTitle: 'Ready to diagnose',
      noResultDescription:
        'Run a quick check for local essentials or a full diagnosis for every available check.',
      noIssues: 'No issues found',
      noIssuesDescription: 'Switch to All to see passed and not-applicable checks.',
      xplaneRunning: 'Close X-Plane before applying repairs that change files.',
      exportSuccess: 'Health report exported.',
      exportFailed: 'Could not export the health report.',
      fixSuccess: 'Repair applied and the affected checks were run again.',
      fixFailed: 'The repair could not be completed.',
      batchResult: '{applied} safe repairs applied; {failed} failed.',
      historyOnly: 'Saved results are read-only. Run a new check before applying repairs.',
    },
  },
  id: {
    heading: 'Kesehatan',
    intro:
      'Periksa X-Plane, sistem komputer, addon, penyimpanan, pembaruan, dan DHUB Manager di satu tempat.',
    quickCheck: 'Cek cepat',
    fullCheck: 'Diagnosa penuh',
    cancel: 'Batalkan pindai',
    quickHint: 'Pemeriksaan lokal penting · tanpa cek jaringan',
    fullHint: 'Inspeksi lokal mendalam · termasuk cek pembaruan online',
    localPhase: 'Memeriksa komputer ini',
    networkPhase: 'Memeriksa layanan online',
    progress: '{done} dari {total} pemeriksaan selesai',
    automaticQuick: 'Pemeriksaan cepat berjalan otomatis sekali saat halaman Kesehatan dibuka.',
    export: 'Ekspor laporan',
    exportMarkdown: 'Laporan Markdown',
    exportJson: 'Data JSON',
    includeSensitive: 'Sertakan path asli',
    excludeSensitive: 'Sembunyikan path pribadi',
    status: {
      allClear: 'Sehat',
      informational: 'Informasi',
      attention: 'Peringatan',
      critical: 'Kritis',
      incomplete: 'Belum lengkap',
      stale: 'Hasil pemeriksaan sudah usang',
      staleDescription:
        'Hasil ini berusia lebih dari 24 jam. Jalankan pemeriksaan baru sebelum mengambil tindakan.',
      partialDescription:
        'Beberapa pemeriksaan tidak dapat diselesaikan. Pemeriksaan yang tidak tersedia tidak dianggap lulus.',
      cancelledDescription: 'Pemindaian dibatalkan sehingga cakupannya tidak lengkap.',
      coverage: 'Cakupan diagnosa',
      checksCovered: '{covered} dari {eligible} pemeriksaan yang berlaku',
      lastChecked: 'Diperiksa {time}',
      duration: 'Durasi {duration}',
      currentResult: 'Hasil saat ini',
      viewingHistory: 'Melihat hasil tersimpan',
    },
    outcomes: {
      pass: 'Lulus',
      info: 'Informasi',
      warning: 'Peringatan',
      critical: 'Kritis',
      unavailable: 'Tidak tersedia',
      notApplicable: 'Tidak berlaku',
      cancelled: 'Dibatalkan',
    },
    filters: {
      all: 'Semua',
      issues: 'Masalah',
    },
    sections: {
      installation: 'Instalasi',
      stability: 'Stabilitas & crash',
      scenery: 'Scenery',
      addons: 'Pesawat & plugin',
      navdata: 'Data navigasi',
      performance: 'Performa',
      storage: 'Penyimpanan',
      system: 'Lingkungan sistem',
      xfast: 'DHUB Manager',
      updates: 'Pembaruan',
    },
    checks: {
      installationStructure: 'Struktur instalasi X-Plane',
      xplaneRunning: 'Status proses X-Plane',
      xplaneSpace: 'Ruang disk X-Plane',
      installationLocation: 'Lokasi instalasi',
      readonlyFiles: 'Izin file addon',
      graphicsInjectors: 'Injector grafis',
      defenderExclusion: 'Pengecualian Windows Defender',
      competingOrganizer: 'Pengatur scenery lain',
      systemInventory: 'Inventaris sistem',
      availableMemory: 'Memori sistem tersedia',
      appDataSpace: 'Ruang disk data DHUB',
      appDataWritable: 'Akses tulis data DHUB',
      database: 'Integritas database DHUB',
      sceneryIndex: 'Konsistensi indeks scenery',
      logAvailable: 'Ketersediaan Log.txt X-Plane',
      crashDump: 'Analisis crash dump',
      lastSession: 'Sesi terakhir X-Plane',
      betaBuild: 'Saluran rilis X-Plane',
      globalAirports: 'Status Global Airports',
      loadOrder: 'Urutan muat scenery',
      sceneryDependencies: 'Dependensi library scenery',
      sceneryOverlaps: 'Tumpang tindih tile scenery',
      duplicateAirports: 'Definisi bandara duplikat',
      flattenOverrides: 'Pengaturan flatten bandara',
      customNavdata: 'Data navigasi kustom',
      navdataCycles: 'Kesehatan siklus AIRAC',
      cifp: 'Data prosedur CIFP',
      navdataIntegrity: 'Integritas data navigasi',
      navdataConsistency: 'Konsistensi siklus navigasi',
      recentFailures: 'Aktivitas pasang & pembaruan terakhir',
      cleanableFiles: 'File Output X-Plane yang bisa dibersihkan',
      addonInventory: 'Inventaris addon',
      addonPlatform: 'Kompatibilitas platform plugin',
      addonUpdates: 'Pembaruan addon',
      gatewayUpdates: 'Pembaruan scenery Gateway',
      appUpdate: 'Pembaruan DHUB Manager',
      cslUpdates: 'Pembaruan CSL & ALTITUDE',
      diagnosticProbe: 'Ketersediaan pemeriksaan {probe}',
      generic: 'Diagnosa: {id}',
    },
    checkDescriptions: {
      pass: 'Pemeriksaan selesai tanpa menemukan masalah.',
      info: 'Tidak menghambat penggunaan, namun baik untuk ditinjau.',
      warning: 'Dapat memengaruhi keandalan atau perilaku simulator.',
      critical: 'Dapat mencegah simulator berjalan benar dan perlu segera ditangani.',
      unavailable: 'Pemeriksaan tidak dapat menghasilkan data yang valid.',
      notApplicable: 'Pemeriksaan ini tidak berlaku untuk instalasi saat ini.',
      cancelled: 'Pemeriksaan berhenti sebelum hasil tersedia.',
    },
    remediation: {
      apply: 'Terapkan perbaikan',
      open: 'Buka halaman terkait',
      guide: 'Lihat panduan',
      safe: 'Perbaikan otomatis aman',
      confirm: 'Perlu konfirmasi',
      destructive: 'Tindakan manual berisiko',
      fixAllSafe: 'Perbaiki semua item aman',
      fixingAll: 'Menerapkan perbaikan aman…',
      confirmTitle: 'Terapkan perbaikan ini?',
      confirmBody:
        'Perbaikan ini mengubah file lokal X-Plane atau data DHUB. Tinjau bukti sebelum melanjutkan.',
      automaticAvailable: 'Perbaikan otomatis tersedia untuk temuan ini.',
      manualReview: 'Tinjau bukti dan pengaturan terkait sebelum mengubah secara manual.',
    },
    evidence: {
      title: 'Bukti',
      show: 'Tampilkan bukti',
      hide: 'Sembunyikan bukti',
      none: 'Tidak ada bukti tambahan yang tercatat.',
    },
    history: {
      title: 'Riwayat diagnosa',
      description: '20 hasil pemeriksaan selesai terakhir disimpan lokal untuk instalasi X-Plane ini.',
      empty: 'Belum ada riwayat pemeriksaan.',
      backToCurrent: 'Kembali ke hasil saat ini',
      quick: 'Cepat',
      full: 'Diagnosa penuh',
    },
    privacy: {
      title: 'Privasi laporan',
      redacted: 'Path pribadi disembunyikan secara default.',
      sensitive: 'Path lokal asli akan disertakan.',
      confirmTitle: 'Sertakan path lokal pribadi?',
      confirmBody:
        'Laporan yang diekspor dapat memperlihatkan nama pengguna, lokasi X-Plane, dan path data DHUB Anda. Bagikan hanya kepada orang yang dipercaya.',
    },
    system: {
      title: 'Ringkasan sistem',
      os: 'Sistem operasi',
      cpu: 'Prosesor',
      memory: 'Memori',
      gpu: 'Grafis',
      xplane: 'Versi X-Plane',
      xplaneDisk: 'Disk X-Plane',
      appDataDisk: 'Disk data DHUB',
    },
    messages: {
      noPathTitle: 'Hubungkan instalasi X-Plane',
      noPathDescription:
        'Fitur Kesehatan memerlukan path X-Plane yang valid sebelum dapat mendiagnosa simulator dan addon.',
      configurePath: 'Atur path X-Plane',
      noResultTitle: 'Siap mendiagnosa',
      noResultDescription:
        'Jalankan cek cepat untuk pemeriksaan lokal penting atau diagnosa penuh untuk semua pemeriksaan.',
      noIssues: 'Tidak ada masalah ditemukan',
      noIssuesDescription: 'Alihkan ke Semua untuk melihat pemeriksaan yang lulus.',
      xplaneRunning: 'Tutup X-Plane terlebih dahulu sebelum menerapkan perbaikan file.',
      exportSuccess: 'Laporan kesehatan berhasil diekspor.',
      exportFailed: 'Gagal mengekspor laporan kesehatan.',
      fixSuccess: 'Perbaikan diterapkan dan pemeriksaan terkait dijalankan ulang.',
      fixFailed: 'Perbaikan tidak dapat diselesaikan.',
      batchResult: '{applied} perbaikan aman diterapkan; {failed} gagal.',
      historyOnly: 'Hasil tersimpan bersifat baca-saja. Jalankan pemeriksaan baru untuk memperbaiki.',
    },
  },
}
