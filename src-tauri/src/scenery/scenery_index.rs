//! Scenery index management module
//!
//! This module manages a persistent SQLite database of scenery classifications
//! with cache invalidation based on directory modification times.

use crate::database::{SceneryQueries, CURRENT_SCHEMA_VERSION};
use crate::logger;
use crate::management_index::read_version_from_paths;
use crate::models::{
    is_global_airports_folder_name, SceneryCategory, SceneryIndex, SceneryIndexScanResult,
    SceneryIndexStats, SceneryIndexStatus, SceneryManagerData, SceneryManagerEntry,
    SceneryPackageInfo, GLOBAL_AIRPORTS_ENTRY_NAME,
};
use crate::scenery_classifier::{
    calculate_sub_priority, classify_scenery, get_scenery_effective_modified_time,
    is_special_airport_tier_folder_name,
};
use anyhow::{anyhow, Result};
use rayon::prelude::*;
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

type AirportCoords = HashMap<(i32, i32), Vec<(String, Option<String>)>>;

// ============================================================================
// Windows Shortcut Resolution (COM API)
// ============================================================================

#[cfg(windows)]
mod shortcut_resolver {
    use super::*;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::guiddef::GUID;
    use winapi::shared::minwindef::MAX_PATH;
    use winapi::shared::winerror::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx, CoUninitialize};
    use winapi::um::objbase::COINIT_APARTMENTTHREADED;
    use winapi::um::objidl::IPersistFile;
    use winapi::um::shobjidl_core::IShellLinkW;
    use winapi::Interface;

    // CLSID_ShellLink: 00021401-0000-0000-C000-000000000046
    const CLSID_SHELL_LINK: GUID = GUID {
        Data1: 0x00021401,
        Data2: 0x0000,
        Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };

    /// RAII wrapper for COM initialization
    struct ComGuard {
        should_uninit: bool,
    }

    impl ComGuard {
        /// Initialize COM with apartment threading model
        fn new() -> Option<Self> {
            unsafe {
                let hr = CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED);
                if hr == S_OK || hr == S_FALSE {
                    Some(Self {
                        should_uninit: true,
                    })
                } else if hr == RPC_E_CHANGED_MODE {
                    logger::log_info(
                        "  COM already initialized with different threading model",
                        Some("scenery_index"),
                    );
                    Some(Self {
                        should_uninit: false,
                    })
                } else {
                    logger::log_info(
                        &format!("  Failed to initialize COM, HRESULT: 0x{:08X}", hr),
                        Some("scenery_index"),
                    );
                    None
                }
            }
        }
    }

    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.should_uninit {
                unsafe { CoUninitialize() };
            }
        }
    }

    /// RAII wrapper for IShellLinkW COM interface
    struct ShellLinkGuard {
        ptr: *mut IShellLinkW,
    }

    impl ShellLinkGuard {
        fn new() -> Option<Self> {
            unsafe {
                let mut shell_link: *mut IShellLinkW = ptr::null_mut();
                let hr = CoCreateInstance(
                    &CLSID_SHELL_LINK,
                    ptr::null_mut(),
                    1, // CLSCTX_INPROC_SERVER
                    &IShellLinkW::uuidof(),
                    &mut shell_link as *mut *mut _ as *mut *mut _,
                );
                if hr == S_OK && !shell_link.is_null() {
                    Some(Self { ptr: shell_link })
                } else {
                    logger::log_info(
                        &format!("  Failed to create IShellLink, HRESULT: 0x{:08X}", hr),
                        Some("scenery_index"),
                    );
                    None
                }
            }
        }

        fn as_ptr(&self) -> *mut IShellLinkW {
            self.ptr
        }
    }

    impl Drop for ShellLinkGuard {
        fn drop(&mut self) {
            if !self.ptr.is_null() {
                unsafe { (*self.ptr).Release() };
            }
        }
    }

    /// RAII wrapper for IPersistFile COM interface
    struct PersistFileGuard {
        ptr: *mut IPersistFile,
    }

    impl PersistFileGuard {
        fn from_shell_link(shell_link: &ShellLinkGuard) -> Option<Self> {
            unsafe {
                let mut persist_file: *mut IPersistFile = ptr::null_mut();
                let hr = (*shell_link.as_ptr()).QueryInterface(
                    &IPersistFile::uuidof(),
                    &mut persist_file as *mut *mut _ as *mut *mut _,
                );
                if hr == S_OK && !persist_file.is_null() {
                    Some(Self { ptr: persist_file })
                } else {
                    logger::log_info(
                        &format!("  Failed to query IPersistFile, HRESULT: 0x{:08X}", hr),
                        Some("scenery_index"),
                    );
                    None
                }
            }
        }

        fn load(&self, path: &Path) -> bool {
            unsafe {
                let wide_path: Vec<u16> = path
                    .as_os_str()
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let hr = (*self.ptr).Load(wide_path.as_ptr(), 0);
                if hr != S_OK {
                    logger::log_info(
                        &format!("  Failed to load shortcut file, HRESULT: 0x{:08X}", hr),
                        Some("scenery_index"),
                    );
                }
                hr == S_OK
            }
        }
    }

    impl Drop for PersistFileGuard {
        fn drop(&mut self) {
            if !self.ptr.is_null() {
                unsafe { (*self.ptr).Release() };
            }
        }
    }

    /// Get the target path from a loaded shell link
    fn get_shell_link_target(shell_link: &ShellLinkGuard) -> Option<PathBuf> {
        unsafe {
            let mut target_path = vec![0u16; MAX_PATH];
            let hr = (*shell_link.as_ptr()).GetPath(
                target_path.as_mut_ptr(),
                MAX_PATH as i32,
                ptr::null_mut(),
                0,
            );
            if hr == S_OK {
                let len = target_path.iter().position(|&c| c == 0).unwrap_or(MAX_PATH);
                let target_str = String::from_utf16_lossy(&target_path[..len]);
                logger::log_info(
                    &format!("  Shortcut target (COM API): {:?}", target_str),
                    Some("scenery_index"),
                );
                let path = PathBuf::from(target_str);
                if path.exists() && path.is_dir() {
                    return Some(path);
                }
            } else {
                logger::log_info(
                    &format!("  GetPath failed with HRESULT: 0x{:08X}", hr),
                    Some("scenery_index"),
                );
            }
            None
        }
    }

    /// Resolve a Windows shortcut (.lnk) to its target path
    pub fn resolve(lnk_path: &Path) -> Option<PathBuf> {
        let _com = ComGuard::new()?;
        let shell_link = ShellLinkGuard::new()?;
        let persist_file = PersistFileGuard::from_shell_link(&shell_link)?;

        if !persist_file.load(lnk_path) {
            return None;
        }

        get_shell_link_target(&shell_link)
    }
}

/// Resolve Windows shortcut (.lnk) to actual path using Windows COM API
#[cfg(windows)]
pub(crate) fn resolve_shortcut(lnk_path: &Path) -> Option<PathBuf> {
    shortcut_resolver::resolve(lnk_path)
}

#[cfg(not(windows))]
pub(crate) fn resolve_shortcut(_lnk_path: &Path) -> Option<PathBuf> {
    None
}

fn is_sam_folder_name(folder_name: &str) -> bool {
    let folder_lower = folder_name.to_lowercase();

    let parts: Vec<&str> = folder_lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    let has_sam_word = parts.contains(&"sam");
    let has_sam_suffix = parts.iter().any(|&part| {
        part.ends_with("sam") && part.len() > 3 && {
            let prefix = &part[..part.len() - 3];
            matches!(prefix, "open" | "my" | "custom" | "new")
        }
    });

    has_sam_word || has_sam_suffix
}

fn is_lines3d_folder_name(folder_name: &str) -> bool {
    folder_name.trim().eq_ignore_ascii_case("lines3d")
}

fn should_promote_to_fixed_high_priority(folder_name: &str, info: &SceneryPackageInfo) -> bool {
    info.original_category == Some(SceneryCategory::FixedHighPriority)
        || ((is_sam_folder_name(folder_name) || is_lines3d_folder_name(folder_name))
            && info.has_library_txt
            && !info.has_apt_dat)
}

fn read_scenery_update_info(folder_path: &Path) -> (Option<String>, Option<String>) {
    let cfg_path = folder_path.join("skunkcrafts_updater.cfg");
    if !cfg_path.is_file() {
        return (None, None);
    }

    let (version, update_url, _cfg_disabled) = read_version_from_paths(Some(&cfg_path), &[]);
    (version, update_url)
}

/// Common sorting comparison for non-FixedHighPriority scenery packages
/// This ensures consistent ordering between rebuild_index, recalculate_sort_order, and reset_sort_order
fn resolve_package_dir(custom_scenery_path: &Path, info: &SceneryPackageInfo) -> PathBuf {
    if let Some(ref actual_path) = info.actual_path {
        let p = PathBuf::from(actual_path);
        if p.exists() {
            return p;
        }
    }
    custom_scenery_path.join(&info.folder_name)
}

fn extract_airport_icao_key(name: &str, info: &SceneryPackageInfo) -> Option<String> {
    const NOISE_WORDS: &[&str] = &[
        "XP11", "XP12", "MESH", "INTL", "CITY", "ROAD", "ROADS", "SFD", "AERO", "TAIM",
        "PACK", "APTS", "DEMO", "SCEN", "PART", "FLAT", "FREE", "BASE", "FULL",
        "AIRP", "PARK", "HIGH", "WIDE", "DEPO", "APRO", "TERM",
    ];

    if let Some(ref id) = info.airport_id {
        let trimmed = id.trim().to_uppercase();
        if trimmed.len() >= 3 && trimmed.len() <= 5 && !NOISE_WORDS.contains(&trimmed.as_str()) {
            return Some(trimmed);
        }
    }

    let upper = name.to_uppercase();
    let tokens: Vec<&str> = upper
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    for token in tokens {
        if token.len() == 4
            && token.chars().all(|c| c.is_ascii_alphabetic())
            && !NOISE_WORDS.contains(&token)
        {
            return Some(token.to_string());
        }
    }

    None
}

fn associate_companion_airport_packages(packages: &mut [SceneryPackageInfo]) {
    let mut airport_icao_map: Vec<(String, String)> = Vec::new();
    for pkg in packages.iter() {
        if pkg.category == SceneryCategory::Airport {
            if let Some(icao) = extract_airport_icao_key(&pkg.folder_name, pkg) {
                airport_icao_map.push((pkg.folder_name.clone(), icao));
            }
        }
    }

    for pkg in packages.iter_mut() {
        if pkg.category == SceneryCategory::Airport && pkg.airport_id.is_none() {
            let lower = pkg.folder_name.to_lowercase();
            for (known_name, icao) in &airport_icao_map {
                if known_name == &pkg.folder_name {
                    continue;
                }
                if let (Some(p1), Some(p2)) = (
                    extract_scenery_prefix(&pkg.folder_name),
                    extract_scenery_prefix(known_name),
                ) {
                    if p1.eq_ignore_ascii_case(&p2) {
                        pkg.airport_id = Some(icao.clone());
                        break;
                    }
                }
                let known_tokens: Vec<&str> = known_name
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .filter(|s| s.len() >= 4 && !s.chars().all(|c| c.is_ascii_digit()))
                    .collect();
                if known_tokens.iter().any(|tok| lower.contains(&tok.to_lowercase())) {
                    pkg.airport_id = Some(icao.clone());
                    break;
                }
            }
        }
    }
}

