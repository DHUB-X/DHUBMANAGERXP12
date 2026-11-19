export const doctorAdvice = {
  saveCrashLog: [
    'Save a copy of Log.txt before restarting. Repeat the same flight or startup steps and run a full diagnosis if the crash recurs.',
    "Simpan salinan Log.txt sebelum memulai ulang. Ulangi penerbangan yang sama dan jalankan diagnosa penuh jika crash terulang.",
  ],
  oldCrashLog: [
    'This crash is from an older session. Save the existing log, then reproduce the problem and scan again before changing settings.',
    "Catatan crash ini berasal dari sesi lama. Simpan log saat ini, ulangi masalah, lalu pindai ulang.",
  ],
  unknownSession: [
    'If X-Plane closed unexpectedly, save the current Log.txt before reopening it. If the problem recurs, run a full diagnosis and keep the new log for comparison.',
    "Jika X-Plane tertutup mendadak, simpan Log.txt sebelum membukanya kembali lalu jalankan diagnosa penuh jika terulang.",
  ],
  activeSession: [
    'Exit X-Plane normally, then run the check again to review the completed session.',
    "Tutup X-Plane secara normal lalu jalankan pemeriksaan ulang untuk meninjau sesi yang selesai.",
  ],
  sessionStateUnavailable: [
    'Confirm X-Plane has exited, then scan again so the completed log can be evaluated.',
    "Pastikan X-Plane sudah ditutup, lalu pindai ulang agar log sesi yang lengkap dapat dievaluasi.",
  ],
  missingLog: [
    'Start X-Plane from the selected installation, exit normally, then scan again using the generated Log.txt.',
    "Jalankan X-Plane dari instalasi terpilih, tutup secara normal, lalu pindai ulang menggunakan Log.txt yang dihasilkan.",
  ],
  unrecognizedLog: [
    'Confirm the selected folder is the simulator installation. Start X-Plane there and exit normally to generate a simulator Log.txt, then scan again.',
    "Pastikan folder terpilih adalah folder instalasi X-Plane yang benar, jalankan lalu tutup normal untuk membuat Log.txt baru.",
  ],
  oldLog: [
    'Save the current Log.txt if you need it for comparison. Reproduce the symptom and rerun the check against the new log.',
    "Simpan Log.txt saat ini jika diperlukan, ulangi masalah, lalu pindai ulang dengan log terbaru.",
  ],
  dumpDisabled: [
    'Enable crash dump analysis in Settings and run a full diagnosis while the crash log and dump are still available.',
    "Aktifkan analisis crash dump di Pengaturan lalu jalankan diagnosa penuh saat log dan dump masih tersedia.",
  ],
  dumpUnavailable: [
    'Keep the crash log and dump together. Use the log findings to continue troubleshooting and capture a fresh dump if the crash recurs.',
    "Simpan log crash dan file dump bersama-sama untuk pemecahan masalah lebih lanjut.",
  ],
  dumpMissing: [
    'Save Log.txt for the confirmed crash. If the crash recurs and produces a dump, keep it with the new log and rerun the full diagnosis.',
    "Simpan Log.txt untuk crash ini. Jika crash terulang dan membuat file dump, jalankan diagnosa penuh lagi.",
  ],
  dumpPlugin: [
    'If the related module belongs to a third-party plugin, temporarily disable that plugin and retry the same flight. Restore it before testing another plugin.',
    "Jika modul terkait berasal dari plugin pihak ketiga, nonaktifkan sementara plugin tersebut dan uji ulang penerbangan.",
  ],
  dumpGpu: [
    'Use the graphics errors in Log.txt to guide the next test. If the crash began after a graphics driver change, compare with the previous working driver.',
    "Perbarui atau pasang ulang driver GPU, nonaktifkan overlay/injector grafis, lalu uji ulang.",
  ],
  dumpMemory: [
    'Check Log.txt for memory pressure. If it appears, close memory-heavy apps and retry the same flight with a lower texture setting.',
    "Tutup aplikasi lain yang memakan RAM/VRAM besar dan kurangi beban scenery/tekstur sebelum mencoba ulang.",
  ],
  dumpInconclusive: [
    'Keep Log.txt and the dump for comparison. Try the same scenario with a stock aircraft and change one variable at a time if it still crashes.',
    "Bukti crash dump belum menentukan penyebab pasti. Gunakan temuan Log.txt dan isolasi plugin satu per satu.",
  ],
  selectInstallation: [
    'In Settings, select the X-Plane installation folder that contains the simulator executable. Reconnect its drive first if it is external, then scan again.',
    "Pilih folder instalasi X-Plane yang valid di halaman Pengaturan.",
  ],
  repairSteam: [
    'Close X-Plane and use Steam’s Verify integrity of game files for this installation. Rerun the check after Steam finishes.',
    "Jalankan Verify Integrity of Game Files melalui Steam untuk memulihkan file X-Plane yang hilang atau rusak.",
  ],
  repairInstaller: [
    'Close X-Plane and use the official X-Plane installer to update or repair this installation. Rerun the check afterward.',
    "Jalankan X-Plane Installer resmi untuk memulihkan file simulator yang hilang atau rusak.",
  ],
  repairStockFiles: [
    'Restore the listed stock files using Steam file verification or the official X-Plane installer, depending on how X-Plane was installed.',
    "Pulihkan file bawaan yang tercantum menggunakan verifikasi Steam atau X-Plane Installer resmi.",
  ],
  restoreAirfoils: [
    'Use the evidence to identify the missing or invalid airfoil. Restore stock files through your simulator installer and aircraft-specific files from the matching aircraft package.',
    "Pulihkan airfoil bawaan lewat installer X-Plane dan airfoil pesawat dari paket pesawat terkait.",
  ],
  readAccess: [
    'Check that the listed paths are reachable and readable from your account, then scan again. Reconnect any external volumes before changing files.',
    "Periksa izin baca folder dan pastikan drive tidak bermasalah atau terkunci oleh aplikasi lain.",
  ],
  scanLimit: [
    'The scan covered only part of the library. Check the listed area or package individually if you need to verify it now.',
    "Batas pemindaian tercapai. Jalankan diagnosa penuh jika Anda ingin cakupan pemeriksaan yang lebih mendalam.",
  ],
  writableFolders: [
    'Check the listed folders and make sure the volume and your account allow writes. If a regular file occupies a required folder path, back it up before correcting that path.',
    "Pastikan akun Windows Anda memiliki izin tulis ke folder yang tercantum dan tidak dikunci antivirus.",
  ],
  readOnlyFiles: [
    'If an install or update fails, inspect the specific files it could not write to. Change permissions only for files that should be writable.',
    "Hapus atribut read-only pada folder addon agar pemasangan dan pembaruan tidak gagal.",
  ],
  steamLocation: [
    'If Steam updates or add-on installs fail because of permissions, use Steam’s storage settings to move the installation to a writable library. Then select that location in DHUB Manager.',
    "Instalasi Steam terdeteksi. Gunakan verifikasi file Steam saat memulihkan file inti simulator.",
  ],
  protectedLocation: [
    'If X-Plane or its add-ons cannot be updated here, use a writable installation folder outside Program Files and select that folder in DHUB Manager.',
    "Folder berada di lokasi sistem yang dilindungi UAC (Program Files). Pertimbangkan memindahkan X-Plane ke folder non-sistem.",
  ],
  simulatorSpace: [
    'Free space on the X-Plane volume before the next install or update. Use Disk Usage to review large items and keep any files you still need.',
    "Kosongkan ruang disk pada drive X-Plane agar pemasangan addon dan cache berjalan lancar.",
  ],
  applicationSpace: [
    'Free space on the volume that contains DHUB Manager’s app data. Review other files on that volume first and keep the app database and settings.',
    "Kosongkan ruang disk pada drive data aplikasi DHUB Manager.",
  ],
  diskUnavailable: [
    'Check that the relevant volume is mounted and view its available space in your operating system before retrying the check.',
    "Status kapasitas disk tidak dapat dibaca. Pastikan drive terhubung dengan baik.",
  ],
  cleanable: [
    'Review the candidates in Output Cleanup. Clean only items you can regenerate or no longer need.',
    "Buka halaman Output Cleanup untuk membersihkan cache dan file sementara yang tidak diperlukan.",
  ],
  availableMemory: [
    'Close memory-heavy applications and check again before starting X-Plane. If available memory is still low during flight, compare the same flight with a lower texture setting.',
    "Tutup aplikasi latar belakang agar RAM bebas cukup sebelum menjalankan X-Plane.",
  ],
  hardware: [
    'Compare the CPU or memory listed in the evidence with the requirements for your X-Plane version before planning any hardware changes.',
    "Spesifikasi perangkat keras berada di bawah rekomendasi ideal; sesuaikan pengaturan grafis X-Plane.",
  ],
  hardwareUnavailable: [
    'Check the processor and installed memory in your operating system’s system information, then retry the check.',
    "Informasi perangkat keras tidak dapat dibaca sepenuhnya oleh sistem.",
  ],
  vulkan: [
    'Check the graphics driver for your GPU and operating system. Install a supported driver, restart if required, then check the Vulkan runtime again.',
    "Pasang atau perbarui driver kartu grafis terbaru yang mendukung Vulkan 1.3.",
  ],
  injector: [
    'If you are investigating a graphics or startup problem, close X-Plane and use the backed-up injector repair on a fresh check. Retest the same scenario and restore the DLLs if the symptom persists.',
    "Nonaktifkan atau hapus DLL injector grafis (seperti ReShade) karena sering menyebabkan CTD pada X-Plane.",
  ],
  beta: [
    'If the issue started after switching to a beta, compare the same scenario on a stable build supported by the affected add-on.',
    "Anda menjalankan versi Beta/RC X-Plane. Sebagian plugin atau pesawat mungkin belum sepenuhnya kompatibel.",
  ],
  registrationMissing: [
    'Reconnect any external scenery drives first. With X-Plane closed, use the sync action on a fresh check to create scenery_packs.ini, then review the resulting order.',
    "Sinkronkan daftar scenery atau jalankan X-Plane sekali agar scenery_packs.ini terbentuk.",
  ],
  registrationUnreadable: [
    'Keep a copy of scenery_packs.ini. Check whether your account can read it and whether it opens as a text file. Restore a known-good copy if it is damaged, then scan again.',
    "Periksa izin baca/tulis pada Custom Scenery/scenery_packs.ini.",
  ],
  invalidEntries: [
    'Back up scenery_packs.ini, then use the listed line numbers to fix empty or invalid entries. Keep valid entries in their existing order.',
    "Hapus atau perbaiki baris entri scenery yang tidak valid di scenery_packs.ini.",
  ],
  missingPaths: [
    'Reconnect the drive or correct the path to the package first. If you intentionally removed it, review the stale entry in Scenery Management after confirming the package is no longer needed.',
    "Periksa apakah folder scenery sudah dipindahkan atau drive eksternal belum terhubung.",
  ],
  duplicatePaths: [
    'Back up scenery_packs.ini and compare the listed entries. Keep one registration at the priority you want for that package.',
    "Hapus entri duplikat di scenery_packs.ini melalui tab Manajemen Scenery.",
  ],
  brokenLinks: [
    'Reconnect the target drive or repair the shortcut or link. Verify that it opens the intended scenery folder before running the check again.',
    "Perbaiki atau hapus shortcut/symlink scenery yang targetnya sudah tidak ada.",
  ],
  globalAirports: [
    'If you want the default airports to load, close X-Plane and re-enable Global Airports. Keep custom airports that should take priority above it.',
    "Aktifkan kembali *GLOBAL_AIRPORTS* di Manajemen Scenery agar bandara bawaan X-Plane tampil.",
  ],
  sortScenery: [
    'Reconnect external scenery drives and review the intended priorities before sorting. Close X-Plane before applying the new order.',
    "Jalankan Auto Sort pada Manajemen Scenery untuk merapikan urutan muat scenery.",
  ],
  manualScenery: [
    'Review the registered packages and adjust the order in Scenery Management if needed. Keep any intentional priority overrides.',
    "Pengurutan otomatis dimatikan; pastikan urutan scenery_packs.ini sudah benar secara manual.",
  ],
  sceneryOrganizers: [
    'If two tools are changing scenery_packs.ini, choose one to manage the order and turn off automatic sorting in the other.',
    "Pengatur scenery lain (seperti xOrganizer) terdeteksi. Hindari menjalankan keduanya bersamaan agar urutan tidak saling menimpa.",
  ],
  dependencies: [
    'For each listed package, check the developer’s requirements and install or enable the named libraries. If a library is already present, check that its package folder is registered.',
    "Pasang library scenery yang hilang sesuai daftar dependensi di Manajemen Scenery.",
  ],
  overlaps: [
    'Compare the listed packages for each tile and check their intended layering. If two packages replace the same layer, choose the one you want and test with the other temporarily disabled.',
    "Periksa scenery dengan tile geografis tumpang tindih; pastikan prioritas bandara berada di atas mesh/ortho.",
  ],
  duplicateAirports: [
    'If the wrong airport appears in the simulator, compare the listed packages for that airport and put the preferred custom airport above the other definitions.',
    "Nonaktifkan salah satu scenery bandara duplikat dengan kode ICAO yang sama agar objek tidak bertumpuk.",
  ],
  flattenDrifted: [
    'Review the listed airports and reapply any saved flattening overrides you still want after closing X-Plane.',
    "Buka Airport Flatten untuk menerapkan ulang pengaturan flatten yang tersimpan.",
  ],
  flattenMissing: [
    'For missing source files, locate or restore the airport package before reapplying its override.',
    "Sumber apt.dat untuk sebagian override flatten tidak ditemukan; periksa di Airport Flatten.",
  ],
  sceneryFiles: [
    'Use the paths in the evidence to identify the affected packages. Restore damaged files from a trusted copy of the same package, or temporarily disable just that package to test.',
    "Pasang ulang paket scenery yang file DSF/apt.dat-nya rusak atau kosong.",
  ],
  sceneryLog: [
    'Check the lines around the error for a package name and the failing file or missing library. Repair that dependency or temporarily disable just the affected package and retest.',
    "Periksa log error scenery dan pastikan semua library dependensi sudah terpasang lengkap.",
  ],
  oldPluginScan: [
    'Run a new full diagnosis to check the plugin entry that this platform actually selects.',
    "Jalankan diagnosa penuh baru untuk memverifikasi biner plugin pada versi terbaru.",
  ],
  pluginPlatform: [
    'Check the plugin’s supported operating systems and CPU architectures. Install the matching build from its developer or disable the incompatible copy if you do not need it.',
    "Plugin yang ditandai tidak memiliki biner untuk sistem operasi ini. Hapus atau pasang versi yang sesuai OS.",
  ],
  pluginBinary: [
    'Use the listed executable and reason to identify the plugin. Restore it from the developer’s complete package for your platform. Retest with that plugin temporarily disabled if X-Plane still fails to load it.',
    "File biner plugin (.xpl) rusak atau tidak cocok arsitekturnya. Pasang ulang plugin terkait.",
  ],
  isolatePlugin: [
    'Check the surrounding log lines for a plugin name. Temporarily disable that third-party plugin and repeat the same scenario. If no plugin is named, test recently changed plugins one at a time.',
    "Nonaktifkan sementara plugin yang dilaporkan error lalu uji ulang X-Plane untuk memastikan penyebabnya.",
  ],
  duplicatePlugin: [
    'Compare the listed plugin copies and their installation locations. Keep the copy required by your setup and temporarily disable the extra copy to test.',
    "Hapus salinan ganda plugin agar tidak terjadi konflik saat X-Plane memuat.",
  ],
  graphicsLog: [
    'Compare the same scenario after restarting X-Plane. If the error recurs, check the driver version and test any recently changed graphics settings or add-ons one at a time.',
    "Perbarui driver GPU dan coba bersihkan shader cache melalui panel Perbaikan jika error grafis berlanjut.",
  ],
  blockedLayer: [
    'If the blocked graphics layer was intentional, check with its developer for a supported version. If rendering works as expected, keep the current setup.',
    "Nonaktifkan aplikasi overlay atau layer pihak ketiga (ReShade, RivaTuner, dll) yang diblokir X-Plane.",
  ],
  logContext: [
    'Read the surrounding lines of Log.txt for the operation and any named component. If the symptom recurs, save the new log and compare the errors before making changes.',
    "Tinjau konteks baris Log.txt yang tercantum untuk melacak plugin atau pesawat yang memicu error.",
  ],
  navOverride: [
    'Back up Custom Data and restore a complete X-Plane navigation package for one AIRAC cycle from your data provider. Check the listed files after reinstalling.',
    "Pasang ulang paket data navigasi (Navigraph/Aerosoft) ke folder Custom Data.",
  ],
  navSupplemental: [
    'Check the source of each listed file. Restore provider files from the matching package and recover user-edited fixes or navaids from your own backup.',
    "Periksa file navigasi tambahan (user_nav/user_fix/FAA) di Custom Data untuk memastikan formatnya valid.",
  ],
  navFuture: [
    'Check the effective dates in the evidence and your system date. Use an already effective cycle for flights scheduled before the new cycle begins.',
    "Sebagian data navigasi memiliki tanggal efektif di masa depan; pastikan tanggal sistem dan siklus sudah sesuai.",
  ],
  navConflict: [
    'Compare the cycle numbers for each affected data source. Restore matching files and metadata from the same package before checking again.',
    "Perbarui semua paket navigasi agar seluruh pesawat menggunakan siklus AIRAC yang sama.",
  ],
  navExpired: [
    'Update the expired sources listed in the evidence if your flight plans or charts require a current cycle. Keep the simulator and relevant aircraft databases aligned.',
    "Data navigasi sudah kedaluwarsa. Perbarui siklus AIRAC terbaru jika terbang online (VATSIM/IVAO).",
  ],
  navExpiring: [
    'Check the listed expiry dates and plan the next data update before a flight that requires the new cycle.',
    "Siklus AIRAC akan segera berakhir. Siapkan pembaruan data navigasi berikutnya.",
  ],
  navUnknown: [
    'Compare the cycle and effective dates with the package’s source information. Restore its metadata from the same package if it is missing or unreadable.',
    "Metadata siklus tidak ditemukan; pastikan file cycle_info.txt tersedia di paket navdata.",
  ],
  appData: [
    'Check space and write access on the DHUB Manager data volume. Resolve the error shown in the evidence before retrying installs or updates.',
    "Periksa izin tulis pada folder data aplikasi DHUB Manager.",
  ],
  appDatabase: [
    'Keep a copy of the application data before making changes. If the problem persists after restarting DHUB Manager, export the health report for support.',
    "Database lokal DHUB Manager bermasalah; coba mulai ulang aplikasi atau bangun ulang indeks di Pengaturan.",
  ],
  sceneryIndex: [
    'Confirm your scenery drives are connected, then refresh or rebuild the scenery index and verify that the expected packages appear.',
    "Buka Pengaturan lalu klik Bangun Ulang Indeks Scenery untuk menyinkronkan database dengan folder disk.",
  ],
  recentFailures: [
    'Open Activity and read the error for the affected installation or update. Resolve that error before retrying the operation.',
    "Tinjau halaman Activity Log untuk melihat detail pemasangan atau pembaruan yang gagal.",
  ],
  addonUpdate: [
    'Review the confirmed updates in Management. Check compatibility with your X-Plane version before installing.',
    "Buka tab Manajemen untuk memperbarui addon ke versi terbaru.",
  ],
  addonManual: [
    'For entries marked as manual upgrades, follow the provider’s migration instructions and back up your current configuration first.',
    "Sebagian addon memerlukan unduhan pembaruan secara manual dari toko/pengembangnya.",
  ],
  addonExternal: [
    'For entries managed by an external updater, check their versions using the updater named in the evidence.',
    "Gunakan updater eksternal resmi (seperti SkunkCrafts/X-Updater) untuk memperbarui addon tersebut.",
  ],
  gatewayUpdate: [
    'In Gateway, review the airport and submission numbers in the evidence before updating. Keep a copy of any local edits you want to preserve.',
    "Buka tab Gateway untuk memperbarui bandara Scenery Gateway ke versi terbaru.",
  ],
  cslUpdate: [
    'Open CSL and review the listed model packages and changed files before applying their updates.',
    "Buka tab X-CSL untuk memperbarui paket model multiplayer.",
  ],
  appUpdate: [
    'Open Settings to review the available DHUB Manager version and its release notes before updating.',
    "Pembaruan DHUB Manager tersedia. Unduh dan pasang versi terbaru.",
  ],
  retryUpdate: [
    'Check the connection and any provider errors in the evidence, then retry the full diagnosis for the entries that could not be verified.',
    "Gagal terhubung ke server pembaruan. Periksa koneksi internet lalu coba lagi nanti.",
  ],
  retryCheck: [
    'Review any error in the evidence and run the check again. If it keeps failing, export the health report with that evidence.',
    "Jalankan ulang pemeriksaan setelah menyelesaikan masalah yang tercantum.",
  ],
  defenderExclusion: [
    'Open Settings and enable Windows Defender Exclusion for your X-Plane folder to prevent false-positive antivirus deletions of aircraft DLLs and plugins.',
    "Buka Pengaturan dan aktifkan Pengecualian Windows Defender untuk folder X-Plane agar antivirus tidak menghapus DLL pesawat atau plugin akibat false positive.",
  ],
} as const
