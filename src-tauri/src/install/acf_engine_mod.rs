//! Automated X-Plane `.acf` Engine Mod Installer (Mango Studios FF 767 PW4000 & GE CF6).
//!
//! Safely copies mod `objects/` (plus optional `liveries/` & compatible objects) into the target
//! aircraft folder, modifies `P _obja/...` attached object entries in each selected `.acf` file
//! without needing Plane Maker, and saves a full revertable backup in `_dhub_patch_backups/`
//! compatible with DHUB's existing patch restore flow.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::scanner::Scanner;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineModPresetId {
    Ff767MangoPw4000,
    Ff767MangoGeCf6,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AircraftVariantKind {
    V200ErSf,
    V300ErF,
    V400Er,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineModObjectSpec {
    pub obj_path: String,
    pub long_arm: f64,
    pub lat_arm: f64,
    pub vert_arm: f64,
    pub hdg: f64,
    pub pitch: f64,
    pub roll: f64,
    pub hide_dataref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedAcfVariant {
    pub file_name: String,
    pub variant: AircraftVariantKind,
    pub supported_by_preset: bool,
    pub has_default_engine_obj: bool,
    pub default_engine_obj_name: Option<String>,
    pub coordinates: Option<(f64, f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineModInspection {
    pub source_path: String,
    pub is_archive: bool,
    pub detected_preset: Option<EngineModPresetId>,
    pub preset_name: Option<String>,
    pub main_mod_objects_subpath: Option<String>,
    pub liveries_subpath: Option<String>,
    pub compatible_objects_subpath: Option<String>,
    pub compatible_objects_label: Option<String>,
    pub acf_variants: Vec<DetectedAcfVariant>,
    pub objects_to_add: Vec<String>,
    pub clear_engine_keyword: Option<String>,
    pub hide_dataref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineModApplyRequest {
    pub source_path: String,
    pub xplane_path: String,
    pub aircraft_folder: String,
    pub preset_id: EngineModPresetId,
    pub selected_acf_files: Vec<String>,
    pub install_liveries: bool,
    pub install_compatible_objects: bool,
    pub backup_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineModApplyResult {
    pub copied_files: usize,
    pub patched_acf_files: Vec<String>,
    pub backup_session_id: Option<String>,
}

fn preset_display_name(preset: EngineModPresetId) -> &'static str {
    match preset {
        EngineModPresetId::Ff767MangoPw4000 => "Mango Studios FF 767 PW4000 Mod",
        EngineModPresetId::Ff767MangoGeCf6 => "Mango Studios FF 767 GE CF6 Mod",
    }
}

fn preset_clear_keyword(preset: EngineModPresetId) -> &'static str {
    match preset {
        EngineModPresetId::Ff767MangoPw4000 => "pw",
        EngineModPresetId::Ff767MangoGeCf6 => "ge",
    }
}

fn preset_hide_dataref(preset: EngineModPresetId) -> &'static str {
    match preset {
        EngineModPresetId::Ff767MangoPw4000 => "1-sim/object/kill/PW",
        EngineModPresetId::Ff767MangoGeCf6 => "1-sim/object/kill/GE",
    }
}

fn preset_object_paths(preset: EngineModPresetId) -> &'static [&'static str] {
    match preset {
        EngineModPresetId::Ff767MangoPw4000 => &[
            "767/PW Engines/PW4000L.obj",
            "767/PW Engines/PW4000R.obj",
            "767/PW Engines/N1_fast.obj",
        ],
        EngineModPresetId::Ff767MangoGeCf6 => &[
            "767/GE CF6 Engines/CF6.obj",
            "767/GE CF6 Engines/Misc.obj",
            "767/GE CF6 Engines/N1.obj",
        ],
    }
}

fn preset_coords_for_variant(
    preset: EngineModPresetId,
    variant: AircraftVariantKind,
) -> Option<(f64, f64, f64)> {
    match (preset, variant) {
        (EngineModPresetId::Ff767MangoPw4000, AircraftVariantKind::V200ErSf) => {
            Some((188.97, 0.0, 26.20))
        }
        (EngineModPresetId::Ff767MangoPw4000, AircraftVariantKind::V300ErF) => {
            Some((199.50, 0.0, 26.20))
        }
        (EngineModPresetId::Ff767MangoPw4000, AircraftVariantKind::V400Er) => None,
        (EngineModPresetId::Ff767MangoGeCf6, AircraftVariantKind::V200ErSf) => {
            Some((188.97, 0.0, 26.20))
        }
        (EngineModPresetId::Ff767MangoGeCf6, AircraftVariantKind::V300ErF) => {
            Some((199.50, 0.0, 26.20))
        }
        (EngineModPresetId::Ff767MangoGeCf6, AircraftVariantKind::V400Er) => {
            Some((210.89, 0.0, 26.21))
        }
        (_, AircraftVariantKind::Unknown) => None,
    }
}

pub fn classify_ff767_acf_variant(file_name: &str) -> AircraftVariantKind {
    let lower = file_name.to_ascii_lowercase();
    if lower.contains("400") {
        AircraftVariantKind::V400Er
    } else if lower.contains("200") || lower.contains("-sf") || lower.contains("_sf") || lower.contains("sf_") {
        AircraftVariantKind::V200ErSf
    } else if lower.contains("300") || lower.contains("-f") || lower.contains("_f.") || lower.contains("_f_") || lower.contains("freighter") {
        AircraftVariantKind::V300ErF
    } else {
        AircraftVariantKind::Unknown
    }
}

fn is_supported_archive(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "zip" | "7z" | "rar"))
}