fn compare_packages_for_sorting(
    name_a: &str,
    info_a: &SceneryPackageInfo,
    name_b: &str,
    info_b: &SceneryPackageInfo,
) -> std::cmp::Ordering {
    let sub_a = if info_a.category == SceneryCategory::Airport {
        calculate_sub_priority(&info_a.category, name_a)
    } else {
        info_a.sub_priority
    };
    let sub_b = if info_b.category == SceneryCategory::Airport {
        calculate_sub_priority(&info_b.category, name_b)
    } else {
        info_b.sub_priority
    };
    let priority_a = (info_a.category.priority(), sub_a);
    let priority_b = (info_b.category.priority(), sub_b);

    match priority_a.cmp(&priority_b) {
        std::cmp::Ordering::Equal => {
            if info_a.category == SceneryCategory::FixedHighPriority
                && info_b.category == SceneryCategory::FixedHighPriority
            {
                // Keep Lines3D at the absolute top of FixedHighPriority.
                let lines3d_a = is_lines3d_folder_name(name_a);
                let lines3d_b = is_lines3d_folder_name(name_b);
                match lines3d_b.cmp(&lines3d_a) {
                    std::cmp::Ordering::Equal => {}
                    other => return other,
                }

                // Keep SAM entries near the top, after Lines3D.
                let sam_a = is_sam_folder_name(name_a);
                let sam_b = is_sam_folder_name(name_b);
                match sam_b.cmp(&sam_a) {
                    std::cmp::Ordering::Equal => {}
                    other => return other,
                }
            }

            if info_a.category == info_b.category
                && matches!(
                    info_a.category,
                    SceneryCategory::AirportMesh | SceneryCategory::Mesh
                )
            {
                // For Mesh category with sub_priority > 0 (XPME), sort only by folder name
                // XPME mesh should be at the bottom of Mesh category, sorted alphabetically
                if info_a.category == SceneryCategory::Mesh && info_a.sub_priority > 0 {
                    name_a.to_lowercase().cmp(&name_b.to_lowercase())
                } else {
                    // Non-XPME: sort by tile count first, then folder name
                    match info_a
                        .earth_nav_tile_count
                        .cmp(&info_b.earth_nav_tile_count)
                    {
                        std::cmp::Ordering::Equal => {
                            name_a.to_lowercase().cmp(&name_b.to_lowercase())
                        }
                        other => other,
                    }
                }
            } else if info_a.category == SceneryCategory::Airport
                && info_b.category == SceneryCategory::Airport
                && sub_a == 0
                && sub_b == 0
            {
                let icao_a = extract_airport_icao_key(name_a, info_a);
                let icao_b = extract_airport_icao_key(name_b, info_b);

                match (&icao_a, &icao_b) {
                    (Some(a), Some(b)) => match a.cmp(b) {
                        std::cmp::Ordering::Equal => {
                            name_a.to_lowercase().cmp(&name_b.to_lowercase())
                        }
                        other => other,
                    },
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => name_a.to_lowercase().cmp(&name_b.to_lowercase()),
                }
            } else {
                name_a.to_lowercase().cmp(&name_b.to_lowercase())
            }
        }
        other => other,
    }
}

fn compare_fixed_high_priority_packages(
    name_a: &str,
    info_a: &SceneryPackageInfo,
    name_b: &str,
    info_b: &SceneryPackageInfo,
) -> std::cmp::Ordering {
    // Keep Lines3D at the absolute top of FixedHighPriority.
    let lines3d_a = is_lines3d_folder_name(name_a);
    let lines3d_b = is_lines3d_folder_name(name_b);
    match lines3d_b.cmp(&lines3d_a) {
        std::cmp::Ordering::Equal => {}
        other => return other,
    }

    // Keep SAM entries near the top, after Lines3D.
    let sam_a = is_sam_folder_name(name_a);
    let sam_b = is_sam_folder_name(name_b);
    match sam_b.cmp(&sam_a) {
        std::cmp::Ordering::Equal => {}
        other => return other,
    }

    match info_a.sort_order.cmp(&info_b.sort_order) {
        std::cmp::Ordering::Equal => name_a.to_lowercase().cmp(&name_b.to_lowercase()),
        other => other,
    }
}

fn is_darkblue_folder_name(folder_name: &str) -> bool {
    folder_name
        .get(..9)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("DarkBlue-"))
}

fn is_darkblue_package_linked_to_airport_by_prefix(
    package_folder_name: &str,
    airport_folder_name: &str,
) -> bool {
    let package_lower = package_folder_name.to_lowercase();
    let airport_lower = airport_folder_name.to_lowercase();
    if !package_lower.starts_with(&airport_lower) {
        return false;
    }

    let suffix = &package_folder_name[airport_folder_name.len()..];
    suffix.starts_with('_') || suffix.starts_with('-')
}

fn find_darkblue_airport_folder_for_package(
    package_folder_name: &str,
    airport_folder_names: &[String],
) -> Option<String> {
    let mut best_match: Option<String> = None;

    for airport_folder_name in airport_folder_names {
        if !is_darkblue_package_linked_to_airport_by_prefix(
            package_folder_name,
            airport_folder_name,
        ) {
            continue;
        }

        let should_replace = match best_match.as_ref() {
            Some(current) => airport_folder_name.len() > current.len(),
            None => true,
        };

        if should_replace {
            best_match = Some(airport_folder_name.clone());
        }
    }

    best_match
}

fn apply_darkblue_airport_package_anchors(
    packages: &mut Vec<SceneryPackageInfo>,
    airport_mesh_matches: &HashMap<String, String>,
) {
    if packages.is_empty() {
        return;
    }

    let ordered_names: Vec<String> = packages.iter().map(|pkg| pkg.folder_name.clone()).collect();
    let darkblue_airport_folders: Vec<String> = packages
        .iter()
        .filter(|pkg| pkg.category == SceneryCategory::Airport)
        .filter(|pkg| is_darkblue_folder_name(&pkg.folder_name))
        .map(|pkg| pkg.folder_name.clone())
        .collect();
    let available_names: HashSet<String> = ordered_names.iter().cloned().collect();
    let mut anchors_by_airport: HashMap<String, Vec<String>> = HashMap::new();
    let mut anchored_names: HashSet<String> = HashSet::new();

    for pkg in packages.iter() {
        if !matches!(
            pkg.category,
            SceneryCategory::Overlay | SceneryCategory::AirportMesh
        ) || !is_darkblue_folder_name(&pkg.folder_name)
        {
            continue;
        }

        let airport_folder = airport_mesh_matches
            .get(&pkg.folder_name)
            .cloned()
            .or_else(|| {
                find_darkblue_airport_folder_for_package(
                    &pkg.folder_name,
                    &darkblue_airport_folders,
                )
            });

        let Some(airport_folder) = airport_folder else {
            continue;
        };
        if !available_names.contains(&airport_folder) {
            continue;
        }

        anchors_by_airport
            .entry(airport_folder)
            .or_default()
            .push(pkg.folder_name.clone());
        anchored_names.insert(pkg.folder_name.clone());
    }

    if anchored_names.is_empty() {
        return;
    }

    let mut package_map: HashMap<String, SceneryPackageInfo> = packages
        .drain(..)
        .map(|pkg| (pkg.folder_name.clone(), pkg))
        .collect();
    let mut reordered = Vec::with_capacity(ordered_names.len());

    for name in &ordered_names {
        if anchored_names.contains(name) {
            continue;
        }

        if let Some(pkg) = package_map.remove(name) {
            reordered.push(pkg);
        }

        if let Some(anchored) = anchors_by_airport.get(name) {
            for anchored_name in anchored {
                if let Some(pkg) = package_map.remove(anchored_name) {
                    reordered.push(pkg);
                }
            }
        }
    }

    // Any unmatched leftovers keep their original relative order.
    for name in &ordered_names {
        if let Some(pkg) = package_map.remove(name) {
            reordered.push(pkg);
        }
    }

    *packages = reordered;
}

fn apply_airport_mesh_matches(
    packages: &mut [SceneryPackageInfo],
    airport_mesh_matches: &HashMap<String, String>,
) -> bool {
    let mut category_changed = false;

    for pkg in packages.iter_mut() {
        if pkg.category == SceneryCategory::Mesh
            && airport_mesh_matches.contains_key(&pkg.folder_name)
        {
            pkg.category = SceneryCategory::AirportMesh;
            category_changed = true;
        }
    }

    category_changed
}

fn sort_packages_with_special_rules(
    xplane_path: &Path,
    packages: &mut Vec<SceneryPackageInfo>,
) -> bool {
    associate_companion_airport_packages(packages);
    let airport_mesh_matches = detect_airport_mesh_matches_with_path(xplane_path, packages);
    let category_changed = apply_airport_mesh_matches(packages, &airport_mesh_matches);

    let mut fixed_packages = Vec::new();
    let mut other_packages = Vec::new();

    for pkg in packages.drain(..) {
        if pkg.category == SceneryCategory::FixedHighPriority {
            fixed_packages.push(pkg);
        } else {
            other_packages.push(pkg);
        }
    }

    fixed_packages
        .sort_by(|a, b| compare_fixed_high_priority_packages(&a.folder_name, a, &b.folder_name, b));
    other_packages
        .sort_by(|a, b| compare_packages_for_sorting(&a.folder_name, a, &b.folder_name, b));

    fixed_packages.extend(other_packages);
    apply_darkblue_airport_package_anchors(&mut fixed_packages, &airport_mesh_matches);
    *packages = fixed_packages;

    category_changed
}

/// Manager for scenery index operations
pub struct SceneryIndexManager {
    xplane_path: PathBuf,
    db: DatabaseConnection,
}

impl SceneryIndexManager {
    /// Create a new index manager
    pub fn new(xplane_path: &Path, db: DatabaseConnection) -> Self {
        Self {
            xplane_path: xplane_path.to_path_buf(),
            db,
        }
    }

    /// Check if the scenery index has been created (has any packages)
    /// Returns true if there are packages in the index, false otherwise
    pub async fn has_index(&self) -> Result<bool> {
        SceneryQueries::has_packages(&self.db)
            .await
            .map_err(|e| anyhow!("{}", e))
    }

    /// Load index from database or create new empty index
    pub async fn load_index(&self) -> Result<SceneryIndex> {
        // Check if database has any packages
        let has_packages = SceneryQueries::has_packages(&self.db)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        if has_packages {
            SceneryQueries::load_all(&self.db)
                .await
                .map_err(|e| anyhow!("{}", e))
        } else {
            Ok(self.create_empty_index())
        }
    }

    /// Save index to database
    pub async fn save_index(&self, index: &SceneryIndex) -> Result<()> {
        SceneryQueries::save_all(&self.db, index)
            .await
            .map_err(|e| anyhow!("{}", e))
    }

    /// Update or add a single package in the index
    pub async fn update_package(&self, package_info: SceneryPackageInfo) -> Result<()> {
        SceneryQueries::update_package(&self.db, &package_info)
            .await
            .map_err(|e| anyhow!("{}", e))
    }

