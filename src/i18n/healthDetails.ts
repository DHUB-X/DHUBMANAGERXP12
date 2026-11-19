export const healthDetailMessages = {
  en: {
    coverage: {
      gatewayNoManaged: 'No Gateway airports managed by DHUB Manager are installed.',
      cslUpdates: 'Checked {checked} installed model packages. Updates found: {count}.',
      manualUpdates: '{manual} require a manual upgrade.',
      navConflict: 'Some AIRAC cycle numbers disagree. Review the listed differences.',
      addonUpdates:
        'Compared {checked} of {installed} supported add-ons. Updates found: {count}. External updater: {external}.',
      navFuture:
        'Some installed navigation packages are not yet effective. Review the listed dates.',
    },
    fields: {
      count: 'Affected items',
      installed: 'Installed airports',
      checked: 'Items checked',
      free: 'Available space',
      total: 'Total capacity',
      available: 'Available memory',
      line: 'Log line',
      days: 'Log age (days)',
      errors: 'Read errors',
      indexed: 'Indexed entries',
      found: 'Entries on disk',
      files: 'Files',
      size: 'Storage used',
      platform: 'Host platform',
      version: 'Version',
      expired: 'Expired data packages',
      expiring: 'Expiring data packages',
      unknown: 'Validity unknown',
      cycles: 'Detected AIRAC cycles',
      aircraft: 'Aircraft',
      plugins: 'Plugins',
      csl: 'X-CSL models',
      altitude: 'Altitude data',
      cause: 'Suspected cause',
      confidence: 'Evidence score',
      module: 'Related module',
      os: 'Operating system',
      arch: 'Architecture',
      cores: 'Logical processors',
      memory: 'Installed memory',
      capped: 'Scan limit reached',
      sessionEnd: 'Session ending',
      steam: 'Steam installation',
    },
    scope: {
      installation: 'Installation folders and file access were checked.',
      stability: 'The latest session log and available crash evidence were reviewed.',
      scenery: 'Scenery registrations, dependencies and loading consistency were checked.',
      addons: 'Installed addons and their compatibility were checked.',
      navdata: 'Installed navigation data was checked for completeness and consistency.',
      performance: 'Available system resources were checked.',
      storage: 'Disk capacity and files eligible for cleanup were checked.',
      system: 'System information and graphics environment were inspected.',
      xfast: 'Application data access, database and scenery index were checked.',
      updates: 'Installed versions were compared with available update information.',
    },
    nav: {
      expired:
        'Some installed navigation packages have expired. Update the listed data sources before relying on their procedures and waypoints.',
      expiring: 'Some navigation packages expire soon. Plan their next update.',
      unknown:
        'Validity could not be established for some packages. Check their cycle and source metadata.',
      current: 'The navigation packages with known validity are current.',
    },
    yes: 'Yes',
    no: 'No',
    findings: {
      scenery_files: {
        title: 'Enabled scenery files',
        description:
          'Some enabled packages contain unreadable, empty, or invalid scenery files. Review the listed files.',
      },
      installation_write_access: {
        title: 'Installation write access',
        description:
          'The listed folders cannot accept new files. Check their permissions and volume state.',
      },
      installation_permission_access: {
        title: 'Permission check incomplete',
        description: 'Some permissions could not be verified. Review the listed errors.',
      },
      system_vulkan_runtime: {
        title: 'Vulkan runtime',
        description:
          'The system Vulkan loader is missing or below 1.3. Install or update the graphics driver.',
      },
      installation_structure: {
        title: 'X-Plane installation structure',
        description:
          'Restore the listed startup files with the X-Plane installer or Steam file verification.',
      },
      installation_stock_aircraft: {
        title: 'Stock aircraft',
        description:
          'The listed aircraft files are missing from the installation. Restore the affected stock aircraft.',
      },
      installation_airfoils: {
        title: 'Airfoils',
        description:
          'The listed shared or aircraft airfoils are missing or damaged. Restore the affected files.',
      },
      installation_asset_access: {
        title: 'Installation file access',
        description: 'Some installation files could not be verified. Review the listed errors.',
      },
      navdata_bundled: {
        title: 'Bundled navigation data',
        description: 'Repair the X-Plane installation to restore the listed bundled files.',
      },
      navdata_supplemental: {
        title: 'Supplemental navigation files',
        description: 'Review the listed ARINC 424, FAA procedure, or user navigation files.',
      },
      navdata_scan_access: {
        title: 'Navigation data access',
        description:
          'Some navigation data or cycle metadata could not be inspected. Review the listed errors.',
      },
      scenery_registration: {
        title: 'Scenery registration file',
        description:
          'scenery_packs.ini is missing. X-Plane may not have generated it yet. Synchronize the scenery list to create it.',
      },
      scenery_invalid_entries: {
        title: 'Invalid scenery registrations',
        description:
          'Some enabled entries in scenery_packs.ini have no path. Review the listed lines.',
      },
      scenery_missing_paths: {
        title: 'Missing scenery paths',
        description:
          'Enabled scenery registrations point to missing folders. Check the listed paths and reconnect external drives before changing the scenery list.',
      },
      scenery_duplicate_paths: {
        title: 'Duplicate scenery registrations',
        description:
          'The same enabled scenery folder is registered more than once. Review the listed line numbers in scenery_packs.ini.',
      },
      scenery_broken_links: {
        title: 'Broken scenery links',
        description:
          'Scenery links or shortcuts have unavailable targets. Reconnect the target disk or correct the link before rescanning.',
      },
      navdata_empty_files: {
        title: 'Empty navigation data files',
        description:
          'Custom navigation files contain zero bytes. Restore a complete navigation data package; empty files cannot supply routes or waypoints.',
      },
      addons_binary_integrity: {
        title: 'Plugin binary headers and architecture',
        description:
          'An entry selected for this platform is empty or has an invalid or unsupported executable header. Review the file and the reason listed in the evidence.',
        scope:
          'The scan checks executable headers of global, aircraft and enabled registered scenery plugin entries. Dependencies and actual loading are outside this check.',
        outdated:
          'This saved result predates platform entry selection. Run a new full scan to verify the plugin files.',
      },
      installation_scan_access: {
        title: 'File check incomplete',
        description:
          'Some files could not be inspected. See the listed read errors or scan limit notices.',
      },
      stability_log_freshness: {
        title: 'Log age',
        description:
          'Log.txt is over seven days old. Its findings describe an older session; reproduce the problem and scan again using the resulting log.',
      },
      updates_gateway_unavailable: {
        title: 'Gateway airports not checked',
        description:
          'Gateway did not return an update result for every airport. The listed airports are unverified; retry the full diagnosis later.',
      },
    },
    purposes: {
      installation_structure:
        'Required X-Plane folders are checked for missing installation components.',
      installation_xplane_running:
        'The simulator process is checked before file repairs are offered.',
      installation_location: 'The installation location is checked for system-protected folders.',
      installation_readonly:
        'Installation files are checked for read-only attributes that can block updates.',
      storage_xplane_space:
        'Free space on the simulator drive is checked against installation requirements.',
      storage_app_data_space: 'Free space on the application data drive is checked.',
      system_inventory:
        'Operating system, processor architecture and installed memory are recorded for troubleshooting.',
      system_injectors:
        'The installation root is checked for graphics proxy libraries and injector configuration.',
      system_defender_exclusion:
        'Windows Defender exclusions are checked to prevent antivirus false positives from removing aircraft DLLs or plugins.',
      performance_available_memory:
        'Available physical memory is checked for resource pressure before running X-Plane.',
      xfast_app_data: 'The application data directory is checked for write access.',
      xfast_database: 'The application database is checked for readable and consistent records.',
      xfast_scenery_index: 'The scenery index is compared with packages present on disk.',
      stability_log_available:
        'The available Log.txt is checked for an identifiable X-Plane session.',
      stability_crash_dump:
        'Available crash dump evidence is reviewed for probable causes, not a confirmed diagnosis.',
      system_beta_build:
        'The recorded simulator version is checked for a beta or release candidate build.',
      scenery_global_airports:
        'The Global Airports registration is checked for disabled default airports.',
      scenery_load_order: 'The scenery loading list is checked against the managed package order.',
      scenery_dependencies: 'Enabled scenery is checked for missing library dependencies.',
      scenery_overlaps: 'Scenery packages are checked for overlapping geographic tiles.',
      scenery_duplicate_airports:
        'Airport registrations are checked for duplicate airport identifiers.',
      scenery_flatten:
        'Managed airport flattening overrides are checked for drift from their saved state.',
      navdata_custom_data:
        'The Custom Data directory is checked for a custom simulator navigation database.',
      navdata_cifp: 'Instrument procedure data is checked for an available CIFP directory.',
      navdata_integrity: 'Required navigation data files are checked for missing components.',
      navdata_consistency:
        'Installed navigation data components are checked for differing AIRAC cycles.',
      xfast_recent_failures:
        'Recent installation and update records are checked for failed operations.',
      storage_cleanable:
        'Recognized caches and generated output are measured for optional cleanup.',
      addons_inventory:
        'Installed aircraft and plugins are counted from the selected installation.',
      addons_platform: 'Installed plugin platforms are compared with the host operating system.',
      updates_addons: 'Supported addon update sources are checked for newer releases.',
      updates_app: 'The installed DHUB Manager version is compared with release information.',
      updates_csl:
        'Installed model and altitude data versions are compared with available updates.',
    },
    sessions: {
      crash:
        'Log.txt contains an explicit X-Plane crash marker. The log alone does not establish the root cause.',
      normal: 'Log.txt records a normal X-Plane shutdown.',
      unknown:
        'The log has no explicit crash or normal shutdown marker. The session ending cannot be determined.',
      running: 'X-Plane is running and the current log is still being written.',
    },
  },
  id: {
    coverage: {
      gatewayNoManaged: 'Tidak ada bandara Gateway yang dikelola oleh DHUB Manager.',
      cslUpdates: 'Memeriksa {checked} paket model terpasang. Pembaruan ditemukan: {count}.',
      manualUpdates: '{manual} memerlukan pembaruan manual.',
      navConflict: 'Beberapa nomor siklus AIRAC tidak konsisten. Tinjau perbedaannya.',
      addonUpdates:
        'Membandingkan {checked} dari {installed} addon yang didukung. Pembaruan: {count}. Updater eksternal: {external}.',
      navFuture: 'Beberapa paket data navigasi belum berlaku. Tinjau tanggal efektifnya.',
    },
    fields: {
      count: 'Item terdampak',
      installed: 'Bandara terpasang',
      checked: 'Item diperiksa',
      free: 'Ruang tersedia',
      total: 'Kapasitas total',
      available: 'Memori tersedia',
      line: 'Baris log',
      days: 'Usia log (hari)',
      errors: 'Galat baca',
      indexed: 'Entri terindeks',
      found: 'Entri di disk',
      files: 'File',
      size: 'Penyimpanan terpakai',
      platform: 'Platform host',
      version: 'Versi',
      expired: 'Paket data kedaluwarsa',
      expiring: 'Segera kedaluwarsa',
      unknown: 'Validitas tidak diketahui',
      cycles: 'Siklus AIRAC terdeteksi',
      aircraft: 'Pesawat',
      plugins: 'Plugin',
      csl: 'Model X-CSL',
      altitude: 'Data Altitude',
      cause: 'Dugaan penyebab',
      confidence: 'Skor bukti',
      module: 'Modul terkait',
      os: 'Sistem operasi',
      arch: 'Arsitektur',
      cores: 'Prosesor logis',
      memory: 'Memori terpasang',
      capped: 'Batas pindai tercapai',
      sessionEnd: 'Akhir sesi',
      steam: 'Instalasi Steam',
    },
    scope: {
      installation: 'Folder instalasi dan akses file telah diperiksa.',
      stability: 'Log sesi terakhir dan bukti crash yang tersedia telah ditinjau.',
      scenery: 'Registrasi scenery, dependensi, dan konsistensi urutan muat telah diperiksa.',
      addons: 'Addon terpasang dan kompatibilitasnya telah diperiksa.',
      navdata: 'Data navigasi terpasang telah diperiksa kelengkapan dan konsistensinya.',
      performance: 'Sumber daya sistem yang tersedia telah diperiksa.',
      storage: 'Kapasitas disk dan file cache yang dapat dibersihkan telah diperiksa.',
      system: 'Informasi sistem dan lingkungan grafis telah diperiksa.',
      xfast: 'Akses data aplikasi, database, dan indeks scenery telah diperiksa.',
      updates: 'Versi terpasang telah dibandingkan dengan informasi pembaruan.',
    },
    nav: {
      expired:
        'Beberapa paket data navigasi telah kedaluwarsa. Perbarui sumber data yang tercantum sebelum menggunakannya.',
      expiring: 'Beberapa paket data navigasi akan segera kedaluwarsa.',
      unknown: 'Masa berlaku sebagian paket tidak dapat ditentukan. Periksa metadata siklusnya.',
      current: 'Paket data navigasi dengan masa berlaku yang diketahui masih aktif.',
    },
    yes: 'Ya',
    no: 'Tidak',
    findings: {
      scenery_files: {
        title: 'File scenery aktif',
        description:
          'Beberapa paket scenery aktif memiliki file yang tidak terbaca, kosong, atau rusak. Tinjau daftar file.',
      },
      installation_write_access: {
        title: 'Akses tulis instalasi',
        description:
          'Folder yang tercantum tidak dapat menerima file baru. Periksa izin folder dan status disk.',
      },
      installation_permission_access: {
        title: 'Pemeriksaan izin belum lengkap',
        description: 'Sebagian izin tidak dapat diverifikasi. Tinjau galat yang tercantum.',
      },
      system_vulkan_runtime: {
        title: 'Runtime Vulkan',
        description:
          'Loader Vulkan sistem tidak ditemukan atau di bawah versi 1.3. Pasang atau perbarui driver GPU.',
      },
      installation_structure: {
        title: 'Struktur instalasi X-Plane',
        description:
          'Pulihkan file startup yang tercantum menggunakan installer X-Plane atau verifikasi file Steam.',
      },
      installation_stock_aircraft: {
        title: 'Pesawat bawaan',
        description:
          'File pesawat bawaan yang tercantum hilang dari instalasi. Pulihkan pesawat bawaan terkait.',
      },
      installation_airfoils: {
        title: 'Airfoil',
        description:
          'File airfoil bersama atau pesawat hilang/rusak. Pulihkan file terkait.',
      },
      installation_asset_access: {
        title: 'Akses file instalasi',
        description: 'Sebagian file instalasi tidak dapat diverifikasi. Tinjau galat yang tercantum.',
      },
      navdata_bundled: {
        title: 'Data navigasi bawaan',
        description: 'Perbaiki instalasi X-Plane untuk memulihkan file navigasi bawaan yang tercantum.',
      },
      navdata_supplemental: {
        title: 'File navigasi tambahan',
        description: 'Tinjau file ARINC 424, prosedur FAA, atau file navigasi pengguna yang tercantum.',
      },
      navdata_scan_access: {
        title: 'Akses data navigasi',
        description:
          'Sebagian data navigasi atau metadata siklus tidak dapat diperiksa. Tinjau galat yang tercantum.',
      },
      scenery_registration: {
        title: 'File registrasi scenery',
        description:
          'scenery_packs.ini tidak ditemukan. Sinkronkan daftar scenery untuk membuatnya.',
      },
      scenery_invalid_entries: {
        title: 'Registrasi scenery tidak valid',
        description:
          'Beberapa entri aktif di scenery_packs.ini tidak memiliki path. Tinjau baris yang tercantum.',
      },
      scenery_missing_paths: {
        title: 'Path scenery hilang',
        description:
          'Registrasi scenery aktif mengarah ke folder yang tidak ada. Periksa path dan koneksi drive eksternal.',
      },
      scenery_duplicate_paths: {
        title: 'Registrasi scenery duplikat',
        description:
          'Folder scenery aktif yang sama terdaftar lebih dari sekali di scenery_packs.ini.',
      },
      scenery_broken_links: {
        title: 'Tautan scenery rusak',
        description:
          'Tautan simbolik atau shortcut scenery mengarah ke target yang tidak tersedia.',
      },
      navdata_empty_files: {
        title: 'File data navigasi kosong',
        description:
          'File navigasi kustom berukuran nol byte. Pasang ulang paket data navigasi yang lengkap.',
      },
      addons_binary_integrity: {
        title: 'Header biner & arsitektur plugin',
        description:
          'Plugin yang dipilih untuk platform ini kosong atau memiliki header eksekusi tidak valid/tidak didukung.',
        scope:
          'Pemindaian memeriksa header biner plugin global, pesawat, dan scenery aktif.',
        outdated:
          'Hasil tersimpan ini berasal dari versi lama. Jalankan diagnosa penuh baru untuk memverifikasi plugin.',
      },
      installation_scan_access: {
        title: 'Pemeriksaan file belum lengkap',
        description:
          'Sebagian file tidak dapat diperiksa. Lihat galat baca atau pemberitahuan batas pindai.',
      },
      stability_log_freshness: {
        title: 'Usia log',
        description:
          'Log.txt berusia lebih dari tujuh hari. Ulangi masalah lalu pindai ulang dengan log terbaru.',
      },
      updates_gateway_unavailable: {
        title: 'Bandara Gateway belum diperiksa',
        description:
          'Gateway tidak mengembalikan status untuk semua bandara. Coba lagi diagnosa penuh nanti.',
      },
    },
    purposes: {
      installation_structure:
        'Folder wajib X-Plane diperiksa untuk komponen instalasi yang hilang.',
      installation_xplane_running:
        'Proses simulator diperiksa sebelum perbaikan file ditawarkan.',
      installation_location: 'Lokasi instalasi diperiksa terhadap folder sistem yang dilindungi.',
      installation_readonly:
        'File instalasi diperiksa untuk atribut read-only yang dapat menghambat pembaruan.',
      storage_xplane_space:
        'Ruang kosong drive simulator diperiksa terhadap kebutuhan instalasi.',
      storage_app_data_space: 'Ruang kosong drive data aplikasi diperiksa.',
      system_inventory:
        'Sistem operasi, arsitektur prosesor, dan memori dicatat untuk pemecahan masalah.',
      system_injectors:
        'Root instalasi diperiksa untuk library proxy grafis dan konfigurasi injector.',
      system_defender_exclusion:
        'Pengecualian Windows Defender diperiksa agar antivirus tidak menghapus DLL pesawat atau plugin akibat false positive.',
      performance_available_memory:
        'Memori fisik tersedia diperiksa sebelum menjalankan X-Plane.',
      xfast_app_data: 'Direktori data aplikasi diperiksa untuk akses tulis.',
      xfast_database: 'Database aplikasi diperiksa keterbacaan dan konsistensinya.',
      xfast_scenery_index: 'Indeks scenery dibandingkan dengan paket yang ada di disk.',
      stability_log_available:
        'Log.txt yang tersedia diperiksa untuk sesi X-Plane yang dapat dikenali.',
      stability_crash_dump:
        'Bukti crash dump ditinjau untuk dugaan penyebab, bukan diagnosa pasti.',
      system_beta_build:
        'Versi simulator diperiksa apakah menggunakan build beta atau release candidate.',
      scenery_global_airports:
        'Registrasi Global Airports diperiksa agar bandara bawaan tidak nonaktif.',
      scenery_load_order: 'Urutan muat scenery dibandingkan dengan urutan paket yang dikelola.',
      scenery_dependencies: 'Scenery aktif diperiksa untuk library dependensi yang hilang.',
      scenery_overlaps: 'Paket scenery diperiksa untuk tile geografis yang tumpang tindih.',
      scenery_duplicate_airports:
        'Registrasi bandara diperiksa untuk kode bandara yang duplikat.',
      scenery_flatten:
        'Pengaturan flatten bandara diperiksa konsistensinya terhadap status tersimpan.',
      navdata_custom_data:
        'Folder Custom Data diperiksa untuk database navigasi kustom.',
      navdata_cifp: 'Data prosedur instrumen diperiksa ketersediaan direktori CIFP-nya.',
      navdata_integrity: 'File data navigasi wajib diperiksa komponen yang hilang.',
      navdata_consistency:
        'Komponen data navigasi terpasang diperiksa perbedaan siklus AIRAC-nya.',
      xfast_recent_failures:
        'Riwayat pemasangan dan pembaruan terakhir diperiksa untuk operasi yang gagal.',
      storage_cleanable:
        'Cache dan file output yang dikenali diukur untuk pembersihan opsional.',
      addons_inventory:
        'Pesawat dan plugin terpasang dihitung dari instalasi yang dipilih.',
      addons_platform: 'Platform plugin terpasang dibandingkan dengan OS host.',
      updates_addons: 'Sumber pembaruan addon yang didukung diperiksa untuk rilis baru.',
      updates_app: 'Versi DHUB Manager terpasang dibandingkan dengan rilis terbaru.',
      updates_csl: 'Versi paket model CSL & Altitude dibandingkan dengan pembaruan tersedia.',
    },
    sessions: {
      crash:
        'Log.txt memuat penanda crash X-Plane secara eksplisit. Log saja belum tentu menentukan akar masalah.',
      normal: 'Log.txt mencatat penutupan normal X-Plane.',
      unknown:
        'Log tidak memiliki penanda crash atau keluar normal. Hasil akhir sesi tidak dapat dipastikan.',
      running: 'X-Plane sedang berjalan dan log saat ini masih ditulis.',
    },
  },
}