fn collect_source_entries(source: &Path) -> Result<(bool, Vec<String>)> {
    if is_supported_archive(source) {
        let scanner = Scanner::new();
        let entries = scanner
            .list_archive_entries(source)?
            .into_iter()
            .map(|p| p.replace('\\', "/"))
            .collect();
        Ok((true, entries))
    } else if source.is_dir() {
        let mut entries = Vec::new();
        for entry in WalkDir::new(source).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(rel) = entry.path().strip_prefix(source) {
                    entries.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        Ok((false, entries))
    } else {
        Err(anyhow!("Source path does not exist or is not a folder/archive"))
    }
}

fn detect_preset_from_entries(source: &Path, entries: &[String]) -> Option<EngineModPresetId> {
    let has_pw4000_obj = entries.iter().any(|e| {
        let l = e.to_ascii_lowercase();
        l.ends_with("pw4000l.obj") || l.ends_with("pw4000r.obj")
    });
    let has_ge_cf6_obj = entries.iter().any(|e| {
        let l = e.to_ascii_lowercase();
        l.contains("ge cf6") && l.ends_with("cf6.obj")
    });
    if has_pw4000_obj && !has_ge_cf6_obj {
        return Some(EngineModPresetId::Ff767MangoPw4000);
    }
    if has_ge_cf6_obj && !has_pw4000_obj {
        return Some(EngineModPresetId::Ff767MangoGeCf6);
    }
    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name.contains("pw4000") {
        Some(EngineModPresetId::Ff767MangoPw4000)
    } else if name.contains("cf6") || name.contains("ge") {
        Some(EngineModPresetId::Ff767MangoGeCf6)
    } else {
        None
    }
}

fn find_subpath_ending(entries: &[String], target_suffix: &str, must_contain: Option<&str>) -> Option<String> {
    let target_lower = target_suffix.to_ascii_lowercase();
    let must_lower = must_contain.map(|s| s.to_ascii_lowercase());
    let mut candidates: Vec<String> = Vec::new();
    for entry in entries {
        let parts: Vec<&str> = entry.split('/').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let dir = parts[..=i].join("/");
            let dir_lower = dir.to_ascii_lowercase();
            let leaf = parts[i].to_ascii_lowercase();
            if leaf == target_lower {
                if let Some(ref must) = must_lower {
                    if !dir_lower.contains(must) {
                        continue;
                    }
                }
                if !candidates.contains(&dir) {
                    candidates.push(dir);
                }
            }
        }
    }
    candidates.sort_by_key(|s| s.len());
    candidates.into_iter().next()
}