    /// Rebuild entire index by scanning all scenery packages
    /// This completely clears the existing index and rebuilds from scratch
    pub async fn rebuild_index(&self) -> Result<SceneryIndex> {
        let custom_scenery_path = self.xplane_path.join("Custom Scenery");
        if !custom_scenery_path.exists() {
            return Err(anyhow!("Custom Scenery folder not found"));
        }

        // Clear all existing index data for a fresh rebuild
        SceneryQueries::clear_all(&self.db)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        logger::log_info(
            "Cleared existing index, starting fresh rebuild",
            Some("scenery_index"),
        );

        let xplane_path = self.xplane_path.clone();
        let custom_scenery_path = custom_scenery_path.clone();

        let index = tokio::task::spawn_blocking(move || -> Result<SceneryIndex> {
            // Collect all scenery folders (including symlinks and .lnk shortcuts)
            // Track shortcuts by their target path to correctly map shortcut names
            // Key: canonical target path, Value: (shortcut_name without .lnk, normalized_target_path for ini)
            let mut shortcut_target_map: HashMap<PathBuf, (String, String)> = HashMap::new();

            let scenery_folders: Vec<PathBuf> = fs::read_dir(&custom_scenery_path)?
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();

                    // Check if it's a .lnk file (Windows shortcut)
                    if path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
                    {
                        // Use shortcut file name (without .lnk extension) as the entry name
                        // This prevents conflicts when multiple shortcuts point to folders with the same name
                        let shortcut_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "<unknown>".to_string());
                        logger::log_info(
                            &format!("Attempting to resolve shortcut: {}.lnk", shortcut_name),
                            Some("scenery_index"),
                        );

                        // Try to resolve the shortcut
                        if let Some(target) = resolve_shortcut(&path) {
                            logger::log_info(
                                &format!(
                                    "✓ Resolved shortcut {}.lnk -> {:?}",
                                    shortcut_name, target
                                ),
                                Some("scenery_index"),
                            );

                            // Store the mapping from target path to shortcut info
                            let target_path_str = target.to_string_lossy().to_string();
                            // Convert backslashes to forward slashes for scenery_packs.ini compatibility
                            let normalized_path_str = target_path_str.replace('\\', "/");
                            shortcut_target_map
                                .insert(target.clone(), (shortcut_name, normalized_path_str));

                            return Some(target);
                        } else {
                            logger::log_info(
                                &format!("✗ Failed to resolve shortcut: {:?}", path),
                                Some("scenery_index"),
                            );
                            return None;
                        }
                    }

                    // Check if it's a directory (including symlinks)
                    if path.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                        return Some(path);
                    }

                    None
                })
                .collect();

            logger::log_info(
                &format!(
                    "Rebuilding scenery index for {} packages",
                    scenery_folders.len()
                ),
                Some("scenery_index"),
            );

            // Always classify all packages in parallel across all CPU cores via Rayon
            let packages_with_paths: Vec<(PathBuf, SceneryPackageInfo)> = scenery_folders
                .par_iter()
                .filter_map(|folder| match classify_scenery(folder, &xplane_path) {
                    Ok(info) => Some((folder.clone(), info)),
                    Err(e) => {
                        logger::log_info(
                            &format!("Failed to classify {:?}: {}", folder, e),
                            Some("scenery_index"),
                        );
                        None
                    }
                })
                .collect();

            // Post-process: Set folder_name and actual_path for shortcut entries
            // For shortcuts, use the shortcut name (not target folder name) to avoid conflicts
            let mut packages_vec: Vec<SceneryPackageInfo> =
                Vec::with_capacity(packages_with_paths.len());
            for (path, mut info) in packages_with_paths {
                if let Some((shortcut_name, actual_path)) = shortcut_target_map.get(&path) {
                    // This entry came from a shortcut - use shortcut name and set actual_path
                    logger::log_info(
                        &format!(
                            "Shortcut entry: {} (target folder: {}) -> actual_path: {}",
                            shortcut_name, info.folder_name, actual_path
                        ),
                        Some("scenery_index"),
                    );
                    info.folder_name = shortcut_name.clone();
                    info.actual_path = Some(actual_path.clone());
                }
                packages_vec.push(info);
            }

            // Apply shared post-processing and sorting rules.
            sort_packages_with_special_rules(&xplane_path, &mut packages_vec);

            // Assign sort_order and set default enabled state
            // Fresh rebuild: Unrecognized packages default to disabled, others default to enabled
            let packages: HashMap<String, SceneryPackageInfo> = packages_vec
                .into_iter()
                .enumerate()
                .map(|(index, mut info)| {
                    info.sort_order = index as u32;
                    // Unrecognized packages default to disabled, others default to enabled
                    info.enabled = info.category != SceneryCategory::Unrecognized;
                    (info.folder_name.clone(), info)
                })
                .collect();

            let mut built_index = SceneryIndex {
                version: CURRENT_SCHEMA_VERSION as u32,
                packages,
                last_updated: SystemTime::now(),
            };

            // Compute missing libraries in parallel in memory BEFORE saving so we write SQLite only once!
            recompute_missing_libraries_in_memory(&xplane_path, &mut built_index);

            Ok(built_index)
        })
        .await
        .map_err(|e| anyhow!("Blocking task failed: {}", e))??;

        self.save_index(&index).await?;
        logger::log_info(
            &format!(
                "Scenery index rebuilt with {} packages (missing libraries resolved in single pass)",
                index.packages.len()
            ),
            Some("scenery_index"),
        );

        Ok(index)
    }

    /// Recalculate sort_order for all packages using the same sorting logic as rebuild_index
    /// This ensures incremental updates produce consistent ordering with full rebuilds
    fn recalculate_sort_order(index: &mut SceneryIndex, xplane_path: &Path) {
        if index.packages.is_empty() {
            return;
        }

        // Promote special high-priority libraries (Lines3D, SAM) and special Airport tier packages before sorting.
        for (name, info) in index.packages.iter_mut() {
            if is_global_airports_folder_name(name)
                && (info.category != SceneryCategory::DefaultAirport || info.sub_priority != 0)
            {
                info.category = SceneryCategory::DefaultAirport;
                info.sub_priority = 0;
            } else if should_promote_to_fixed_high_priority(name, info)
                && info.category != SceneryCategory::FixedHighPriority
            {
                info.category = SceneryCategory::FixedHighPriority;
                info.sub_priority = 0;
            } else if is_special_airport_tier_folder_name(name) {
                info.category = SceneryCategory::Airport;
                info.original_category = Some(SceneryCategory::Airport);
            }

            info.sub_priority = calculate_sub_priority(&info.category, name);
        }

        let mut packages_vec: Vec<SceneryPackageInfo> =
            index.packages.drain().map(|(_, v)| v).collect();
        sort_packages_with_special_rules(xplane_path, &mut packages_vec);
        let package_count = packages_vec.len();

        for (new_order, mut info) in packages_vec.into_iter().enumerate() {
            info.sort_order = new_order as u32;
            index.packages.insert(info.folder_name.clone(), info);
        }

        logger::log_info(
            &format!("Recalculated sort order for {} packages", package_count),
            Some("scenery_index"),
        );
    }

    /// Update index incrementally - only re-classify modified packages
    pub async fn update_index(&self) -> Result<SceneryIndex> {
        self.update_index_with_locked_entries(Vec::new()).await
    }

    /// Update index incrementally while preserving locked scenery sort positions.
    pub async fn update_index_with_locked_entries(
        &self,
        locked_folder_names: Vec<String>,
    ) -> Result<SceneryIndex> {
        let custom_scenery_path = self.xplane_path.join("Custom Scenery");
        if !custom_scenery_path.exists() {
            return Err(anyhow!("Custom Scenery folder not found"));
        }

        let index = self.load_index().await?;
        let xplane_path = self.xplane_path.clone();
        let custom_scenery_path = custom_scenery_path.clone();
        let locked_folder_names_for_sort = locked_folder_names.clone();
        let locked_name_set: HashSet<String> = locked_folder_names
            .iter()
            .map(|name| name.trim().to_lowercase())
            .filter(|name| !name.is_empty())
            .collect();

        let (index, should_save) = tokio::task::spawn_blocking(move || -> Result<(SceneryIndex, bool)> {
            let mut index = index;
            let mut changed = false;

            // Track shortcuts by their target path
            // Key: target path, Value: (shortcut_name without .lnk, normalized_target_path for ini)
            let mut shortcut_target_map: HashMap<PathBuf, (String, String)> = HashMap::new();

            // Get current scenery folders (including symlinks and .lnk shortcuts)
            // Key: entry name (shortcut name for shortcuts, folder name for directories)
            // Value: actual path to scan
            let current_folders: HashMap<String, PathBuf> = fs::read_dir(&custom_scenery_path)?
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();

                    // Check if it's a .lnk file (Windows shortcut)
                    if path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
                    {
                        // Try to resolve the shortcut
                        if let Some(target) = resolve_shortcut(&path) {
                            // Use shortcut file name (without .lnk) as the entry name
                            // This prevents conflicts when multiple shortcuts point to folders with the same name
                            let shortcut_name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "<unknown>".to_string());
                            // Track the resolved target path for writing to scenery_packs.ini
                            let target_path_str = target.to_string_lossy().to_string();
                            // Convert backslashes to forward slashes for scenery_packs.ini compatibility
                            let normalized_path_str = target_path_str.replace('\\', "/");
                            shortcut_target_map.insert(
                                target.clone(),
                                (shortcut_name.clone(), normalized_path_str),
                            );
                            return Some((shortcut_name, target));
                        }
                        return None;
                    }

                    // Check if it's a directory (including symlinks)
                    if path.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                        if let Some(name) = e.file_name().to_str() {
                            return Some((name.to_string(), path));
                        }
                    }

                    None
                })
                .collect();

            // Remove stale entries (deleted folders)
            let stale_keys: Vec<String> = index
                .packages
                .keys()
                .filter(|name| !current_folders.contains_key(*name))
                .cloned()
                .collect();

            if !stale_keys.is_empty() {
                changed = true;
            }
            for key in stale_keys {
                index.packages.remove(&key);
                crate::log_debug!(&format!("Removed stale entry: {}", key), "scenery_index");
            }

            // Find packages that need updating in parallel using deep effective modification times
            let current_entries_vec: Vec<(&String, &PathBuf)> = current_folders.iter().collect();
            let packages_to_update: Vec<PathBuf> = current_entries_vec
                .par_iter()
                .filter(|(name, path)| {
                    if name.starts_with("XPME_") {
                        return !index.packages.contains_key(*name);
                    }

                    if let Some(existing) = index.packages.get(*name) {
                        if let Ok(modified) = get_scenery_effective_modified_time(path) {
                            let mod_secs = modified
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let prev_mod_secs = existing
                                .last_modified
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let idx_secs = existing
                                .indexed_at
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            return mod_secs > idx_secs || mod_secs != prev_mod_secs;
                        }
                        false
                    } else {
                        true
                    }
                })
                .map(|(_, path)| (*path).clone())
                .collect();

            if !packages_to_update.is_empty() {
                changed = true;
                logger::log_info(
                    &format!("Updating {} scenery packages", packages_to_update.len()),
                    Some("scenery_index"),
                );

                // Keep current sort slots for locked entries before recalculation.
                let locked_anchor_orders: HashMap<String, usize> = index
                    .packages
                    .iter()
                    .filter(|(name, _)| locked_name_set.contains(&name.to_lowercase()))
                    .map(|(name, info)| (name.to_lowercase(), info.sort_order as usize))
                    .collect();

                // Always classify updated packages in parallel across CPU cores
                let packages_with_paths: Vec<(PathBuf, SceneryPackageInfo)> = packages_to_update
                    .par_iter()
                    .filter_map(|folder| {
                        classify_scenery(folder, &xplane_path)
                            .ok()
                            .map(|info| (folder.clone(), info))
                    })
                    .collect();

                for (path, mut info) in packages_with_paths {
                    // Check if this entry came from a shortcut
                    if let Some((shortcut_name, actual_path)) = shortcut_target_map.get(&path) {
                        // Use shortcut name and set actual_path
                        info.folder_name = shortcut_name.clone();
                        info.actual_path = Some(actual_path.clone());
                    }
                    index.packages.insert(info.folder_name.clone(), info);
                }

                // After adding new packages, recalculate sort_order using the same logic as rebuild_index
                // This ensures incremental updates produce the same ordering as full rebuilds
                Self::recalculate_sort_order(&mut index, &xplane_path);

                if !locked_anchor_orders.is_empty() {
                    let mut sorted_entries: Vec<(&String, &SceneryPackageInfo)> =
                        index.packages.iter().collect();
                    sorted_entries.sort_by(|(name_a, info_a), (name_b, info_b)| {
                        info_a
                            .sort_order
                            .cmp(&info_b.sort_order)
                            .then_with(|| name_a.to_lowercase().cmp(&name_b.to_lowercase()))
                    });
                    let sorted_names: Vec<String> = sorted_entries
                        .into_iter()
                        .map(|(name, _)| name.clone())
                        .collect();

                    let (_, locked_applied) = Self::apply_sort_order_with_locked_slots(
                        &mut index,
                        &sorted_names,
                        &locked_folder_names_for_sort,
                        Some(&locked_anchor_orders),
                    );
                    if locked_applied > 0 {
                        logger::log_info(
                            &format!(
                                "Incremental update preserved {} locked scenery positions",
                                locked_applied
                            ),
                            Some("scenery_index"),
                        );
                    }
                }
            }

            // Also update actual_path for existing entries that are shortcuts
            // (in case they weren't updated but the shortcut info needs to be preserved)
            for (folder_name, info) in index.packages.iter_mut() {
                // Look up by folder_name in shortcut_target_map values (shortcut names)
                for (_, (shortcut_name, actual_path)) in shortcut_target_map.iter() {
                    if folder_name == shortcut_name {
                        if info.actual_path.is_none()
                            || info.actual_path.as_ref() != Some(actual_path)
                        {
                            info.actual_path = Some(actual_path.clone());
                            changed = true;
                        }
                        break;
                    }
                }
            }

            // Migrate any packages whose sub_priority or special Airport tier category is outdated
            let mut sub_priority_migrated = false;
            for (name, info) in index.packages.iter_mut() {
                if !locked_name_set.contains(&name.to_lowercase())
                    && is_special_airport_tier_folder_name(name)
                    && (info.category != SceneryCategory::Airport
                        || info.original_category != Some(SceneryCategory::Airport))
                {
                    info.category = SceneryCategory::Airport;
                    info.original_category = Some(SceneryCategory::Airport);
                    sub_priority_migrated = true;
                }
                let expected_sub = calculate_sub_priority(&info.category, name);
                if info.sub_priority != expected_sub {
                    info.sub_priority = expected_sub;
                    sub_priority_migrated = true;
                }
            }

            if sub_priority_migrated {
                let locked_anchor_orders: HashMap<String, usize> = index
                    .packages
                    .iter()
                    .filter(|(name, _)| locked_name_set.contains(&name.to_lowercase()))
                    .map(|(name, info)| (name.to_lowercase(), info.sort_order as usize))
                    .collect();

                Self::recalculate_sort_order(&mut index, &xplane_path);

                if !locked_anchor_orders.is_empty() {
                    let mut sorted_entries: Vec<(&String, &SceneryPackageInfo)> =
                        index.packages.iter().collect();
                    sorted_entries.sort_by(|(name_a, info_a), (name_b, info_b)| {
                        info_a
                            .sort_order
                            .cmp(&info_b.sort_order)
                            .then_with(|| name_a.to_lowercase().cmp(&name_b.to_lowercase()))
                    });
                    let sorted_names: Vec<String> = sorted_entries
                        .into_iter()
                        .map(|(name, _)| name.clone())
                        .collect();
                    let _ = Self::apply_sort_order_with_locked_slots(
                        &mut index,
                        &sorted_names,
                        &locked_folder_names_for_sort,
                        Some(&locked_anchor_orders),
                    );
                }
                changed = true;
            }

            if recompute_missing_libraries_in_memory(&xplane_path, &mut index) {
                changed = true;
            }

            if changed {
                index.last_updated = SystemTime::now();
            }

            Ok((index, changed))
        })
        .await
        .map_err(|e| anyhow!("Blocking task failed: {}", e))??;

        if should_save {
            self.save_index(&index).await?;
            logger::log_info(
                "Saved updated scenery index and missing libraries",
                Some("scenery_index"),
            );
        }

        Ok(index)
    }

    /// Check if a package needs re-classification
    pub async fn is_package_stale(&self, folder_name: &str, folder_path: &Path) -> Result<bool> {
        let index = self.load_index().await?;
        let folder_name = folder_name.to_string();
        let folder_path = folder_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            if let Some(existing) = index.packages.get(&folder_name) {
                if let Ok(modified) = get_scenery_effective_modified_time(&folder_path) {
                    let mod_secs = modified
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let prev_mod_secs = existing
                        .last_modified
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let idx_secs = existing
                        .indexed_at
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    return Ok(mod_secs > idx_secs || mod_secs != prev_mod_secs);
                }
            }

            Ok(true) // Assume stale if we can't determine
        })
        .await
        .map_err(|e| anyhow!("Blocking task failed: {}", e))?
    }

    /// Get package info from index
    pub async fn get_package(&self, folder_name: &str) -> Result<Option<SceneryPackageInfo>> {
        let index = self.load_index().await?;
        Ok(index.packages.get(folder_name).cloned())
    }

    /// Get or classify a package (uses cache if available and not stale)
    pub async fn get_or_classify(&self, folder_path: &Path) -> Result<SceneryPackageInfo> {
        let folder_name = folder_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid folder name"))?;

        // Check if we have a valid cached entry
        if !self.is_package_stale(folder_name, folder_path).await? {
            if let Some(info) = self.get_package(folder_name).await? {
                return Ok(info);
            }
        }

        // Classify and update index
        let folder_path = folder_path.to_path_buf();
        let xplane_path = self.xplane_path.clone();
        let info =
            tokio::task::spawn_blocking(move || classify_scenery(&folder_path, &xplane_path))
                .await
                .map_err(|e| anyhow!("Blocking task failed: {}", e))??;
        self.update_package(info.clone()).await?;
        Ok(info)
    }

    pub async fn index_status(&self) -> Result<SceneryIndexStatus> {
        let total_packages = SceneryQueries::get_package_count(&self.db)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        let index_exists = total_packages > 0;

        Ok(SceneryIndexStatus {
            index_exists,
            total_packages,
        })
    }

    pub async fn quick_scan_and_update(&self) -> Result<SceneryIndexScanResult> {
        self.quick_scan_and_update_with_locked_entries(Vec::new())
            .await
    }

    pub async fn quick_scan_and_update_with_locked_entries(
        &self,
        locked_folder_names: Vec<String>,
    ) -> Result<SceneryIndexScanResult> {
        let has_packages = SceneryQueries::has_packages(&self.db)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        if !has_packages {
            return Ok(SceneryIndexScanResult {
                index_exists: false,
                added: Vec::new(),
                removed: Vec::new(),
                updated: Vec::new(),
            });
        }

        let before_index = self.load_index().await?;
        let before_keys: HashSet<String> = before_index.packages.keys().cloned().collect();

        let after_index = self
            .update_index_with_locked_entries(locked_folder_names)
            .await?;
        let after_keys: HashSet<String> = after_index.packages.keys().cloned().collect();

        let mut added: Vec<String> = after_keys.difference(&before_keys).cloned().collect();
        added.sort();
        let mut removed: Vec<String> = before_keys.difference(&after_keys).cloned().collect();
        removed.sort();

        // For "updated", compare actual content changes (not just indexed_at timestamp)
        // Some folders get re-indexed every time due to mtime changes from antivirus/sync tools,
        // but their actual classification hasn't changed - don't report those as updates.
        let mut updated: Vec<String> = after_index
            .packages
            .iter()
            .filter(|(name, info)| {
                // Must exist in both before and after (not added/removed)
                if let Some(before_info) = before_index.packages.get(*name) {
                    // Only count as updated if indexed_at changed AND actual content differs
                    info.indexed_at > before_info.indexed_at
                        && (info.category != before_info.category
                            || info.sub_priority != before_info.sub_priority
                            || info.enabled != before_info.enabled
                            || info.has_apt_dat != before_info.has_apt_dat
                            || info.has_dsf != before_info.has_dsf
                            || info.has_library_txt != before_info.has_library_txt
                            || info.missing_libraries != before_info.missing_libraries
                            || info.exported_library_names != before_info.exported_library_names
                            || info.earth_nav_tile_count != before_info.earth_nav_tile_count)
                } else {
                    false
                }
            })
            .map(|(name, _)| name.clone())
            .collect();
        updated.sort();

        Ok(SceneryIndexScanResult {
            index_exists: true,
            added,
            removed,
            updated,
        })
    }

    /// Get index statistics
    pub async fn get_stats(&self) -> Result<SceneryIndexStats> {
        let index = self.load_index().await?;

        let mut by_category: HashMap<String, usize> = HashMap::new();
        for info in index.packages.values() {
            let category_name = format!("{:?}", info.category);
            *by_category.entry(category_name).or_insert(0) += 1;
        }

        Ok(SceneryIndexStats {
            total_packages: index.packages.len(),
            by_category,
            last_updated: index.last_updated,
        })
    }

    /// Batch update multiple entries' enabled state and sort_order from UI
    pub async fn batch_update_entries(
        &self,
        entries: &[crate::models::SceneryEntryUpdate],
    ) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        SceneryQueries::batch_update_entries(&self.db, entries)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        logger::log_info(
            &format!("Batch updated {} entries in scenery index", entries.len()),
            Some("scenery_index"),
        );

        Ok(())
    }

    /// Update a single entry's enabled state, sort_order, and/or category
    pub async fn update_entry(
        &self,
        folder_name: &str,
        enabled: Option<bool>,
        sort_order: Option<u32>,
        category: Option<SceneryCategory>,
    ) -> Result<()> {
        SceneryQueries::update_entry(
            &self.db,
            folder_name,
            enabled,
            sort_order,
            category.as_ref(),
        )
        .await
        .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    /// Remove an entry from the index
    pub async fn remove_entry(&self, folder_name: &str) -> Result<()> {
        let deleted = SceneryQueries::delete_package(&self.db, folder_name)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        if deleted {
            logger::log_info(
                &format!("Removed entry from scenery index: {}", folder_name),
                Some("scenery_index"),
            );
        }

        Ok(())
    }

    /// Move an entry from one position to another, auto-adjusting other entries
    pub async fn move_entry(&self, folder_name: &str, new_sort_order: u32) -> Result<()> {
        let mut index = self.load_index().await?;

        // Get current sort_order
        let current_sort_order = match index.packages.get(folder_name) {
            Some(info) => info.sort_order,
            None => return Err(anyhow!("Package not found: {}", folder_name)),
        };

        // Validate and clamp new_sort_order to valid range [0, packages.len() - 1]
        let max_valid_order = index.packages.len().saturating_sub(1) as u32;
        let new_sort_order = new_sort_order.min(max_valid_order);

        if current_sort_order == new_sort_order {
            return Ok(()); // No change needed
        }

        // Adjust sort_orders of other entries
        if new_sort_order < current_sort_order {
            // Moving up: increment sort_order of entries in between
            for info in index.packages.values_mut() {
                if info.sort_order >= new_sort_order && info.sort_order < current_sort_order {
                    info.sort_order += 1;
                }
            }
        } else {
            // Moving down: decrement sort_order of entries in between
            for info in index.packages.values_mut() {
                if info.sort_order > current_sort_order && info.sort_order <= new_sort_order {
                    info.sort_order -= 1;
                }
            }
        }

        // Set the new sort_order for the moved entry
        if let Some(info) = index.packages.get_mut(folder_name) {
            info.sort_order = new_sort_order;
        }

        index.last_updated = SystemTime::now();
        self.save_index(&index).await?;

        Ok(())
    }

    /// Reset sort_order for all packages based on category priority
    /// This recalculates the sort order using the classification algorithm
    /// without writing to the ini file
    /// Returns true if the sort order was changed, false if it was already correct
    pub async fn reset_sort_order(&self) -> Result<bool> {
        self.reset_sort_order_with_locked_entries(Vec::new()).await
    }

    /// Reset sort_order while preserving the current sort slots of locked scenery entries.
    /// Locked entries are identified by folder name (case-insensitive).
    pub async fn reset_sort_order_with_locked_entries(
        &self,
        locked_folder_names: Vec<String>,
    ) -> Result<bool> {
        let mut index = self.load_index().await?;

        if index.packages.is_empty() {
            return Ok(false);
        }

        let locked_name_set: HashSet<String> = locked_folder_names
            .iter()
            .map(|name| name.trim().to_lowercase())
            .filter(|name| !name.is_empty())
            .collect();

        // Restore original category for unlocked entries and promote special packages before sorting.
        let mut category_changed = false;
        for (name, info) in index.packages.iter_mut() {
            let is_locked = locked_name_set.contains(&name.to_lowercase());

            if !is_locked {
                if let Some(ref orig_cat) = info.original_category {
                    if info.category != *orig_cat {
                        info.category = orig_cat.clone();
                        category_changed = true;
                    }
                }
            }

            if is_global_airports_folder_name(name)
                && (info.category != SceneryCategory::DefaultAirport || info.sub_priority != 0)
            {
                info.category = SceneryCategory::DefaultAirport;
                info.sub_priority = 0;
                category_changed = true;
            } else if should_promote_to_fixed_high_priority(name, info)
                && info.category != SceneryCategory::FixedHighPriority
            {
                info.category = SceneryCategory::FixedHighPriority;
                info.sub_priority = 0;
                category_changed = true;
            } else if !is_locked && is_special_airport_tier_folder_name(name) {
                if info.category != SceneryCategory::Airport {
                    info.category = SceneryCategory::Airport;
                    category_changed = true;
                }
                info.original_category = Some(SceneryCategory::Airport);
            }

            let expected_sub = calculate_sub_priority(&info.category, name);
            if info.sub_priority != expected_sub {
                info.sub_priority = expected_sub;
                category_changed = true;
            }
        }

        let mut packages_vec: Vec<SceneryPackageInfo> =
            index.packages.drain().map(|(_, v)| v).collect();
        let airport_mesh_category_changed =
            sort_packages_with_special_rules(&self.xplane_path, &mut packages_vec);
        let sorted_names: Vec<String> = packages_vec
            .iter()
            .map(|info| info.folder_name.clone())
            .collect();
        index.packages = packages_vec
            .into_iter()
            .map(|info| (info.folder_name.clone(), info))
            .collect();
        let (sort_changed, locked_applied) = Self::apply_sort_order_with_locked_slots(
            &mut index,
            &sorted_names,
            &locked_folder_names,
            None,
        );
        let has_changes = category_changed || airport_mesh_category_changed || sort_changed;

        if has_changes {
            index.last_updated = SystemTime::now();
            self.save_index(&index).await?;

            if locked_applied > 0 {
                logger::log_info(
                    &format!(
                        "Reset sort order for {} packages (preserved {} locked positions)",
                        sorted_names.len(),
                        locked_applied
                    ),
                    Some("scenery_index"),
                );
            } else {
                logger::log_info(
                    &format!("Reset sort order for {} packages", sorted_names.len()),
                    Some("scenery_index"),
                );
            }
        } else {
            logger::log_info(
                "Sort order is already correct, no changes needed",
                Some("scenery_index"),
            );
        }

        Ok(has_changes)
    }

    fn find_nearest_free_slot(occupied: &[bool], desired: usize) -> Option<usize> {
        let len = occupied.len();
        if len == 0 {
            return None;
        }

        let desired = desired.min(len - 1);
        if !occupied[desired] {
            return Some(desired);
        }

        for offset in 1..len {
            let forward = desired + offset;
            if forward < len && !occupied[forward] {
                return Some(forward);
            }

            if desired >= offset {
                let backward = desired - offset;
                if !occupied[backward] {
                    return Some(backward);
                }
            }
        }

        None
    }

    fn apply_sort_order_with_locked_slots(
        index: &mut SceneryIndex,
        sorted_names: &[String],
        locked_folder_names: &[String],
        locked_anchor_orders: Option<&HashMap<String, usize>>,
    ) -> (bool, usize) {
        if sorted_names.is_empty() {
            return (false, 0);
        }

        let assign_sequential =
            |index: &mut SceneryIndex, sorted_names: &[String]| -> (bool, usize) {
                let mut has_changes = false;
                for (new_order, folder_name) in sorted_names.iter().enumerate() {
                    if let Some(info) = index.packages.get_mut(folder_name) {
                        let new_order_u32 = new_order as u32;
                        if info.sort_order != new_order_u32 {
                            info.sort_order = new_order_u32;
                            has_changes = true;
                        }
                    }
                }
                (has_changes, 0)
            };

        let locked_names: HashSet<String> = locked_folder_names
            .iter()
            .map(|name| name.trim().to_lowercase())
            .filter(|name| !name.is_empty())
            .collect();

        if locked_names.is_empty() {
            return assign_sequential(index, sorted_names);
        }

        let total = sorted_names.len();
        let mut locked_candidates: Vec<(String, usize)> = sorted_names
            .iter()
            .filter(|name| locked_names.contains(&name.to_lowercase()))
            .filter_map(|name| {
                index.packages.get(name).map(|info| {
                    let normalized_name = name.to_lowercase();
                    let desired_slot = locked_anchor_orders
                        .and_then(|anchors| anchors.get(&normalized_name).copied())
                        .unwrap_or(info.sort_order as usize)
                        .min(total - 1);
                    (name.clone(), desired_slot)
                })
            })
            .collect();

        if locked_candidates.is_empty() {
            return assign_sequential(index, sorted_names);
        }

        locked_candidates.sort_by(|(name_a, order_a), (name_b, order_b)| {
            order_a
                .cmp(order_b)
                .then_with(|| name_a.to_lowercase().cmp(&name_b.to_lowercase()))
        });

        let mut occupied = vec![false; total];
        let mut final_order: Vec<Option<String>> = vec![None; total];
        let mut locked_applied: HashSet<String> = HashSet::new();

        for (name, desired_slot) in locked_candidates {
            if let Some(slot) = Self::find_nearest_free_slot(&occupied, desired_slot) {
                occupied[slot] = true;
                final_order[slot] = Some(name.clone());
                locked_applied.insert(name);
            }
        }

        let mut unlocked_iter = sorted_names
            .iter()
            .filter(|name| !locked_applied.contains(*name))
            .cloned();

        for slot in 0..total {
            if final_order[slot].is_none() {
                final_order[slot] = unlocked_iter.next();
            }
        }

        let mut has_changes = false;
        for (new_order, folder_name_opt) in final_order.into_iter().enumerate() {
            if let Some(folder_name) = folder_name_opt {
                if let Some(info) = index.packages.get_mut(&folder_name) {
                    let new_order_u32 = new_order as u32;
                    if info.sort_order != new_order_u32 {
                        info.sort_order = new_order_u32;
                        has_changes = true;
                    }
                }
            }
        }

        (has_changes, locked_applied.len())
    }

    /// Get scenery manager data for UI
    pub async fn get_manager_data(&self) -> Result<SceneryManagerData> {
        let mut index = self.load_index().await?;
        recompute_missing_libraries_in_memory(&self.xplane_path, &mut index);

        // Check if ini is synced with index
        let packs_manager = crate::scenery_packs_manager::SceneryPacksManager::new(
            &self.xplane_path,
            self.db.clone(),
        );
        let needs_sync = !packs_manager.is_synced_with_index().await.unwrap_or(true);

        // Detect duplicate tiles within Mesh and AirportMesh categories
        let custom_scenery_path = self.xplane_path.join("Custom Scenery");
        let raw_tile_overlaps = detect_raw_tile_overlaps(&index.packages, &custom_scenery_path);
        let duplicate_tiles_map =
            filter_tile_overlaps_with_xpme_rules(&raw_tile_overlaps, &index.packages);

        // Detect duplicate airports (same airport_id across multiple packages)
        let duplicate_airports_map = detect_duplicate_airports(&index.packages);

        let all_packages: Vec<_> = index.packages.values().collect();
        let global_airports = packs_manager
            .get_global_airports_state_for_packages(&all_packages)
            .await?;

        let mut packages: Vec<_> = index
            .packages
            .values()
            .filter(|info| !is_global_airports_folder_name(&info.folder_name))
            .collect();
        let mut entries_with_sort: Vec<(u32, bool, SceneryManagerEntry)> =
            Vec::with_capacity(packages.len() + 1);

        for info in packages.drain(..) {
            let flatten_target =
                crate::airport_flatten::inspect_scenery_flatten_target(&self.xplane_path, info);
            let pkg_dir = resolve_package_dir(&custom_scenery_path, info);
            let (version, update_url) = read_scenery_update_info(&pkg_dir);
            let update_provider = update_url.as_ref().map(|_| "skunkcrafts".to_string());
            let (is_symlink, link_target) = {
                let p = custom_scenery_path.join(&info.folder_name);
                #[cfg(windows)]
                {
                    if junction::exists(&p).unwrap_or(false) {
                        let target = junction::get_target(&p).ok().map(|t| crate::symlink_manager::clean_path_string(&t));
                        (true, target)
                    } else if let Ok(target) = p.read_link() {
                        (true, Some(crate::symlink_manager::clean_path_string(&target)))
                    } else if let Some(ref act) = info.actual_path {
                        (true, Some(act.clone()))
                    } else {
                        (false, None)
                    }
                }
                #[cfg(not(windows))]
                {
                    if let Ok(target) = p.read_link() {
                        (true, Some(target.to_string_lossy().to_string()))
                    } else if let Some(ref act) = info.actual_path {
                        (true, Some(act.clone()))
                    } else {
                        (false, None)
                    }
                }
            };
            entries_with_sort.push((
                info.sort_order,
                false,
                SceneryManagerEntry {
                    folder_name: info.folder_name.clone(),
                    category: info.category.clone(),
                    sub_priority: info.sub_priority,
                    enabled: info.enabled,
                    sort_order: info.sort_order,
                    update_url,
                    update_provider,
                    version,
                    latest_version: None,
                    has_update: false,
                    missing_libraries: info.missing_libraries.clone(),
                    required_libraries: info.required_libraries.clone(),
                    continent: info.continent.clone(),
                    duplicate_tiles: duplicate_tiles_map
                        .get(&info.folder_name)
                        .cloned()
                        .unwrap_or_default(),
                    duplicate_airports: duplicate_airports_map
                        .get(&info.folder_name)
                        .cloned()
                        .unwrap_or_default(),
                    airport_id: info.airport_id.clone(),
                    original_category: info.original_category.clone(),
                    flatten_available: flatten_target.is_some(),
                    flattened: flatten_target
                        .as_ref()
                        .map(|target| target.flattened)
                        .unwrap_or(false),
                    is_symlink,
                    link_target,
                },
            ));
        }

        entries_with_sort.push((
            global_airports.sort_order,
            true,
            SceneryManagerEntry {
                folder_name: GLOBAL_AIRPORTS_ENTRY_NAME.to_string(),
                category: global_airports.category.clone(),
                sub_priority: 0,
                enabled: global_airports.enabled,
                sort_order: global_airports.sort_order,
                update_url: None,
                update_provider: None,
                version: None,
                latest_version: None,
                has_update: false,
                missing_libraries: Vec::new(),
                required_libraries: Vec::new(),
                continent: None,
                duplicate_tiles: Vec::new(),
                duplicate_airports: Vec::new(),
                airport_id: None,
                original_category: None,
                flatten_available: false,
                flattened: false,
                is_symlink: false,
                link_target: None,
            },
        ));

        entries_with_sort.sort_by(|(sort_a, global_a, _), (sort_b, global_b, _)| {
            sort_a.cmp(sort_b).then_with(|| global_b.cmp(global_a))
        });

        let mut entries: Vec<SceneryManagerEntry> = entries_with_sort
            .into_iter()
            .map(|(_, _, entry)| entry)
            .collect();
        for (index, entry) in entries.iter_mut().enumerate() {
            entry.sort_order = index as u32;
        }

        // Calculate statistics
        let total_count = entries.len();
        let enabled_count = entries.iter().filter(|e| e.enabled).count();
        let missing_deps_count = entries
            .iter()
            .filter(|e| !e.missing_libraries.is_empty())
            .count();
        let duplicate_tiles_count = entries
            .iter()
            .filter(|e| !e.duplicate_tiles.is_empty())
            .count();
        let duplicate_airports_count = entries
            .iter()
            .filter(|e| !e.duplicate_airports.is_empty())
            .count();

        Ok(SceneryManagerData {
            entries,
            total_count,
            enabled_count,
            missing_deps_count,
            duplicate_tiles_count,
            duplicate_airports_count,
            needs_sync,
            tile_overlaps: raw_tile_overlaps,
        })
    }

    /// Create an empty index
    fn create_empty_index(&self) -> SceneryIndex {
        SceneryIndex {
            version: CURRENT_SCHEMA_VERSION as u32,
            packages: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }
}

/// Parse library.txt file and extract all real exported virtual library prefixes.
/// Supports all official X-Plane 11 & X-Plane 12 export directives:
/// - EXPORT, EXPORT_EXTEND (and alias EXPORT_EXTENDED), EXPORT_EXCLUDE -> virtual path at index 1
/// - EXPORT_RATIO, EXPORT_SEASON, EXPORT_EXTEND_SEASON, EXPORT_EXCLUDE_SEASON -> virtual path at index 2
/// - EXPORT_RATIO_SEASON -> virtual path at index 3
/// Intentionally excludes EXPORT_BACKUP and EXPORT_EXCLUDE_BACKUP, which are dummy placeholder
/// stubs bundled by sceneries to avoid fatal X-Plane popups when a required library is missing.
pub fn parse_library_exports(library_txt_path: &Path) -> HashSet<String> {
    let mut library_names = HashSet::new();

    let file = match fs::File::open(library_txt_path) {
        Ok(f) => f,
        Err(_) => return library_names,
    };

    let reader = BufReader::with_capacity(32 * 1024, file);

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let cmd = parts[0];
        let vpath_opt = if cmd.eq_ignore_ascii_case("EXPORT")
            || cmd.eq_ignore_ascii_case("EXPORT_EXTEND")
            || cmd.eq_ignore_ascii_case("EXPORT_EXTENDED")
            || cmd.eq_ignore_ascii_case("EXPORT_EXCLUDE")
        {
            Some(parts[1])
        } else if (cmd.eq_ignore_ascii_case("EXPORT_RATIO")
            || cmd.eq_ignore_ascii_case("EXPORT_SEASON")
            || cmd.eq_ignore_ascii_case("EXPORT_EXTEND_SEASON")
            || cmd.eq_ignore_ascii_case("EXPORT_EXCLUDE_SEASON"))
            && parts.len() >= 4
        {
            Some(parts[2])
        } else if cmd.eq_ignore_ascii_case("EXPORT_RATIO_SEASON") && parts.len() >= 5 {
            Some(parts[3])
        } else {
            None
        };

        if let Some(raw_vpath) = vpath_opt {
            let normalized = if raw_vpath.contains('\\') {
                raw_vpath.replace('\\', "/")
            } else {
                raw_vpath.to_string()
            };
            let clean = normalized.trim_start_matches('/');
            if let Some(first) = clean.split('/').next() {
                let first = first.trim();
                if !first.is_empty() {
                    library_names.insert(first.to_string());
                }
            }
        }
    }

    library_names
}

/// Cache of built-in X-Plane default scenery library prefixes per X-Plane installation root
fn get_default_xplane_libraries(xplane_path: &Path) -> HashSet<String> {
    static DEFAULT_LIB_CACHE: OnceLock<Mutex<HashMap<PathBuf, HashSet<String>>>> = OnceLock::new();
    let cache = DEFAULT_LIB_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(xplane_path) {
            return cached.clone();
        }
    }

    let mut prefixes: HashSet<String> = HashSet::from(["lib".to_string()]);
    let default_scenery_dir = xplane_path.join("Resources").join("default scenery");
    if default_scenery_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&default_scenery_dir) {
            let lib_files: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path().join("library.txt"))
                .filter(|p| p.is_file())
                .collect();
            let parsed_sets: Vec<HashSet<String>> = lib_files
                .par_iter()
                .map(|lp| parse_library_exports(lp))
                .collect();
            for s in parsed_sets {
                for p in s {
                    prefixes.insert(p.to_lowercase());
                }
            }
        }
    }

    if let Ok(mut guard) = cache.lock() {
        guard.insert(xplane_path.to_path_buf(), prefixes.clone());
    }

    prefixes
}

/// Build a library name index from all enabled packages in the scenery index.
/// Maps lowercased virtual library prefixes to providing package folder names.
pub fn build_library_index_from_scenery_index(index: &SceneryIndex) -> HashMap<String, String> {
    let mut library_index: HashMap<String, String> = HashMap::new();

    for (folder_name, package_info) in &index.packages {
        // Only active/enabled packages satisfy external library dependencies in X-Plane
        if !package_info.enabled {
            continue;
        }

        for lib_name in &package_info.exported_library_names {
            let norm = lib_name.replace('\\', "/");
            if let Some(first) = norm.trim_start_matches('/').split('/').next() {
                let first = first.trim();
                if !first.is_empty() {
                    library_index.insert(first.to_lowercase(), folder_name.clone());
                }
            }
        }

        // For dedicated library packages, also register the folder name itself
        if matches!(
            package_info.category,
            SceneryCategory::Library | SceneryCategory::FixedHighPriority
        ) {
            library_index.insert(folder_name.to_lowercase(), folder_name.clone());
        }
    }

    library_index
}