fn find_compatible_objects_dir(entries: &[String]) -> Option<(String, String)> {
    let mut best: Option<(String, String)> = None;
    for entry in entries {
        let parts: Vec<&str> = entry.split('/').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let segment_lower = parts[i].to_ascii_lowercase();
            if segment_lower.contains("compatible") && segment_lower.contains("object") {
                let dir = parts[..=i].join("/");
                // If there is an inner `objects` directory inside this compatible folder, point to it
                let inner_objects = format!("{dir}/objects");
                let has_inner_objects = entries.iter().any(|e| {
                    e.to_ascii_lowercase()
                        .starts_with(&format!("{}/", inner_objects.to_ascii_lowercase()))
                });
                let subpath = if has_inner_objects {
                    inner_objects
                } else {
                    dir.clone()
                };
                let label = parts[i].to_string();
                if best.as_ref().is_none_or(|(b, _)| subpath.len() < b.len()) {
                    best = Some((subpath, label));
                }
            }
        }
    }
    best
}

fn resolve_aircraft_dir(xplane_path: &str, aircraft_folder: &str) -> Result<PathBuf> {
    let root = Path::new(xplane_path).join("Aircraft").join(aircraft_folder);
    if !root.is_dir() {
        return Err(anyhow!("Aircraft folder not found: {}", root.display()));
    }
    Ok(fs::canonicalize(root)?)
}

fn is_default_engine_obj(stl_value: &str, keyword: &str) -> bool {
    let normalized = stl_value.trim().replace('\\', "/").to_ascii_lowercase();
    let file_name = normalized.rsplit('/').next().unwrap_or(&normalized);
    let stem = file_name.strip_suffix(".obj").unwrap_or(file_name);
    // Matches "PW.obj", "767/PW.obj", "GE.obj", "767/GE.obj", "PW_engines.obj" etc.,
    // but does NOT match our own newly installed "PW4000L.obj" / "CF6.obj".
    if stem == keyword {
        return true;
    }
    if keyword == "pw" && (stem == "pw_eng" || stem == "pw_engines") {
        return true;
    }
    if keyword == "ge" && (stem == "ge_eng" || stem == "ge_engines") {
        return true;
    }
    false
}