/// Recompute missing_libraries for all packages in parallel in memory.
/// Returns true if any package's missing_libraries list changed.
pub fn recompute_missing_libraries_in_memory(xplane_path: &Path, index: &mut SceneryIndex) -> bool {
    let library_index = build_library_index_from_scenery_index(index);
    let default_libs = get_default_xplane_libraries(xplane_path);
    let custom_scenery_path = xplane_path.join("Custom Scenery");

    let changed_flags: Vec<bool> = index
        .packages
        .par_iter_mut()
        .map(|(folder_name, package_info)| {
            if package_info.required_libraries.is_empty() {
                if !package_info.missing_libraries.is_empty() {
                    package_info.missing_libraries.clear();
                    return true;
                }
                return false;
            }

            let self_exports_lower: HashSet<String> = package_info
                .exported_library_names
                .iter()
                .filter_map(|exp| {
                    let norm = exp.replace('\\', "/");
                    norm.trim_start_matches('/')
                        .split('/')
                        .next()
                        .map(|s| s.trim().to_lowercase())
                })
                .filter(|s| !s.is_empty())
                .collect();

            let pkg_dir = package_info
                .actual_path
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(|| custom_scenery_path.join(folder_name));

            let mut missing_map: HashMap<String, String> = HashMap::new();

            for raw_lib in &package_info.required_libraries {
                let norm = raw_lib.replace('\\', "/");
                let Some(lib_clean) = norm.trim_start_matches('/').split('/').next() else {
                    continue;
                };
                let lib_clean = lib_clean.trim();
                if lib_clean.is_empty() || lib_clean.starts_with("::") {
                    continue;
                }

                let lib_lower = lib_clean.to_lowercase();

                // 1. Built-in X-Plane default scenery library
                if default_libs.contains(&lib_lower) {
                    continue;
                }

                // 2. Self folder name or package's own library.txt exports (even if disabled)
                if lib_clean.eq_ignore_ascii_case(folder_name)
                    || self_exports_lower.contains(&lib_lower)
                {
                    continue;
                }

                // 3. Enabled global library in Custom Scenery
                if library_index.contains_key(&lib_lower) {
                    continue;
                }

                // 4. Local subdirectory inside the resolved package folder (including .lnk shortcuts)
                if pkg_dir.join(lib_clean).is_dir() {
                    continue;
                }

                missing_map
                    .entry(lib_lower)
                    .or_insert_with(|| lib_clean.to_string());
            }

            let mut new_missing: Vec<String> = missing_map.into_values().collect();
            new_missing.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

            if package_info.missing_libraries != new_missing {
                package_info.missing_libraries = new_missing;
                true
            } else {
                false
            }
        })
        .collect();

    changed_flags.into_iter().any(|c| c)
}

/// Remove a scenery entry from the index (public helper function)
pub async fn remove_scenery_entry(
    db: &DatabaseConnection,
    xplane_path: &str,
    folder_name: &str,
) -> Result<()> {
    let manager = SceneryIndexManager::new(Path::new(xplane_path), db.clone());
    manager.remove_entry(folder_name).await
}

/// Parse airport apt.dat to extract coordinates and ICAO code
/// Returns (latitude_floor, longitude_floor, Option<icao_code>)
/// Tries datum_lat/datum_lon first, falls back to runway coordinates
fn parse_airport_coords(scenery_path: &Path) -> Option<(i32, i32, Option<String>)> {
    // Find apt.dat file
    let apt_dat_path = scenery_path.join("Earth nav data").join("apt.dat");
    if !apt_dat_path.exists() {
        return None;
    }

    let file = fs::File::open(&apt_dat_path).ok()?;
    let reader = BufReader::new(file);

    let mut datum_lat: Option<f64> = None;
    let mut datum_lon: Option<f64> = None;
    let mut icao_code: Option<String> = None;
    let mut runway_lat: Option<f64> = None;
    let mut runway_lon: Option<f64> = None;

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();

        // Look for 1302 metadata lines
        if trimmed.starts_with("1302 ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                match parts[1] {
                    "datum_lat" => {
                        if let Ok(lat) = parts[2].parse::<f64>() {
                            // Validate latitude range
                            if (-90.0..=90.0).contains(&lat) {
                                datum_lat = Some(lat);
                            }
                        }
                    }
                    "datum_lon" => {
                        if let Ok(lon) = parts[2].parse::<f64>() {
                            // Validate longitude range
                            if (-180.0..=180.0).contains(&lon) {
                                datum_lon = Some(lon);
                            }
                        }
                    }
                    "icao_code" => {
                        icao_code = Some(parts[2].to_string());
                    }
                    _ => {}
                }
            }
        }
        // Fallback: parse runway line (row code 100) for coordinates
        // Format: 100 width surface shoulder smoothness centerline edge autosign runway_number lat lon ...
        // See X-Plane apt.dat specification for full format
        else if trimmed.starts_with("100 ") && runway_lat.is_none() {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            // Runway line requires at least 11 parts to have lat/lon at indices 9 and 10
            if parts.len() >= 11 {
                if let (Ok(lat), Ok(lon)) = (parts[9].parse::<f64>(), parts[10].parse::<f64>()) {
                    // Validate coordinate ranges to catch malformed data
                    // Latitude: -90 to 90, Longitude: -180 to 180
                    if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) {
                        runway_lat = Some(lat);
                        runway_lon = Some(lon);
                    }
                }
            }
        }

        // If we have datum coords and icao, we can stop early
        if datum_lat.is_some() && datum_lon.is_some() && icao_code.is_some() {
            break;
        }
    }

    // Prefer datum coordinates, fall back to runway coordinates
    let (lat, lon) = match (datum_lat, datum_lon) {
        (Some(lat), Some(lon)) => (lat, lon),
        _ => match (runway_lat, runway_lon) {
            (Some(lat), Some(lon)) => (lat, lon),
            _ => return None,
        },
    };

    // Floor the coordinates to get the tile
    let lat_floor = lat.floor() as i32;
    let lon_floor = lon.floor() as i32;
    Some((lat_floor, lon_floor, icao_code))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum AirportMeshMatchStrength {
    Prefix = 1,
    Icao = 2,
}