/// Inspect an engine mod folder or archive against the target aircraft.
pub fn inspect_engine_mod(
    source_path: &str,
    xplane_path: &str,
    aircraft_folder: &str,
) -> Result<EngineModInspection> {
    let source = Path::new(source_path);
    let aircraft_dir = resolve_aircraft_dir(xplane_path, aircraft_folder)?;
    let (is_archive, entries) = collect_source_entries(source)?;
    let detected_preset = detect_preset_from_entries(source, &entries);

    let main_mod_objects_subpath = find_subpath_ending(&entries, "objects", Some("main mod"))
        .or_else(|| find_subpath_ending(&entries, "objects", Some("main")))
        .or_else(|| find_subpath_ending(&entries, "objects", None));
    let liveries_subpath = find_subpath_ending(&entries, "liveries", None);
    let (compatible_objects_subpath, compatible_objects_label) =
        match find_compatible_objects_dir(&entries) {
            Some((sub, label)) => (Some(sub), Some(label)),
            None => (None, None),
        };

    let mut acf_variants = Vec::new();
    for entry in fs::read_dir(&aircraft_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if ext != "acf" && ext != "xfma" {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let variant = classify_ff767_acf_variant(&file_name);
        let coordinates = detected_preset.and_then(|p| preset_coords_for_variant(p, variant));
        let supported_by_preset = coordinates.is_some();

        let (has_default_engine_obj, default_engine_obj_name) =
            if let (Some(preset), Ok(content)) = (detected_preset, fs::read_to_string(&path)) {
                let kw = preset_clear_keyword(preset);
                let found = find_matching_obja_stl(&content, kw);
                (found.is_some(), found)
            } else {
                (false, None)
            };

        acf_variants.push(DetectedAcfVariant {
            file_name,
            variant,
            supported_by_preset,
            has_default_engine_obj,
            default_engine_obj_name,
            coordinates,
        });
    }
    acf_variants.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

    Ok(EngineModInspection {
        source_path: source_path.to_string(),
        is_archive,
        detected_preset,
        preset_name: detected_preset.map(|p| preset_display_name(p).to_string()),
        main_mod_objects_subpath,
        liveries_subpath,
        compatible_objects_subpath,
        compatible_objects_label,
        acf_variants,
        objects_to_add: detected_preset
            .map(|p| preset_object_paths(p).iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        clear_engine_keyword: detected_preset.map(|p| preset_clear_keyword(p).to_uppercase()),
        hide_dataref: detected_preset.map(|p| preset_hide_dataref(p).to_string()),
    })
}

fn find_matching_obja_stl(acf_content: &str, keyword: &str) -> Option<String> {
    for line in acf_content.lines() {
        let trimmed = line.trim_end_matches('\r');
        if let Some(rest) = trimmed.strip_prefix("P _obja/") {
            if let Some((_, prop_and_val)) = rest.split_once('/') {
                if let Some(val) = prop_and_val.strip_prefix("_v10_att_file_stl ") {
                    if is_default_engine_obj(val, keyword) {
                        return Some(val.trim().to_string());
                    }
                }
            }
        }
    }
    None
}

/// Patch the `P _obja/...` section of an X-Plane `.acf` text file:
/// 1. Parses all `P _obja/<idx>/<prop> <val>` entries while keeping all other lines byte-identical.
/// 2. Removes any existing entry matching `clear_keyword` (e.g. default `PW` or `GE`) OR any
///    previously installed copy of the incoming `add_objects` (making patching idempotent!).
/// 3. Appends the new engine objects with exact coordinates, `hide_dataref`, and cloned `_obj_flags`
///    (defaulting to `17` = Outside + All Views + Hi-Res if no template entry was found).
/// 4. Re-indexes `0..N-1` cleanly and updates `P _obja/count N`.
pub fn patch_acf_objects_content(
    acf_content: &str,
    clear_keyword: &str,
    add_objects: &[EngineModObjectSpec],
) -> Result<String> {
    let line_ending = if acf_content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let mut non_obja_before: Vec<String> = Vec::new();
    let mut non_obja_after: Vec<String> = Vec::new();
    let mut seen_obja = false;
    let mut obja_entries: BTreeMap<usize, BTreeMap<String, String>> = BTreeMap::new();

    for raw_line in acf_content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix("P _obja/") {
            seen_obja = true;
            if rest.starts_with("count ") {
                continue;
            }
            if let Some((idx_str, prop_and_val)) = rest.split_once('/') {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    let (prop, val) = match prop_and_val.split_once(' ') {
                        Some((p, v)) => (p.to_string(), v.to_string()),
                        None => (prop_and_val.to_string(), String::new()),
                    };
                    obja_entries.entry(idx).or_default().insert(prop, val);
                    continue;
                }
            }
        }
        if !seen_obja {
            non_obja_before.push(line.to_string());
        } else {
            non_obja_after.push(line.to_string());
        }
    }

    if !seen_obja {
        return Err(anyhow!(
            "Invalid or unsupported .acf file: no P _obja section found"
        ));
    }

    let incoming_paths_lower: HashSet<String> = add_objects
        .iter()
        .map(|o| o.obj_path.replace('\\', "/").to_ascii_lowercase())
        .collect();

    // Capture template properties (especially `_obj_flags`) from the default engine object being replaced
    let mut template_props: Option<BTreeMap<String, String>> = None;
    let mut fallback_exterior_flags: Option<String> = None;

    let mut kept_entries: Vec<BTreeMap<String, String>> = Vec::new();
    for (_idx, props) in obja_entries {
        let stl = props
            .get("_v10_att_file_stl")
            .map(String::as_str)
            .unwrap_or_default();
        let stl_lower = stl.replace('\\', "/").to_ascii_lowercase();

        if fallback_exterior_flags.is_none() {
            if let Some(flags) = props.get("_obj_flags") {
                if !stl_lower.contains("cockpit") && !stl_lower.contains("glass") {
                    fallback_exterior_flags = Some(flags.clone());
                }
            }
        }

        if is_default_engine_obj(stl, clear_keyword) {
            if template_props.is_none() {
                template_props = Some(props);
            }
            continue;
        }
        if incoming_paths_lower.contains(&stl_lower) {
            if template_props.is_none() {
                template_props = Some(props);
            }
            continue;
        }
        kept_entries.push(props);
    }

    let base_flags = template_props
        .as_ref()
        .and_then(|p| p.get("_obj_flags").cloned())
        .or(fallback_exterior_flags)
        .unwrap_or_else(|| "17".to_string());

    for spec in add_objects {
        let mut props = template_props.clone().unwrap_or_default();
        props.insert("_obj_flags".to_string(), base_flags.clone());
        props.insert("_obj_hide_dataref".to_string(), spec.hide_dataref.clone());
        props.insert("_v10_att_file_stl".to_string(), spec.obj_path.clone());
        props.insert("_v10_att_part".to_string(), "-1".to_string());
        props.insert("_v10_att_phi_ref".to_string(), format!("{:.9}", spec.roll));
        props.insert("_v10_att_psi_ref".to_string(), format!("{:.9}", spec.hdg));
        props.insert("_v10_att_the_ref".to_string(), format!("{:.9}", spec.pitch));
        props.insert(
            "_v10_att_x_acf_prt_ref".to_string(),
            format!("{:.9}", spec.lat_arm),
        );
        props.insert(
            "_v10_att_y_acf_prt_ref".to_string(),
            format!("{:.9}", spec.vert_arm),
        );
        props.insert(
            "_v10_att_z_acf_prt_ref".to_string(),
            format!("{:.9}", spec.long_arm),
        );
        kept_entries.push(props);
    }

    let mut out_lines = Vec::with_capacity(
        non_obja_before.len() + non_obja_after.len() + kept_entries.len() * 11 + 2,
    );
    out_lines.extend(non_obja_before);

    for (new_idx, props) in kept_entries.iter().enumerate() {
        for (prop, val) in props {
            if val.is_empty() {
                out_lines.push(format!("P _obja/{new_idx}/{prop}"));
            } else {
                out_lines.push(format!("P _obja/{new_idx}/{prop} {val}"));
            }
        }
    }
    out_lines.push(format!("P _obja/count {}", kept_entries.len()));
    out_lines.extend(non_obja_after);

    let mut result = out_lines.join(line_ending);
    if acf_content.ends_with('\n') {
        result.push_str(line_ending);
    }
    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
struct BackupEntry {
    backup: String,
    original: String,
    #[serde(default)]
    checksum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddedFile {
    path: String,
    checksum: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PatchBackupManifest {
    entries: Vec<BackupEntry>,
    #[serde(default)]
    added_files: Option<Vec<AddedFile>>,
}

fn sha256_file(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

fn extract_archive_to_dir(archive: &Path, dest: &Path) -> Result<()> {
    let ext = archive
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "zip" => {
            let file = fs::File::open(archive)?;
            let mut zip = zip::ZipArchive::new(file)?;
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i)?;
                let Some(rel) =
                    crate::package_artifacts::normalize_safe_relative_path(entry.name())
                else {
                    continue;
                };
                let out = dest.join(rel);
                if entry.is_dir() {
                    fs::create_dir_all(&out)?;
                } else {
                    if let Some(p) = out.parent() {
                        fs::create_dir_all(p)?;
                    }
                    let mut out_file = fs::File::create(&out)?;
                    std::io::copy(&mut entry, &mut out_file)?;
                }
            }
            Ok(())
        }
        "7z" => {
            sevenz_rust2::decompress_file(archive, dest)
                .map_err(|e| anyhow!("Failed to extract 7z archive: {}", e))?;
            Ok(())
        }
        "rar" => {
            let mut rar = unrar::Archive::new(archive)
                .open_for_processing()
                .map_err(|e| anyhow!("Failed to open RAR archive: {:?}", e))?;
            while let Some(header) = rar
                .read_header()
                .map_err(|e| anyhow!("RAR read error: {:?}", e))?
            {
                let entry_name = header.entry().filename.to_string_lossy().into_owned();
                if let Some(rel) =
                    crate::package_artifacts::normalize_safe_relative_path(&entry_name)
                {
                    let out = dest.join(rel);
                    if header.entry().is_directory() {
                        fs::create_dir_all(&out)?;
                        rar = header
                            .skip()
                            .map_err(|e| anyhow!("RAR skip error: {:?}", e))?;
                    } else {
                        if let Some(p) = out.parent() {
                            fs::create_dir_all(p)?;
                        }
                        rar = header
                            .extract_to(&out)
                            .map_err(|e| anyhow!("RAR extract error: {:?}", e))?;
                    }
                } else {
                    rar = header
                        .skip()
                        .map_err(|e| anyhow!("RAR skip error: {:?}", e))?;
                }
            }
            Ok(())
        }
        _ => Err(anyhow!("Unsupported archive format: {}", ext)),
    }
}

/// Apply an engine mod (copy objects + optional liveries/compatible objects + patch `.acf` files)
/// with full `_dhub_patch_backups` backup support for 1-click revert.
pub fn apply_engine_mod(req: EngineModApplyRequest) -> Result<EngineModApplyResult> {
    let source = Path::new(&req.source_path);
    let aircraft_dir = resolve_aircraft_dir(&req.xplane_path, &req.aircraft_folder)?;
    let inspection = inspect_engine_mod(&req.source_path, &req.xplane_path, &req.aircraft_folder)?;

    let temp_extract = if inspection.is_archive {
        let dir = tempfile::tempdir()?;
        extract_archive_to_dir(source, dir.path())?;
        Some(dir)
    } else {
        None
    };
    let source_root = temp_extract
        .as_ref()
        .map(|d| d.path())
        .unwrap_or(source);

    // Collect all file copies: (src_file, dest_file_relative_to_aircraft)
    let mut planned_copies: Vec<(PathBuf, PathBuf)> = Vec::new();

    let mut queue_copy_dir = |subpath: &str, dest_prefix: &str| -> Result<()> {
        let from_dir = source_root.join(subpath);
        if !from_dir.is_dir() {
            return Ok(());
        }
        for entry in WalkDir::new(&from_dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            if crate::package_artifacts::is_ignored_package_artifact_path(entry.path()) {
                continue;
            }
            let rel = entry.path().strip_prefix(&from_dir)?;
            let dest_rel = if dest_prefix.is_empty() {
                rel.to_path_buf()
            } else {
                Path::new(dest_prefix).join(rel)
            };
            planned_copies.push((entry.path().to_path_buf(), dest_rel));
        }
        Ok(())
    };

    if let Some(ref main_objects) = inspection.main_mod_objects_subpath {
        queue_copy_dir(main_objects, "objects")?;
    } else {
        return Err(anyhow!(
            "Could not find an 'objects' folder inside the selected engine mod source"
        ));
    }

    if req.install_compatible_objects {
        if let Some(ref compat) = inspection.compatible_objects_subpath {
            queue_copy_dir(compat, "objects")?;
        }
    }

    if req.install_liveries {
        if let Some(ref liv) = inspection.liveries_subpath {
            queue_copy_dir(liv, "liveries")?;
        }
    }

    // Deduplicate copies (later entries like compatible_objects override earlier ones)
    let mut dedup_copies: HashMap<String, (PathBuf, PathBuf)> = HashMap::new();
    for (src_file, dest_rel) in planned_copies {
        let key = dest_rel.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
        dedup_copies.insert(key, (src_file, dest_rel));
    }

    // Prepare patched `.acf` contents BEFORE writing anything to disk
    let clear_kw = preset_clear_keyword(req.preset_id);
    let hide_dataref = preset_hide_dataref(req.preset_id);
    let obj_paths = preset_object_paths(req.preset_id);

    let mut planned_acf_writes: Vec<(String, PathBuf, String)> = Vec::new();
    for acf_name in &req.selected_acf_files {
        let acf_path = aircraft_dir.join(acf_name);
        if !acf_path.is_file() {
            continue;
        }
        let variant = classify_ff767_acf_variant(acf_name);
        let Some((long_arm, lat_arm, vert_arm)) = preset_coords_for_variant(req.preset_id, variant)
        else {
            continue;
        };
        let specs: Vec<EngineModObjectSpec> = obj_paths
            .iter()
            .map(|obj| EngineModObjectSpec {
                obj_path: (*obj).to_string(),
                long_arm,
                lat_arm,
                vert_arm,
                hdg: 0.0,
                pitch: 0.0,
                roll: 0.0,
                hide_dataref: hide_dataref.to_string(),
            })
            .collect();
        let original = fs::read_to_string(&acf_path)
            .with_context(|| format!("Failed to read {}", acf_path.display()))?;
        let patched = patch_acf_objects_content(&original, clear_kw, &specs)
            .with_context(|| format!("Failed to patch {}", acf_path.display()))?;
        planned_acf_writes.push((acf_name.clone(), acf_path, patched));
    }

    // Create revertable backup session in `_dhub_patch_backups`
    let mut backup_session_id = None;
    let mut manifest = PatchBackupManifest::default();
    let mut added_files = Vec::new();
    let backup_dir_opt = if req.backup_enabled {
        let session_id = format!(
            "engine_{}_{}",
            match req.preset_id {
                EngineModPresetId::Ff767MangoPw4000 => "FF767_PW4000",
                EngineModPresetId::Ff767MangoGeCf6 => "FF767_GE_CF6",
            },
            Uuid::new_v4()
        );
        let dir = aircraft_dir.join("_dhub_patch_backups").join(&session_id);
        fs::create_dir_all(dir.join("files"))?;
        backup_session_id = Some(session_id);
        Some(dir)
    } else {
        None
    };

    // Backup existing files before overwriting
    if let Some(ref backup_dir) = backup_dir_opt {
        for (src_file, dest_rel) in dedup_copies.values() {
            let dest_abs = aircraft_dir.join(dest_rel);
            if dest_abs.is_file() {
                let bk_path = backup_dir.join("files").join(dest_rel);
                if let Some(p) = bk_path.parent() {
                    fs::create_dir_all(p)?;
                }
                fs::copy(&dest_abs, &bk_path)?;
                manifest.entries.push(BackupEntry {
                    backup: bk_path.to_string_lossy().into_owned(),
                    original: dest_abs.to_string_lossy().into_owned(),
                    checksum: Some(sha256_file(&bk_path)?),
                });
            } else {
                added_files.push(AddedFile {
                    path: dest_rel.to_string_lossy().replace('\\', "/"),
                    checksum: sha256_file(src_file)?,
                });
            }
        }
        for (acf_name, acf_abs, _) in &planned_acf_writes {
            let bk_path = backup_dir.join("files").join(acf_name);
            if let Some(p) = bk_path.parent() {
                fs::create_dir_all(p)?;
            }
            fs::copy(acf_abs, &bk_path)?;
            manifest.entries.push(BackupEntry {
                backup: bk_path.to_string_lossy().into_owned(),
                original: acf_abs.to_string_lossy().into_owned(),
                checksum: Some(sha256_file(&bk_path)?),
            });
        }
        manifest.added_files = Some(added_files);
        fs::write(
            backup_dir.join("manifest_patch.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
    }

    // Write copied files
    let mut copied_files = 0usize;
    for (src_file, dest_rel) in dedup_copies.values() {
        let dest_abs = aircraft_dir.join(dest_rel);
        if let Some(p) = dest_abs.parent() {
            fs::create_dir_all(p)?;
        }
        fs::copy(src_file, &dest_abs)?;
        copied_files += 1;
    }

    // Write patched `.acf` files
    let mut patched_acf_files = Vec::new();
    for (acf_name, acf_abs, patched_content) in planned_acf_writes {
        fs::write(&acf_abs, patched_content)?;
        patched_acf_files.push(acf_name);
    }

    Ok(EngineModApplyResult {
        copied_files,
        patched_acf_files,
        backup_session_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_acf_obja_cleanly_and_updates_count() {
        let sample = "\
I
1200 Version
ACF
P _obja/0/_obj_flags 17
P _obja/0/_v10_att_file_stl fuselage.obj
P _obja/1/_obj_flags 17
P _obja/1/_obj_hide_dataref 1-sim/object/kill/PW
P _obja/1/_v10_att_file_stl 767/PW.obj
P _obja/1/_v10_att_part -1
P _obja/1/_v10_att_z_acf_prt_ref 188.970000000
P _obja/count 2
P acf/_name Boeing 767-200ER
";
        let specs: Vec<EngineModObjectSpec> = preset_object_paths(EngineModPresetId::Ff767MangoPw4000)
            .iter()
            .map(|p| EngineModObjectSpec {
                obj_path: (*p).to_string(),
                long_arm: 188.97,
                lat_arm: 0.0,
                vert_arm: 26.20,
                hdg: 0.0,
                pitch: 0.0,
                roll: 0.0,
                hide_dataref: "1-sim/object/kill/PW".to_string(),
            })
            .collect();
        let patched = patch_acf_objects_content(sample, "pw", &specs).unwrap();
        assert!(!patched.contains("767/PW.obj"));
        assert!(patched.contains("P _obja/1/_v10_att_file_stl 767/PW Engines/PW4000L.obj"));
        assert!(patched.contains("P _obja/2/_v10_att_file_stl 767/PW Engines/PW4000R.obj"));
        assert!(patched.contains("P _obja/3/_v10_att_file_stl 767/PW Engines/N1_fast.obj"));
        assert!(patched.contains("P _obja/count 4"));
        assert!(patched.contains("P acf/_name Boeing 767-200ER"));
    }
}