fn resolve_airport_mesh_match_for_candidates(
    pkg: &SceneryPackageInfo,
    airports: &[(String, Option<String>)],
    airport_prefixes: &HashSet<String>,
) -> Option<(String, AirportMeshMatchStrength)> {
    let folder_upper = pkg.folder_name.to_uppercase();

    // Prefer packages whose folder name contains the ICAO code.
    for (airport_folder, icao) in airports {
        if let Some(ref code) = icao {
            if folder_upper.contains(code) {
                return Some((airport_folder.clone(), AirportMeshMatchStrength::Icao));
            }
        }
    }

    // If no ICAO match, fall back to the existing prefix-based association rule.
    if let Some(prefix) = extract_scenery_prefix(&pkg.folder_name) {
        if airport_prefixes.contains(&prefix) {
            if let Some((airport_folder, _)) = airports.first() {
                return Some((airport_folder.clone(), AirportMeshMatchStrength::Prefix));
            }
        }
    }

    None
}

/// Detect mesh packages that are associated with airports.
/// Uses an explicit X-Plane path to avoid borrowing &self in blocking contexts.
fn detect_airport_mesh_matches_with_path(
    xplane_path: &Path,
    packages: &[SceneryPackageInfo],
) -> HashMap<String, String> {
    logger::log_info(
        "Detecting airport-associated mesh packages...",
        Some("scenery_index"),
    );

    // Step 1: Collect all airports with their coordinates, ICAO codes, and folder names
    let mut airport_coords: AirportCoords = HashMap::new();

    // Also collect airport folder name prefixes for prefix matching
    let mut airport_prefixes: HashSet<String> = HashSet::new();

    for pkg in packages.iter() {
        if pkg.category == SceneryCategory::Airport && pkg.has_apt_dat {
            // Parse apt.dat to get coordinates and ICAO code
            let scenery_path = resolve_package_dir(&xplane_path.join("Custom Scenery"), pkg);
            if let Some((lat, lon, icao)) = parse_airport_coords(&scenery_path) {
                let coord_key = (lat, lon);
                airport_coords
                    .entry(coord_key)
                    .or_default()
                    .push((pkg.folder_name.clone(), icao));

                // Extract common prefix (e.g., "ACS_Singapore" from "ACS_Singapore_0_Airport")
                if let Some(prefix) = extract_scenery_prefix(&pkg.folder_name) {
                    airport_prefixes.insert(prefix);
                }
            }
        }
    }

    logger::log_info(
        &format!("Found {} airport coordinate tiles", airport_coords.len()),
        Some("scenery_index"),
    );

    // Step 2: Resolve mesh packages with small DSF counts to their matching airport.
    let custom_scenery_path = xplane_path.join("Custom Scenery");
    let mut matches = HashMap::new();

    for pkg in packages.iter() {
        if !matches!(
            pkg.category,
            SceneryCategory::Mesh | SceneryCategory::AirportMesh
        ) {
            continue;
        }

        // Skip Ortho4XP packages - they are regional orthophotos, not airport-specific
        if pkg.folder_name.starts_with("zOrtho4XP") {
            continue;
        }

        let scenery_path = resolve_package_dir(&custom_scenery_path, pkg);

        // Count DSF files and get their coordinates
        if let Some(dsf_coords) = get_mesh_dsf_coordinates(&scenery_path) {
            // Only consider meshes with 4 or fewer DSF files
            if dsf_coords.len() > 4 {
                continue;
            }

            let mut best_match: Option<(String, AirportMeshMatchStrength)> = None;

            for (lat, lon) in &dsf_coords {
                if let Some(airports) = airport_coords.get(&(*lat, *lon)) {
                    crate::log_debug!(
                        &format!(
                            "  Mesh candidate: {} at ({}, {})",
                            pkg.folder_name, lat, lon
                        ),
                        "scenery_index"
                    );

                    if let Some(candidate_match) =
                        resolve_airport_mesh_match_for_candidates(pkg, airports, &airport_prefixes)
                    {
                        let should_replace = match best_match.as_ref() {
                            Some((_, strength)) => candidate_match.1 > *strength,
                            None => true,
                        };

                        if should_replace {
                            let is_icao_match = candidate_match.1 == AirportMeshMatchStrength::Icao;
                            best_match = Some(candidate_match);
                            if is_icao_match {
                                break;
                            }
                        }
                    }
                }
            }

            if let Some((airport_folder, _)) = best_match {
                matches.insert(pkg.folder_name.clone(), airport_folder);
            }
        }
    }

    logger::log_info(
        &format!("Matched {} airport-associated mesh packages", matches.len()),
        Some("scenery_index"),
    );

    matches
}

/// Get DSF file coordinates from a mesh scenery package
/// Returns list of (latitude, longitude) tuples extracted from DSF filenames
fn get_mesh_dsf_coordinates(scenery_path: &Path) -> Option<Vec<(i32, i32)>> {
    let earth_nav_data = scenery_path.join("Earth nav data");
    if !earth_nav_data.exists() {
        return None;
    }

    let mut coordinates: Vec<(i32, i32)> = Vec::new();

    // Iterate through subdirectories (e.g., +30+135)
    if let Ok(entries) = fs::read_dir(&earth_nav_data) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Look for .dsf files in this directory
            if let Ok(dsf_entries) = fs::read_dir(&path) {
                for dsf_entry in dsf_entries.flatten() {
                    let dsf_path = dsf_entry.path();
                    if let Some(ext) = dsf_path.extension() {
                        if ext.eq_ignore_ascii_case("dsf") {
                            // Parse coordinates from filename (e.g., +30+135.dsf)
                            if let Some(coord) = parse_dsf_filename(&dsf_path) {
                                coordinates.push(coord);
                            }
                        }
                    }
                }
            }
        }
    }

    if coordinates.is_empty() {
        None
    } else {
        Some(coordinates)
    }
}

/// Parse DSF filename to extract coordinates
/// Format: +30+135.dsf or -45-073.dsf (latitude + longitude)
fn parse_dsf_filename(dsf_path: &Path) -> Option<(i32, i32)> {
    let stem = dsf_path.file_stem()?.to_str()?;

    // DSF filenames are in format: [+-]NN[+-]NNN
    // e.g., +30+135, -45-073, +09-079
    if stem.len() < 7 {
        return None;
    }

    // Find the second sign character (start of longitude)
    let chars: Vec<char> = stem.chars().collect();
    let mut lon_start = None;

    for (i, ch) in chars.iter().enumerate().skip(1) {
        if *ch == '+' || *ch == '-' {
            lon_start = Some(i);
            break;
        }
    }

    let lon_start = lon_start?;

    let lat_str = &stem[0..lon_start];
    let lon_str = &stem[lon_start..];

    let lat: i32 = lat_str.parse().ok()?;
    let lon: i32 = lon_str.parse().ok()?;

    Some((lat, lon))
}

/// Detect duplicate airports: packages that define the same airport identifier.
/// Returns a map of folder_name -> list of other folder names sharing the same airport_id.
fn detect_duplicate_airports(
    packages: &HashMap<String, SceneryPackageInfo>,
) -> HashMap<String, Vec<String>> {
    // Build map: airport_id -> Vec<folder_name> (only for enabled packages)
    let mut id_map: HashMap<String, Vec<String>> = HashMap::new();
    for (folder_name, info) in packages {
        if !info.enabled {
            continue;
        }
        if let Some(ref airport_id) = info.airport_id {
            id_map
                .entry(airport_id.clone())
                .or_default()
                .push(folder_name.clone());
        }
    }

    // For each airport_id with >1 package, record overlaps
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    for folders in id_map.values() {
        if folders.len() > 1 {
            for folder in folders {
                let others: Vec<String> =
                    folders.iter().filter(|f| *f != folder).cloned().collect();
                result.entry(folder.clone()).or_default().extend(others);
            }
        }
    }

    // Deduplicate
    for v in result.values_mut() {
        v.sort();
        v.dedup();
    }

    result
}

/// Detect all DSF tile overlaps within Mesh and AirportMesh categories separately.
/// Returns a raw map of folder_name -> list of ALL overlapping folder names (no XPME filtering).
/// Cross-category duplicates (Mesh vs AirportMesh) are NOT flagged.
fn detect_raw_tile_overlaps(
    packages: &HashMap<String, SceneryPackageInfo>,
    custom_scenery_path: &Path,
) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    // Process each category separately
    for category in [SceneryCategory::Mesh, SceneryCategory::AirportMesh] {
        // Build coordinate index: (lat, lon) -> list of folder names
        let mut coord_map: HashMap<(i32, i32), Vec<String>> = HashMap::new();

        for (folder_name, info) in packages.iter() {
            if info.category != category {
                continue;
            }

            // Use actual_path if available (for shortcuts/symlinks), otherwise use folder_name
            let scenery_path = if let Some(ref actual_path) = info.actual_path {
                PathBuf::from(actual_path)
            } else {
                custom_scenery_path.join(folder_name)
            };

            if let Some(coords) = get_mesh_dsf_coordinates(&scenery_path) {
                for coord in coords {
                    coord_map
                        .entry(coord)
                        .or_default()
                        .push(folder_name.clone());
                }
            }
        }

        // For each coordinate with multiple packages, record all overlaps
        for (_coord, folder_names) in coord_map.iter() {
            if folder_names.len() > 1 {
                for folder in folder_names {
                    let others: Vec<String> = folder_names
                        .iter()
                        .filter(|f| *f != folder)
                        .cloned()
                        .collect();

                    if !others.is_empty() {
                        result.entry(folder.clone()).or_default().extend(others);
                    }
                }
            }
        }
    }

    // Deduplicate each folder's overlap list
    for duplicates in result.values_mut() {
        duplicates.sort();
        duplicates.dedup();
    }

    result
}

/// Apply XPME filtering rules to raw tile overlaps based on current sort_order.
/// - XPME_ packages never receive duplicate warnings themselves.
/// - Non-XPME packages are only warned about XPME overlap when they are
///   sorted below (higher sort_order = lower priority than) the XPME package.
fn filter_tile_overlaps_with_xpme_rules(
    raw_overlaps: &HashMap<String, Vec<String>>,
    packages: &HashMap<String, SceneryPackageInfo>,
) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for (folder, overlaps) in raw_overlaps {
        // XPME_ packages and disabled packages never get flagged
        if folder.starts_with("XPME_") {
            continue;
        }
        let Some(folder_info) = packages.get(folder.as_str()) else {
            continue;
        };
        if !folder_info.enabled {
            continue;
        }

        let folder_sort = folder_info.sort_order;

        let filtered: Vec<String> = overlaps
            .iter()
            .filter(|other| {
                let Some(other_info) = packages.get(other.as_str()) else {
                    return false;
                };
                if !other_info.enabled {
                    return false;
                }
                if other.starts_with("XPME_") {
                    return folder_sort > other_info.sort_order;
                }
                true
            })
            .cloned()
            .collect();

        if !filtered.is_empty() {
            result.insert(folder.clone(), filtered);
        }
    }

    result
}

/// Extract scenery package naming prefix for matching related packages
/// Examples:
///   "ACS_Singapore_0_Airport" -> "ACS_Singapore"
///   "ACS_Singapore_3_Orthos" -> "ACS_Singapore"
///   "Taimodels_WSSS_Singapore_Changi-MESH" -> "Taimodels_WSSS_Singapore_Changi"
///   "FlyTampa_Amsterdam_3_mesh" -> "FlyTampa_Amsterdam"
/// The prefix is the part before "_<number>_" pattern (if found)
fn extract_scenery_prefix(folder_name: &str) -> Option<String> {
    // Look for pattern: prefix_<number>_suffix
    // We want to extract "prefix" part
    let parts: Vec<&str> = folder_name.split('_').collect();

    // Need at least 3 parts to have "prefix_number_suffix" pattern
    if parts.len() >= 3 {
        // Find index of numeric part
        for i in 1..parts.len() - 1 {
            if parts[i].chars().all(|c| c.is_ascii_digit()) && !parts[i].is_empty() {
                // Found numeric part, prefix is everything before it
                let prefix = parts[..i].join("_");
                if !prefix.is_empty() {
                    return Some(prefix);
                }
            }
        }
    }

    // Fallback: if no "_<number>_" pattern found, try to extract meaningful prefix
    // by taking everything before common suffixes like "-MESH", "_Mesh", "_Orthos", "_Airport"
    let folder_lower = folder_name.to_lowercase();
    let suffixes = ["-mesh", "_mesh", "_orthos", "_orthophoto", "_airport"];

    for suffix in suffixes {
        if let Some(pos) = folder_lower.rfind(suffix) {
            let prefix = &folder_name[..pos];
            // Strip trailing underscore or dash if present
            let prefix = prefix.trim_end_matches(['_', '-']);
            if !prefix.is_empty() {
                return Some(prefix.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_package(
        folder_name: &str,
        category: SceneryCategory,
        sort_order: u32,
    ) -> SceneryPackageInfo {
        SceneryPackageInfo {
            folder_name: folder_name.to_string(),
            category,
            sub_priority: 0,
            last_modified: SystemTime::UNIX_EPOCH,
            has_apt_dat: false,
            airport_id: None,
            has_dsf: false,
            has_library_txt: false,
            has_textures: false,
            has_objects: false,
            texture_count: 0,
            earth_nav_tile_count: 1,
            indexed_at: SystemTime::UNIX_EPOCH,
            required_libraries: Vec::new(),
            missing_libraries: Vec::new(),
            exported_library_names: Vec::new(),
            enabled: true,
            sort_order,
            actual_path: None,
            continent: None,
            original_category: None,
        }
    }

    fn write_test_airport(
        xplane_root: &Path,
        folder_name: &str,
        airport_id: &str,
        lat: f64,
        lon: f64,
    ) {
        let airport_path = xplane_root
            .join("Custom Scenery")
            .join(folder_name)
            .join("Earth nav data");
        fs::create_dir_all(&airport_path).unwrap();
        fs::write(
            airport_path.join("apt.dat"),
            format!(
                "I\n1100 Test\n1 0 0 0 {airport_id} Test Airport\n1302 datum_lat {lat}\n1302 datum_lon {lon}\n1302 icao_code {airport_id}\n"
            ),
        )
        .unwrap();
    }

    fn write_test_mesh(xplane_root: &Path, folder_name: &str, dsf_name: &str) {
        let mesh_path = xplane_root
            .join("Custom Scenery")
            .join(folder_name)
            .join("Earth nav data")
            .join("+40-130");
        fs::create_dir_all(&mesh_path).unwrap();
        fs::write(mesh_path.join(dsf_name), b"test").unwrap();
    }

    #[test]
    fn test_empty_index_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db = crate::database::open_memory_connection().unwrap();
        crate::database::apply_migrations(&db).unwrap();
        let manager = SceneryIndexManager::new(temp_dir.path(), db);
        let index = manager.create_empty_index();

        assert_eq!(index.version, CURRENT_SCHEMA_VERSION as u32);
        assert!(index.packages.is_empty());
    }

    #[test]
    fn test_extract_scenery_prefix() {
        // Test "_<number>_" pattern extraction
        assert_eq!(
            extract_scenery_prefix("ACS_Singapore_0_Airport"),
            Some("ACS_Singapore".to_string())
        );
        assert_eq!(
            extract_scenery_prefix("ACS_Singapore_3_Orthos"),
            Some("ACS_Singapore".to_string())
        );
        assert_eq!(
            extract_scenery_prefix("FlyTampa_Amsterdam_3_mesh"),
            Some("FlyTampa_Amsterdam".to_string())
        );

        // Test suffix-based extraction fallback
        assert_eq!(
            extract_scenery_prefix("Taimodels_WSSS_Singapore_Changi-MESH"),
            Some("Taimodels_WSSS_Singapore_Changi".to_string())
        );

        // Test names without patterns
        assert_eq!(extract_scenery_prefix("SimpleFolder"), None);
    }

    #[test]
    fn test_lines3d_folder_detection() {
        assert!(is_lines3d_folder_name("Lines3D"));
        assert!(is_lines3d_folder_name("lines3d"));
        assert!(!is_lines3d_folder_name("Lines3D_expanded_documentation"));
        assert!(!is_lines3d_folder_name(
            "Simple_Ground_Equipment_and_Services"
        ));
    }

    #[test]
    fn test_apply_darkblue_airport_package_anchors_moves_darkblue_overlay_and_mesh_below_airport() {
        let mut packages = vec![
            make_package("DarkBlue-KSEA", SceneryCategory::Airport, 0),
            make_package("MisterX Library", SceneryCategory::Library, 1),
            make_package("DarkBlue-KSEA_Overlays1", SceneryCategory::Overlay, 2),
            make_package("DarkBlue-KSEA_Overlays2", SceneryCategory::AirportMesh, 3),
            make_package("Regular-KSEA Mesh", SceneryCategory::AirportMesh, 4),
        ];
        let airport_mesh_matches = HashMap::from([
            (
                "DarkBlue-KSEA_Overlays2".to_string(),
                "DarkBlue-KSEA".to_string(),
            ),
            ("Regular-KSEA Mesh".to_string(), "DarkBlue-KSEA".to_string()),
        ]);

        apply_darkblue_airport_package_anchors(&mut packages, &airport_mesh_matches);

        let ordered_names: Vec<&str> = packages
            .iter()
            .map(|pkg| pkg.folder_name.as_str())
            .collect();
        assert_eq!(
            ordered_names,
            vec![
                "DarkBlue-KSEA",
                "DarkBlue-KSEA_Overlays1",
                "DarkBlue-KSEA_Overlays2",
                "MisterX Library",
                "Regular-KSEA Mesh"
            ]
        );
    }

    #[test]
    fn test_apply_darkblue_airport_package_anchors_ignores_unmatched_or_non_darkblue() {
        let mut packages = vec![
            make_package("DarkBlue-KSEA", SceneryCategory::Airport, 0),
            make_package("MisterX Library", SceneryCategory::Library, 1),
            make_package("DarkBlue-KJFK Overlay", SceneryCategory::Overlay, 2),
            make_package("DarkBlue-KJFK Mesh", SceneryCategory::AirportMesh, 3),
            make_package("Regular-KSEA Overlay", SceneryCategory::Overlay, 4),
        ];
        let original_names: Vec<String> =
            packages.iter().map(|pkg| pkg.folder_name.clone()).collect();
        let airport_mesh_matches = HashMap::from([(
            "DarkBlue-KJFK Mesh".to_string(),
            "Missing Airport".to_string(),
        )]);

        apply_darkblue_airport_package_anchors(&mut packages, &airport_mesh_matches);

        let ordered_names: Vec<String> =
            packages.iter().map(|pkg| pkg.folder_name.clone()).collect();
        assert_eq!(ordered_names, original_names);
    }

    #[test]
    fn test_sort_packages_with_special_rules_promotes_and_anchors_darkblue_overlays_and_meshes() {
        let temp_dir = tempfile::tempdir().unwrap();
        write_test_airport(temp_dir.path(), "DarkBlue-KSEA", "KSEA", 47.4489, -122.3094);
        write_test_mesh(temp_dir.path(), "DarkBlue-KSEA_Overlays1", "+47-123.dsf");
        write_test_mesh(temp_dir.path(), "DarkBlue-KSEA_Overlays2", "+47-123.dsf");
        write_test_mesh(temp_dir.path(), "Regular-KSEA Mesh", "+47-123.dsf");

        let mut airport = make_package("DarkBlue-KSEA", SceneryCategory::Airport, 0);
        airport.has_apt_dat = true;
        airport.airport_id = Some("KSEA".to_string());

        let mut packages = vec![
            airport,
            make_package("MisterX Library", SceneryCategory::Library, 1),
            make_package("DarkBlue-KSEA_Overlays1", SceneryCategory::Overlay, 2),
            make_package("DarkBlue-KSEA_Overlays2", SceneryCategory::Mesh, 3),
            make_package("Regular-KSEA Mesh", SceneryCategory::Mesh, 4),
        ];

        let category_changed = sort_packages_with_special_rules(temp_dir.path(), &mut packages);

        assert!(category_changed);
        let ordered_names: Vec<&str> = packages
            .iter()
            .map(|pkg| pkg.folder_name.as_str())
            .collect();
        assert_eq!(
            ordered_names,
            vec![
                "DarkBlue-KSEA",
                "DarkBlue-KSEA_Overlays1",
                "DarkBlue-KSEA_Overlays2",
                "MisterX Library",
                "Regular-KSEA Mesh"
            ]
        );
        assert_eq!(packages[1].category, SceneryCategory::Overlay);
        assert_eq!(packages[2].category, SceneryCategory::AirportMesh);
        assert_eq!(packages[4].category, SceneryCategory::AirportMesh);
    }

    #[test]
    fn test_reset_sort_order_detects_darkblue_anchor_difference() {
        let temp_dir = tempfile::tempdir().unwrap();
        write_test_airport(temp_dir.path(), "KSEA Demo", "KSEA", 47.4489, -122.3094);
        write_test_mesh(temp_dir.path(), "DarkBlue-KSEA Mesh", "+47-123.dsf");

        let db = crate::database::open_memory_connection().unwrap();
        crate::database::apply_migrations(&db).unwrap();
        let manager = SceneryIndexManager::new(temp_dir.path(), db);

        let mut airport = make_package("KSEA Demo", SceneryCategory::Airport, 0);
        airport.has_apt_dat = true;
        airport.airport_id = Some("KSEA".to_string());

        let mut darkblue_mesh = make_package("DarkBlue-KSEA Mesh", SceneryCategory::AirportMesh, 2);
        darkblue_mesh.airport_id = Some("KSEA".to_string());

        let index = SceneryIndex {
            version: CURRENT_SCHEMA_VERSION as u32,
            packages: HashMap::from([
                ("KSEA Demo".to_string(), airport),
                (
                    "MisterX Library".to_string(),
                    make_package("MisterX Library", SceneryCategory::Library, 1),
                ),
                ("DarkBlue-KSEA Mesh".to_string(), darkblue_mesh),
            ]),
            last_updated: SystemTime::UNIX_EPOCH,
        };

        tauri::async_runtime::block_on(manager.save_index(&index)).unwrap();
        let has_changes = tauri::async_runtime::block_on(
            manager.reset_sort_order_with_locked_entries(Vec::new()),
        )
        .unwrap();
        assert!(has_changes);

        let updated_index = tauri::async_runtime::block_on(manager.load_index()).unwrap();
        let mut ordered_entries: Vec<(&String, &SceneryPackageInfo)> =
            updated_index.packages.iter().collect();
        ordered_entries.sort_by_key(|(_, info)| info.sort_order);
        let ordered_names: Vec<&str> = ordered_entries
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();

        assert_eq!(
            ordered_names,
            vec!["KSEA Demo", "DarkBlue-KSEA Mesh", "MisterX Library"]
        );
    }
}
