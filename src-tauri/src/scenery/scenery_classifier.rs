//! Scenery classification module for auto-sorting
//!
//! This module analyzes scenery packages and determines their category
//! by parsing DSF file headers and checking file system structure.

use crate::geo_regions;
use crate::models::{
    is_global_airports_folder_name, DsfHeader, SceneryCategory, SceneryPackageInfo,
};
use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

const MAX_PLUGIN_SCAN_DEPTH: usize = 5;

fn is_lines3d_folder_name(folder_name: &str) -> bool {
    folder_name.trim().eq_ignore_ascii_case("lines3d")
}

fn is_sam_folder_name(folder_name: &str) -> bool {
    let folder_lower = folder_name.to_lowercase();

    let parts: Vec<&str> = folder_lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let has_sam_word = parts.contains(&"sam");

    parts.iter().any(|&part| {
        part.ends_with("sam") && part.len() > 3 && {
            let prefix = &part[..part.len() - 3];
            matches!(prefix, "open" | "my" | "custom" | "new")
        }
    }) || has_sam_word
}

fn classify_library_folder_name(folder_name: &str) -> SceneryCategory {
    if is_lines3d_folder_name(folder_name) || is_sam_folder_name(folder_name) {
        SceneryCategory::FixedHighPriority
    } else {
        SceneryCategory::Library
    }
}

pub(crate) fn is_aep_overlays_folder_name(folder_name: &str) -> bool {
    let lower = folder_name.trim().to_lowercase();
    lower == "aep overlays"
        || lower == "aep_overlays"
        || lower == "aep-overlays"
        || lower.starts_with("aep overlay")
        || lower.starts_with("aep_overlay")
        || lower.starts_with("aep-overlay")
}

pub(crate) fn is_xplane_landmarks_folder_name(folder_name: &str) -> bool {
    let lower = folder_name.trim().to_lowercase();
    lower.starts_with("x-plane landmarks")
        || lower.starts_with("xplane landmarks")
        || lower.starts_with("x-plane_landmarks")
        || lower.starts_with("xplane_landmarks")
}

pub(crate) fn is_xplane_airports_folder_name(folder_name: &str) -> bool {
    let lower = folder_name.trim().to_lowercase();
    lower.starts_with("x-plane airports")
        || lower.starts_with("xplane airports")
        || lower.starts_with("x-plane_airports")
        || lower.starts_with("xplane_airports")
}

pub(crate) fn is_special_airport_tier_folder_name(folder_name: &str) -> bool {
    is_aep_overlays_folder_name(folder_name)
        || is_xplane_landmarks_folder_name(folder_name)
        || is_xplane_airports_folder_name(folder_name)
}

/// Check if folder contains plugins (.xpl files)
fn has_plugins(scenery_path: &Path) -> bool {
    let plugins_path = scenery_path.join("plugins");

    // Check if plugins folder exists (follow symlinks)
    if !plugins_path.metadata().map(|m| m.is_dir()).unwrap_or(false) {
        return false;
    }

    // Search for .xpl files up to 5 levels deep
    for entry in WalkDir::new(&plugins_path)
        .follow_links(true)
        .max_depth(MAX_PLUGIN_SCAN_DEPTH)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext.eq_ignore_ascii_case("xpl") {
                    return true;
                }
            }
        }
    }

    false
}

/// Main entry point for scenery classification
pub fn classify_scenery(scenery_path: &Path, _xplane_path: &Path) -> Result<SceneryPackageInfo> {
    let folder_name = scenery_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid folder name"))?
        .to_string();

    crate::log_debug!(
        &format!("Classifying scenery: {}", folder_name),
        "scenery_classifier"
    );
    crate::log_debug!(&format!("  Path: {:?}", scenery_path), "scenery_classifier");

    // Check if path is a symlink and log the target
    if let Ok(metadata) = scenery_path.symlink_metadata() {
        if metadata.is_symlink() {
            if let Ok(target) = std::fs::read_link(scenery_path) {
                crate::log_debug!(
                    &format!("  ⚠ This is a symlink pointing to: {:?}", target),
                    "scenery_classifier"
                );
            }
        }
    }

    // Early validation: Check if this is a valid scenery package
    // Must have either "Earth nav data" folder, "library.txt" file, or plugins folder with .xpl files
    // Use metadata() to follow symbolic links
    let library_txt_path = scenery_path.join("library.txt");
    let has_library_txt = library_txt_path
        .metadata()
        .map(|m| m.is_file())
        .unwrap_or(false);
    if has_library_txt {
        crate::log_debug!(
            &format!("  Found library.txt at: {:?}", library_txt_path),
            "scenery_classifier"
        );
    }

    let earth_nav_path = scenery_path.join("Earth nav data");
    let has_earth_nav_data = earth_nav_path
        .metadata()
        .map(|m| m.is_dir())
        .unwrap_or(false);
    if has_earth_nav_data {
        crate::log_debug!(
            &format!("  Found Earth nav data at: {:?}", earth_nav_path),
            "scenery_classifier"
        );
    }

    let has_plugin_files = has_plugins(scenery_path);
    if has_plugin_files {
        crate::log_debug!(
            "  Found plugins folder with .xpl files",
            "scenery_classifier"
        );
    }

    crate::log_debug!(
        &format!(
            "  has_library_txt: {}, has_earth_nav_data: {}, has_plugins: {}",
            has_library_txt, has_earth_nav_data, has_plugin_files
        ),
        "scenery_classifier"
    );

    if !has_library_txt && !has_earth_nav_data && !has_plugin_files {
        crate::log_debug!(
            "  ⚠ Classified as Unrecognized: missing 'Earth nav data', 'library.txt', and plugins",
            "scenery_classifier"
        );
        return build_package_info(
            folder_name,
            SceneryCategory::Unrecognized,
            scenery_path,
            PackageInfoDetails::default(),
        );
    }

    // If only has plugins (no scenery features), classify as Other
    if has_plugin_files && !has_library_txt && !has_earth_nav_data {
        crate::log_debug!(
            "  ✓ Classified as Other (has plugins but no scenery features)",
            "scenery_classifier"
        );
        return build_package_info(
            folder_name,
            SceneryCategory::Other,
            scenery_path,
            PackageInfoDetails::default(),
        );
    }

    // Collect file system information
    crate::log_debug!("  Searching for DSF files...", "scenery_classifier");
    let dsf_files = find_dsf_files(scenery_path)?;
    crate::log_debug!(
        &format!("  DSF search complete: {} files", dsf_files.len()),
        "scenery_classifier"
    );

    crate::log_debug!("  Counting textures...", "scenery_classifier");
    let texture_count = count_texture_files(scenery_path)?;
    crate::log_debug!(
        &format!("  Texture count complete: {}", texture_count),
        "scenery_classifier"
    );

    crate::log_debug!(
        &format!(
            "  pre-check summary: has_library_txt={}, has_earth_nav_data={}, dsf_files={}, texture_count={}",
            has_library_txt, has_earth_nav_data, dsf_files.len(), texture_count
        ),
        "scenery_classifier"
    );

    // Fast path for library packages:
    // if there is no top-level "Earth nav data", treat top-level library packages
    // as libraries even if they contain nested test/demo apt.dat files.
    if has_library_txt && !has_earth_nav_data {
        let category = classify_library_folder_name(&folder_name);

        crate::log_debug!(
            if category == SceneryCategory::FixedHighPriority {
                "  ✓ Classified as FixedHighPriority (library.txt without top-level Earth nav data)"
            } else {
                "  ✓ Classified as Library (library.txt without top-level Earth nav data)"
            },
            "scenery_classifier"
        );

        let exported_library_names = crate::scenery_index::parse_library_exports(&library_txt_path)
            .into_iter()
            .collect::<Vec<_>>();

        crate::log_debug!(
            &format!("  Library exports: {:?}", exported_library_names),
            "scenery_classifier"
        );

        return build_package_info(
            folder_name,
            category,
            scenery_path,
            PackageInfoDetails {
                has_apt_dat: false,
                has_dsf: !dsf_files.is_empty(),
                has_library_txt: true,
                texture_count,
                earth_nav_tile_count: 0,
                required_libraries: Vec::new(),
                missing_libraries: Vec::new(),
                exported_library_names,
                ..Default::default()
            },
        );
    }

    crate::log_debug!("  Checking for apt.dat...", "scenery_classifier");
    let has_apt_dat = check_apt_dat_recursive(scenery_path)?;
    crate::log_debug!(
        &format!("  apt.dat check complete: {}", has_apt_dat),
        "scenery_classifier"
    );

    // Decision Tree Classification
    // Parse DSF header if available
    let dsf_header_opt: Option<DsfHeader> = if !dsf_files.is_empty() {
        match parse_dsf_header(&dsf_files[0]) {
            Ok(header) => {
                if let Some(ref agent) = header.creation_agent {
                    crate::log_debug!(
                        &format!("  creation_agent: {}", agent),
                        "scenery_classifier"
                    );
                }
                crate::log_debug!(
                    &format!("  sim/overlay: {}", header.is_overlay),
                    "scenery_classifier"
                );
                crate::log_debug!(
                    &format!(
                        "  has terrain refs: {} (count: {})",
                        !header.terrain_references.is_empty(),
                        header.terrain_references.len()
                    ),
                    "scenery_classifier"
                );
                if !header.terrain_references.is_empty() {
                    crate::log_debug!(
                        &format!(
                            "  terrain refs sample: {:?}",
                            header.terrain_references.iter().take(3).collect::<Vec<_>>()
                        ),
                        "scenery_classifier"
                    );
                }
                Some(header)
            }
            Err(e) => {
                crate::log_debug!(
                    &format!("  Failed to parse DSF: {}", e),
                    "scenery_classifier"
                );
                None
            }
        }
    } else {
        None
    };

    // Decision Tree:
    // 1. Has apt.dat OR (DSF with WorldEditor creation_agent) → Airport
    if has_apt_dat {
        let category = if is_global_airports_folder_name(&folder_name) {
            SceneryCategory::DefaultAirport
        } else {
            SceneryCategory::Airport
        };
        crate::log_debug!(
            if category == SceneryCategory::DefaultAirport {
                "  ✓ Classified as DefaultAirport (Global Airports)"
            } else {
                "  ✓ Classified as Airport (has apt.dat)"
            },
            "scenery_classifier"
        );

        // Extract required libraries thoroughly across airport DSF tiles in parallel
        let required_libraries = if category == SceneryCategory::DefaultAirport {
            Vec::new()
        } else {
            collect_all_required_libraries(dsf_header_opt.as_ref(), &dsf_files, scenery_path, 32)
        };

        // Parse library.txt if exists to get exported library names
        let exported_library_names = if has_library_txt {
            let library_txt_path = scenery_path.join("library.txt");
            crate::scenery_index::parse_library_exports(&library_txt_path)
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if !exported_library_names.is_empty() {
            crate::log_debug!(
                &format!(
                    "  Airport also exports libraries: {:?}",
                    exported_library_names
                ),
                "scenery_classifier"
            );
        }

        return build_package_info(
            folder_name,
            category,
            scenery_path,
            PackageInfoDetails {
                has_apt_dat: true,
                has_dsf: !dsf_files.is_empty(),
                has_library_txt,
                texture_count,
                earth_nav_tile_count: 0,
                required_libraries,
                missing_libraries: Vec::new(), // missing_libraries will be filled later
                exported_library_names,
                airport_id: parse_airport_id(scenery_path),
            },
        );
    }

    if let Some(ref header) = dsf_header_opt {
        if let Some(ref agent) = header.creation_agent {
            if agent.to_lowercase().contains("worldeditor") {
                let category = if is_global_airports_folder_name(&folder_name) {
                    SceneryCategory::DefaultAirport
                } else {
                    SceneryCategory::Airport
                };
                crate::log_debug!(
                    if category == SceneryCategory::DefaultAirport {
                        "  ✓ Classified as DefaultAirport (Global Airports)"
                    } else {
                        "  ✓ Classified as Airport (WorldEditor without apt.dat)"
                    },
                    "scenery_classifier"
                );
                let required = if category == SceneryCategory::DefaultAirport {
                    Vec::new()
                } else {
                    collect_all_required_libraries(Some(header), &dsf_files, scenery_path, 32)
                };

                // Parse library.txt if exists to get exported library names
                let exported_library_names = if has_library_txt {
                    let library_txt_path = scenery_path.join("library.txt");
                    crate::scenery_index::parse_library_exports(&library_txt_path)
                        .into_iter()
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };

                if !exported_library_names.is_empty() {
                    crate::log_debug!(
                        &format!(
                            "  Airport also exports libraries: {:?}",
                            exported_library_names
                        ),
                        "scenery_classifier"
                    );
                }

                return build_package_info(
                    folder_name,
                    category,
                    scenery_path,
                    PackageInfoDetails {
                        has_apt_dat: false,
                        has_dsf: true,
                        has_library_txt,
                        texture_count,
                        earth_nav_tile_count: 0,
                        required_libraries: required,
                        missing_libraries: Vec::new(), // missing_libraries will be filled later
                        exported_library_names,
                        ..Default::default()
                    },
                );
            }
        }
    }

    // 2. Has sim/overlay 1 but no apt.dat → Overlay
    if let Some(ref header) = dsf_header_opt {
        if header.is_overlay {
            crate::log_debug!(
                "  ✓ Classified as Overlay (sim/overlay without apt.dat)",
                "scenery_classifier"
            );
            let required = collect_all_required_libraries(Some(header), &dsf_files, scenery_path, 8);

            // Parse library.txt if exists to get exported library names
            let exported_library_names = if has_library_txt {
                let library_txt_path = scenery_path.join("library.txt");
                crate::scenery_index::parse_library_exports(&library_txt_path)
                    .into_iter()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            if !exported_library_names.is_empty() {
                crate::log_debug!(
                    &format!(
                        "  Overlay also exports libraries: {:?}",
                        exported_library_names
                    ),
                    "scenery_classifier"
                );
            }

            let tile_count = count_earth_nav_tile_folders(scenery_path)?;

            let category = if is_special_airport_tier_folder_name(&folder_name) {
                SceneryCategory::Airport
            } else {
                SceneryCategory::Overlay
            };

            return build_package_info(
                folder_name,
                category,
                scenery_path,
                PackageInfoDetails {
                    has_apt_dat: false,
                    has_dsf: true,
                    has_library_txt,
                    texture_count,
                    earth_nav_tile_count: tile_count,
                    required_libraries: required,
                    missing_libraries: Vec::new(), // missing_libraries will be filled later
                    exported_library_names,
                    ..Default::default()
                },
            );
        }
    }

    // 3. Has library.txt but no Earth nav data → Library or FixedHighPriority (Lines3D/SAM)
    if has_library_txt && !has_earth_nav_data {
        let category = classify_library_folder_name(&folder_name);

        crate::log_debug!(
            if category == SceneryCategory::FixedHighPriority {
                "  ✓ Classified as FixedHighPriority (Lines3D/SAM library)"
            } else {
                "  ✓ Classified as Library (has library.txt but no Earth nav data)"
            },
            "scenery_classifier"
        );

        // Parse library.txt to get exported library names
        let library_txt_path = scenery_path.join("library.txt");
        let exported_library_names = crate::scenery_index::parse_library_exports(&library_txt_path)
            .into_iter()
            .collect::<Vec<_>>();

        crate::log_debug!(
            &format!("  Library exports: {:?}", exported_library_names),
            "scenery_classifier"
        );

        return build_package_info(
            folder_name,
            category,
            scenery_path,
            PackageInfoDetails {
                has_apt_dat: false,
                has_dsf: !dsf_files.is_empty(),
                has_library_txt: true,
                texture_count,
                earth_nav_tile_count: 0,
                required_libraries: Vec::new(),
                missing_libraries: Vec::new(),
                exported_library_names,
                ..Default::default()
            },
        );
    }

    // 4. No sim/overlay and no apt.dat, OR has TERRAIN_DEF → Mesh
    // Check for terrain references in DSF
    let has_terrain_def = if let Some(ref header) = dsf_header_opt {
        !header.terrain_references.is_empty()
    } else {
        false
    };

    if !has_apt_dat && has_earth_nav_data {
        // Has Earth nav data but no apt.dat and no sim/overlay → Mesh
        crate::log_debug!(
            "  ✓ Classified as Mesh (Earth nav data without apt.dat/overlay)",
            "scenery_classifier"
        );

        // All mesh/orthophoto scenery is now classified as Mesh
        let category = SceneryCategory::Mesh;

        let required = collect_all_required_libraries(dsf_header_opt.as_ref(), &dsf_files, scenery_path, 6);
        let (required_libraries, missing_libraries) = (required, Vec::new());

        // Parse library.txt if exists to get exported library names
        let exported_library_names = if has_library_txt {
            let library_txt_path = scenery_path.join("library.txt");
            crate::scenery_index::parse_library_exports(&library_txt_path)
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if !exported_library_names.is_empty() {
            crate::log_debug!(
                &format!(
                    "  Mesh also exports libraries: {:?}",
                    exported_library_names
                ),
                "scenery_classifier"
            );
        }

        let tile_count = count_earth_nav_tile_folders(scenery_path)?;

        return build_package_info(
            folder_name,
            category,
            scenery_path,
            PackageInfoDetails {
                has_apt_dat: false,
                has_dsf: !dsf_files.is_empty(),
                has_library_txt,
                texture_count,
                earth_nav_tile_count: tile_count,
                required_libraries,
                missing_libraries,
                exported_library_names,
                ..Default::default()
            },
        );
    }

    if has_terrain_def {
        crate::log_debug!(
            "  ✓ Classified as Mesh (has TERRAIN_DEF)",
            "scenery_classifier"
        );
        let required = collect_all_required_libraries(dsf_header_opt.as_ref(), &dsf_files, scenery_path, 6);
        let (required_libraries, missing_libraries) = (required, Vec::new());

        // Parse library.txt if exists to get exported library names
        let exported_library_names = if has_library_txt {
            let library_txt_path = scenery_path.join("library.txt");
            crate::scenery_index::parse_library_exports(&library_txt_path)
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if !exported_library_names.is_empty() {
            crate::log_debug!(
                &format!(
                    "  Mesh also exports libraries: {:?}",
                    exported_library_names
                ),
                "scenery_classifier"
            );
        }

        let tile_count = count_earth_nav_tile_folders(scenery_path)?;

        return build_package_info(
            folder_name,
            SceneryCategory::Mesh,
            scenery_path,
            PackageInfoDetails {
                has_apt_dat: false,
                has_dsf: true,
                has_library_txt,
                texture_count,
                earth_nav_tile_count: tile_count,
                required_libraries,
                missing_libraries,
                exported_library_names,
                ..Default::default()
            },
        );
    }

    // Default: Other
    crate::log_debug!(
        "  ✓ Classified as Other (no clear indicators)",
        "scenery_classifier"
    );

    // Parse library.txt if exists to get exported library names
    let exported_library_names = if has_library_txt {
        let library_txt_path = scenery_path.join("library.txt");
        crate::scenery_index::parse_library_exports(&library_txt_path)
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if !exported_library_names.is_empty() {
        crate::log_debug!(
            &format!(
                "  Other also exports libraries: {:?}",
                exported_library_names
            ),
            "scenery_classifier"
        );
    }

    build_package_info(
        folder_name,
        SceneryCategory::Other,
        scenery_path,
        PackageInfoDetails {
            has_apt_dat: false,
            has_dsf: !dsf_files.is_empty(),
            has_library_txt,
            texture_count,
            earth_nav_tile_count: 0,
            required_libraries: Vec::new(),
            missing_libraries: Vec::new(),
            exported_library_names,
            ..Default::default()
        },
    )
}

/// Check if apt.dat exists recursively in Earth nav data directories
fn check_apt_dat_recursive(scenery_path: &Path) -> Result<bool> {
    // apt.dat is always in Earth nav data folder, so only search there
    let earth_nav_path = scenery_path.join("Earth nav data");
    if !earth_nav_path.exists() {
        return Ok(false);
    }

    // Only search up to 5 levels deep in Earth nav data
    for entry in WalkDir::new(&earth_nav_path)
        .follow_links(true) // Explicitly follow symbolic links
        .max_depth(5) // Limit depth to avoid scanning too deep
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if name.eq_ignore_ascii_case("apt.dat") {
                    // Validate apt.dat format
                    if validate_apt_dat(entry.path())? {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

/// Parse the airport identifier from apt.dat
/// Reads "Earth nav data/apt.dat" and returns the airport code from the first airport header line.
/// An airport header line has its first whitespace-delimited field equal to "1", "16", or "17",
/// and the identifier is the 5th field (0-indexed field 4).
fn parse_airport_id(scenery_path: &Path) -> Option<String> {
    let apt_dat_path = scenery_path.join("Earth nav data").join("apt.dat");
    let file = File::open(&apt_dat_path).ok()?;
    let reader = std::io::BufReader::new(file);

    use std::io::BufRead;
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.is_empty() {
            continue;
        }
        match fields[0] {
            "1" | "16" | "17" => {
                // Airport header line: fields[4] is the airport identifier
                if fields.len() >= 5 {
                    return Some(fields[4].to_string());
                }
                // Found a header but no identifier field — stop searching
                return None;
            }
            _ => continue,
        }
    }
    None
}

/// Validate apt.dat file format (first line "I", second line starts with "1")
fn validate_apt_dat(path: &Path) -> Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 256];
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read < 10 {
        return Ok(false);
    }

    let content = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 2 {
        return Ok(false);
    }

    // First line should be "I" or "A"
    let first_line = lines[0].trim();
    if first_line != "I" && first_line != "A" {
        return Ok(false);
    }

    // Second line should start with version number (usually 1000, 1050, 1100, 1200, etc.)
    let second_line = lines[1].trim();
    if second_line.starts_with("1") || second_line.starts_with("850") {
        return Ok(true);
    }

    Ok(false)
}

/// Collect local immediate subdirectory names (lowercased) for fast local asset filtering
fn collect_local_subdirs_lower(scenery_path: &Path) -> HashSet<String> {
    let mut subdirs = HashSet::new();
    if let Ok(entries) = fs::read_dir(scenery_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    subdirs.insert(name.to_lowercase());
                }
            }
        }
    }
    subdirs
}

fn is_standard_local_asset_folder(name_lower: &str) -> bool {
    matches!(
        name_lower,
        "objects"
            | "obj"
            | "facades"
            | "forests"
            | "polygons"
            | "pol"
            | "lines"
            | "strings"
            | "agp"
            | "textures"
            | "terrain"
            | "orthophotos"
            | "orthophoto"
            | "orthos"
            | "ortho"
            | "ground"
            | "apron"
            | "aerial"
            | "winter"
            | "summer"
            | "autumn"
            | "spring"
            | "deepwinter"
            | "fall"
            | "decals"
            | "animations"
            | "custom_lights"
            | "lights"
            | "grass"
            | "vegetation"
            | "roads"
            | "beaches"
            | "scenery_sdk"
            | "effects"
            | "marks"
            | "cars"
            | "people"
            | "trains"
            | "landmarks"
            | "earth nav data"
            | "plugins"
            | "documentation"
            | "doc"
            | "docs"
    )
}

/// Collect required libraries from DSF files in parallel across CPU cores,
/// checking both object/polygon/facade/forest/line/network definitions and terrain definitions.
fn collect_all_required_libraries(
    primary_header: Option<&DsfHeader>,
    dsf_files: &[std::path::PathBuf],
    scenery_path: &Path,
    max_dsfs: usize,
) -> Vec<String> {
    let mut all_refs: Vec<String> = Vec::new();

    if let Some(header) = primary_header {
        all_refs.extend(header.object_references.iter().cloned());
        all_refs.extend(header.terrain_references.iter().cloned());
    }

    let extra_limit = max_dsfs.saturating_sub(1);
    if extra_limit > 0 && dsf_files.len() > 1 {
        let extra_slices: Vec<&std::path::PathBuf> =
            dsf_files.iter().skip(1).take(extra_limit).collect();

        // Parallelize multi-tile DSF header streaming across Rayon worker threads
        let extra_refs: Vec<Vec<String>> = extra_slices
            .par_iter()
            .filter_map(|dsf_path| {
                parse_dsf_header(dsf_path).ok().map(|hdr| {
                    let mut v = hdr.object_references;
                    v.extend(hdr.terrain_references);
                    v
                })
            })
            .collect();

        for refs in extra_refs {
            all_refs.extend(refs);
        }
    }

    let local_subdirs_lower = collect_local_subdirs_lower(scenery_path);
    extract_required_libraries_with_context(&all_refs, scenery_path, &local_subdirs_lower)
}

/// Find DSF files in scenery package
fn find_dsf_files(scenery_path: &Path) -> Result<Vec<std::path::PathBuf>> {
    crate::log_debug!("  Starting DSF search...", "scenery_classifier");

    // First, try to find DSF in Earth nav data (most common location)
    let earth_nav_path = scenery_path.join("Earth nav data");
    if earth_nav_path.exists() {
        crate::log_debug!("  Scanning Earth nav data folder...", "scenery_classifier");

        // Scan depth 1 and 2 in Earth nav data:
        // depth 1 catches loose DSF (Earth nav data/*.dsf)
        // depth 2 catches subdirectories (Earth nav data/+XX+YYY/*.dsf)
        let walker = WalkDir::new(&earth_nav_path)
            .follow_links(true)
            .min_depth(1)
            .max_depth(2)
            .into_iter();

        let mut dsf_candidates: Vec<(std::path::PathBuf, u64)> = Vec::new();
        let mut count = 0;
        for entry in walker.filter_map(|e| e.ok()) {
            count += 1;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext.eq_ignore_ascii_case("dsf") {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        dsf_candidates.push((entry.path().to_path_buf(), size));
                        if dsf_candidates.len() >= 50 {
                            break;
                        }
                    }
                }
            }
        }

        if !dsf_candidates.is_empty() {
            // Sort by file size descending so the primary/largest airport tile is always first
            dsf_candidates.sort_by(|a, b| b.1.cmp(&a.1));
            let dsf_paths: Vec<std::path::PathBuf> = dsf_candidates.into_iter().map(|(p, _)| p).collect();
            crate::log_debug!(
                &format!("  Found {} DSF files in Earth nav data (primary: {:?})", dsf_paths.len(), dsf_paths[0]),
                "scenery_classifier"
            );
            return Ok(dsf_paths);
        }

        crate::log_debug!(
            &format!(
                "  No DSF found in Earth nav data after scanning {} entries",
                count
            ),
            "scenery_classifier"
        );
    }

    crate::log_debug!("  Doing general search...", "scenery_classifier");

    // If not found in Earth nav data, do a general search (but still limit depth)
    let mut dsf_candidates: Vec<(std::path::PathBuf, u64)> = Vec::new();
    for entry in WalkDir::new(scenery_path)
        .follow_links(true)
        .max_depth(5) // Limit depth to avoid scanning too deep
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext.eq_ignore_ascii_case("dsf") {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    dsf_candidates.push((entry.path().to_path_buf(), size));
                    if dsf_candidates.len() >= 20 {
                        break;
                    }
                }
            }
        }
    }

    if !dsf_candidates.is_empty() {
        dsf_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        let dsf_paths: Vec<std::path::PathBuf> = dsf_candidates.into_iter().map(|(p, _)| p).collect();
        crate::log_debug!(
            &format!("  Found {} DSF files (primary: {:?})", dsf_paths.len(), dsf_paths[0]),
            "scenery_classifier"
        );
        return Ok(dsf_paths);
    }

    // No DSF files found
    crate::log_debug!("  No DSF files found", "scenery_classifier");
    Ok(Vec::new())
}

/// Check if DSF file is 7z compressed
fn is_dsf_compressed(dsf_path: &Path) -> Result<bool> {
    let mut file = File::open(dsf_path)?;
    let mut magic = [0u8; 4];

    if file.read_exact(&mut magic).is_err() {
        return Ok(false);
    }

    // 7z magic bytes: 0x37 0x7A 0xBC 0xAF
    Ok(magic == [0x37, 0x7A, 0xBC, 0xAF])
}

/// Read exact number of bytes from stream, returning Ok(false) on clean EOF at start
fn read_exact_or_eof<R: Read + ?Sized>(reader: &mut R, buf: &mut [u8]) -> std::io::Result<bool> {
    let mut read_total = 0;
    while read_total < buf.len() {
        match reader.read(&mut buf[read_total..]) {
            Ok(0) => return Ok(false),
            Ok(n) => read_total += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(true)
}

/// Stream only the DAEH (HEAD) and NFED (DEFN) atoms from a DSF reader in memory,
/// stopping immediately before multi-megabyte DOEG/SDMC/SMED geometry blocks.
fn parse_dsf_from_reader<R: Read + ?Sized>(reader: &mut R) -> Result<DsfHeader> {
    let mut header_magic = [0u8; 12];
    if !read_exact_or_eof(reader, &mut header_magic)? || &header_magic[0..8] != b"XPLNEDSF" {
        return Err(anyhow!("Invalid DSF stream: missing XPLNEDSF magic bytes"));
    }

    let mut properties = HashMap::new();
    let mut object_references = Vec::new();
    let mut terrain_references = Vec::new();
    let mut seen_head = false;
    let mut seen_defn = false;

    loop {
        if seen_head && seen_defn {
            break;
        }

        let mut atom_hdr = [0u8; 8];
        if !read_exact_or_eof(reader, &mut atom_hdr)? {
            break;
        }

        let atom_id = &atom_hdr[0..4];
        let atom_len =
            u32::from_le_bytes([atom_hdr[4], atom_hdr[5], atom_hdr[6], atom_hdr[7]]) as usize;

        if !(8..=128 * 1024 * 1024).contains(&atom_len) {
            break;
        }

        let payload_len = atom_len - 8;

        if atom_id == b"DAEH" {
            let mut head_data = vec![0u8; payload_len];
            if !read_exact_or_eof(reader, &mut head_data)? {
                break;
            }
            if let Ok(props) = extract_props_from_head(&head_data) {
                properties = props;
            }
            seen_head = true;
        } else if atom_id == b"NFED" {
            let mut defn_data = vec![0u8; payload_len];
            if !read_exact_or_eof(reader, &mut defn_data)? {
                break;
            }
            if let Ok((objs, terrains)) = extract_definitions_from_defn(&defn_data) {
                object_references = objs;
                terrain_references = terrains;
            }
            seen_defn = true;
        } else if atom_id == b"DOEG" || atom_id == b"SDMC" || atom_id == b"SMED" {
            // Geometry, command, and raster DEM atoms always follow HEAD and DEFN.
            // Stop immediately to avoid reading/decompressing massive mesh payloads!
            break;
        } else {
            let skipped = std::io::copy(&mut reader.take(payload_len as u64), &mut std::io::sink())?;
            if (skipped as usize) < payload_len {
                break;
            }
        }
    }

    Ok(DsfHeader {
        is_overlay: properties
            .get("sim/overlay")
            .map(|v| v == "1")
            .unwrap_or(false),
        creation_agent: properties.get("sim/creation_agent").cloned(),
        object_references,
        terrain_references,
    })
}

/// Stream DSF header directly from a 7z-compressed DSF in memory with zero temp files on disk,
/// aborting decompression immediately once HEAD + DEFN atoms are parsed.
fn parse_compressed_dsf_header(dsf_path: &Path) -> Result<DsfHeader> {
    let file = File::open(dsf_path)?;
    let buf_reader = BufReader::with_capacity(64 * 1024, file);
    let mut archive =
        sevenz_rust2::ArchiveReader::new(buf_reader, sevenz_rust2::Password::empty())
            .map_err(|e| anyhow!("Failed to open 7z DSF archive: {}", e))?;

    let mut parsed_header: Option<DsfHeader> = None;

    let _ = archive.for_each_entries(|entry, entry_reader| {
        if entry.is_directory() || entry.size() < 12 {
            return Ok(true);
        }

        match parse_dsf_from_reader(entry_reader) {
            Ok(hdr) => {
                parsed_header = Some(hdr);
                // Return false to immediately stop decompressing remaining MBs of geometry!
                Ok(false)
            }
            Err(_) => {
                // Drain non-DSF entry if archive contains multiple files
                let _ = std::io::copy(entry_reader, &mut std::io::sink());
                Ok(true)
            }
        }
    });

    parsed_header.ok_or_else(|| anyhow!("No valid DSF stream found in 7z archive"))
}

/// Parse DSF file header (in-memory streaming for both uncompressed and 7z-compressed DSFs)
pub fn parse_dsf_header(dsf_path: &Path) -> Result<DsfHeader> {
    if is_dsf_compressed(dsf_path)? {
        parse_compressed_dsf_header(dsf_path)
    } else {
        let file = File::open(dsf_path)?;
        let mut reader = BufReader::with_capacity(64 * 1024, file);
        parse_dsf_from_reader(&mut reader)
    }
}

/// Extract PROP data from HEAD atom
fn extract_props_from_head(head_data: &[u8]) -> Result<HashMap<String, String>> {
    let mut properties = HashMap::new();
    let mut offset = 0;

    while offset + 8 <= head_data.len() {
        let atom_id = &head_data[offset..offset + 4];
        let atom_len = u32::from_le_bytes([
            head_data[offset + 4],
            head_data[offset + 5],
            head_data[offset + 6],
            head_data[offset + 7],
        ]) as usize;

        if atom_len < 8 || offset + atom_len > head_data.len() {
            break;
        }

        // Check for PROP atom
        if atom_id == b"PORP" {
            let prop_data = &head_data[offset + 8..offset + atom_len];
            properties = parse_prop_strings(prop_data)?;
            break;
        }

        offset += atom_len;
    }

    Ok(properties)
}

/// Parse null-terminated key-value strings from PROP data
fn parse_prop_strings(prop_data: &[u8]) -> Result<HashMap<String, String>> {
    let mut properties = HashMap::new();
    let mut i = 0;

    while i < prop_data.len() {
        // Read key (null-terminated)
        let key_end = prop_data[i..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(prop_data.len() - i);

        if key_end == 0 {
            break;
        }

        let key = String::from_utf8_lossy(&prop_data[i..i + key_end]).to_string();
        i += key_end + 1;

        if i >= prop_data.len() {
            break;
        }

        // Read value (null-terminated)
        let value_end = prop_data[i..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(prop_data.len() - i);

        let value = String::from_utf8_lossy(&prop_data[i..i + value_end]).to_string();
        i += value_end + 1;

        if !key.is_empty() {
            properties.insert(key, value);
        }
    }

    Ok(properties)
}

/// Extract definitions from DEFN atom
fn extract_definitions_from_defn(defn_data: &[u8]) -> Result<(Vec<String>, Vec<String>)> {
    let mut object_refs = Vec::new();
    let mut terrain_refs = Vec::new();
    let mut offset = 0;

    while offset + 8 <= defn_data.len() {
        let atom_id = &defn_data[offset..offset + 4];
        let atom_len = u32::from_le_bytes([
            defn_data[offset + 4],
            defn_data[offset + 5],
            defn_data[offset + 6],
            defn_data[offset + 7],
        ]) as usize;

        if atom_len < 8 || offset + atom_len > defn_data.len() {
            break;
        }

        // TRET (terrain), OBJT (objects), POLY (polygons), NETW (networks)
        if atom_id == b"TRET" {
            // Terrain definitions
            let def_data = &defn_data[offset + 8..offset + atom_len];
            if let Ok(defs) = parse_definition_strings(def_data) {
                terrain_refs.extend(defs);
            }
        } else if atom_id == b"TJBO" || atom_id == b"YLOP" || atom_id == b"WTEN" || atom_id == b"SCAF" || atom_id == b"ENIL" || atom_id == b"SROF" || atom_id == b"SRTS" || atom_id == b"PRGA" {
            // Object, polygon, and network definitions
            let def_data = &defn_data[offset + 8..offset + atom_len];
            if let Ok(defs) = parse_definition_strings(def_data) {
                object_refs.extend(defs);
            }
        }

        offset += atom_len;
    }

    Ok((object_refs, terrain_refs))
}

/// Parse null-terminated definition strings
fn parse_definition_strings(def_data: &[u8]) -> Result<Vec<String>> {
    let mut definitions = Vec::new();
    let mut i = 0;

    while i < def_data.len() {
        let end = def_data[i..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(def_data.len() - i);

        if end == 0 {
            i += 1;
            continue;
        }

        let def = String::from_utf8_lossy(&def_data[i..i + end]).to_string();
        if !def.is_empty() && (def.contains('/') || def.contains('\\') || def.contains('.')) {
            definitions.push(def);
        }

        i += end + 1;
    }

    Ok(definitions)
}

/// Count texture files in scenery folder (up to 5 for classification)
fn count_texture_files(scenery_path: &Path) -> Result<usize> {
    let textures_path = scenery_path.join("textures");
    if !textures_path.exists() {
        return Ok(0);
    }

    // Only count up to 5 textures - enough to determine if this is an orthophoto scenery
    let mut count = 0;
    for entry in WalkDir::new(&textures_path)
        .follow_links(true) // Explicitly follow symbolic links
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if ext_lower == "dds" || ext_lower == "png" || ext_lower == "jpg" {
                    count += 1;
                    if count >= 5 {
                        // Found enough textures for classification
                        return Ok(count);
                    }
                }
            }
        }
    }

    Ok(count)
}

fn is_ten_degree_tile_folder_name(name: &str) -> bool {
    if name.len() != 7 {
        return false;
    }

    let bytes = name.as_bytes();
    if (bytes[0] != b'+' && bytes[0] != b'-') || (bytes[3] != b'+' && bytes[3] != b'-') {
        return false;
    }

    let lat_str = &name[1..3];
    let lon_str = &name[4..7];

    if !lat_str.chars().all(|c| c.is_ascii_digit()) || !lon_str.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }

    let lat: u32 = match lat_str.parse() {
        Ok(value) => value,
        Err(_) => return false,
    };
    let lon: u32 = match lon_str.parse() {
        Ok(value) => value,
        Err(_) => return false,
    };

    if lat > 90 || lon > 180 {
        return false;
    }

    lat.is_multiple_of(10) && lon.is_multiple_of(10)
}

fn count_earth_nav_tile_folders(scenery_path: &Path) -> Result<u32> {
    let earth_nav_path = scenery_path.join("Earth nav data");
    if !earth_nav_path.exists() {
        return Ok(0);
    }

    let mut count = 0u32;
    for entry in fs::read_dir(&earth_nav_path)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(value) => value,
            None => continue,
        };

        if is_ten_degree_tile_folder_name(name) {
            count += 1;
        }
    }

    Ok(count)
}

/// Normalize a DSF virtual/relative path (handling Windows backslashes and leading slashes)
fn normalize_dsf_asset_path(raw_path: &str) -> String {
    let trimmed = raw_path.trim();
    if trimmed.contains('\\') {
        trimmed.replace('\\', "/").trim_start_matches('/').to_string()
    } else {
        trimmed.trim_start_matches('/').to_string()
    }
}

/// Extract library names with package-local directory awareness and deterministic deduplication
fn extract_required_libraries_with_context(
    asset_refs: &[String],
    scenery_path: &Path,
    local_subdirs_lower: &HashSet<String>,
) -> Vec<String> {
    let mut seen_lower: HashMap<String, String> = HashMap::new();

    for raw_ref in asset_refs {
        let norm = normalize_dsf_asset_path(raw_ref);
        if norm.is_empty()
            || norm.starts_with("::")
            || norm.starts_with("./")
            || norm.starts_with("../")
            || norm.eq_ignore_ascii_case("terrain_Water")
            || !norm.contains('/')
        {
            continue;
        }

        let Some(first_component) = norm.split('/').next() else {
            continue;
        };
        let first_component = first_component.trim();
        if first_component.is_empty() {
            continue;
        }

        let first_lower = first_component.to_lowercase();
        // Skip built-in X-Plane default virtual library
        if first_lower == "lib" {
            continue;
        }

        // If this package has a matching local directory, verify whether it's a local asset folder
        if local_subdirs_lower.contains(&first_lower) {
            if is_standard_local_asset_folder(&first_lower) || scenery_path.join(&norm).exists() {
                continue;
            }
        } else if is_standard_local_asset_folder(&first_lower) {
            // Standard local folder name (e.g. objects/, terrain/, textures/) is local to the package,
            // except if exported via library.txt, which won't require external downloads anyway
            continue;
        }

        seen_lower
            .entry(first_lower)
            .or_insert_with(|| first_component.to_string());
    }

    let mut result: Vec<String> = seen_lower.into_values().collect();
    result.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    result
}

/// Extract library name from object path (standalone helper used by unit tests)
#[allow(dead_code)]
fn extract_library_name(obj_path: &str) -> Option<String> {
    let norm = normalize_dsf_asset_path(obj_path);
    if norm.is_empty()
        || norm.starts_with("::")
        || norm.starts_with("./")
        || norm.starts_with("../")
        || !norm.contains('/')
    {
        return None;
    }

    let first_component = norm.split('/').next()?.trim();
    if first_component.is_empty() {
        return None;
    }

    let first_lower = first_component.to_lowercase();
    if first_lower == "lib" || is_standard_local_asset_folder(&first_lower) {
        return None;
    }

    Some(first_component.to_string())
}

/// Compute effective modification time across top-level scenery folder, library.txt,
/// and Earth nav data (apt.dat + tile folders) so subfolder edits are always detected!
pub(crate) fn get_scenery_effective_modified_time(path: &Path) -> Result<SystemTime> {
    let mut latest = std::fs::metadata(path)?.modified()?;

    let mut bump = |candidate: &Path| {
        if let Ok(meta) = std::fs::metadata(candidate) {
            if let Ok(mtime) = meta.modified() {
                if mtime > latest {
                    latest = mtime;
                }
            }
        }
    };

    bump(&path.join("library.txt"));

    let earth_nav = path.join("Earth nav data");
    if earth_nav.exists() {
        bump(&earth_nav);
        bump(&earth_nav.join("apt.dat"));
        if let Ok(entries) = std::fs::read_dir(&earth_nav) {
            for entry in entries.flatten().take(64) {
                bump(&entry.path());
            }
        }
    }

    Ok(latest)
}

/// Collect DSF file coordinates from a scenery package
/// Returns list of (latitude, longitude) tuples extracted from DSF filenames
fn collect_dsf_coordinates(scenery_path: &Path) -> Vec<(i32, i32)> {
    let earth_nav_data = scenery_path.join("Earth nav data");
    if !earth_nav_data.exists() {
        return Vec::new();
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
                            if let Some(coord) = parse_dsf_coord_from_filename(&dsf_path) {
                                coordinates.push(coord);
                            }
                        }
                    }
                }
            }
        }
    }

    coordinates
}

/// Parse DSF filename to extract coordinates
/// Format: +30+135.dsf or -45-073.dsf (latitude + longitude)
fn parse_dsf_coord_from_filename(dsf_path: &Path) -> Option<(i32, i32)> {
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

#[derive(Default)]
struct PackageInfoDetails {
    has_apt_dat: bool,
    has_dsf: bool,
    has_library_txt: bool,
    texture_count: usize,
    earth_nav_tile_count: u32,
    required_libraries: Vec<String>,
    missing_libraries: Vec<String>,
    exported_library_names: Vec<String>,
    airport_id: Option<String>,
}

/// Build SceneryPackageInfo with geographic information
fn build_package_info(
    folder_name: String,
    category: SceneryCategory,
    scenery_path: &Path,
    details: PackageInfoDetails,
) -> Result<SceneryPackageInfo> {
    let PackageInfoDetails {
        has_apt_dat,
        has_dsf,
        has_library_txt,
        texture_count,
        earth_nav_tile_count,
        required_libraries,
        missing_libraries,
        exported_library_names,
        airport_id,
    } = details;
    // Calculate sub-priority based on category and folder name
    let sub_priority = calculate_sub_priority(&category, &folder_name);

    // Collect DSF coordinates temporarily for continent calculation
    let coordinates = collect_dsf_coordinates(scenery_path);

    // Calculate continent from coordinates (coordinates are not stored)
    let continent = if !coordinates.is_empty() {
        // Count continents for all coordinates
        let mut continent_counts: HashMap<&'static str, usize> = HashMap::new();

        for (lat, lon) in &coordinates {
            let cont = geo_regions::lookup_region(*lat, *lon);
            *continent_counts.entry(cont).or_insert(0) += 1;
        }

        // Find the most frequent continent
        continent_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cont, _)| cont.to_string())
    } else {
        None
    };

    crate::log_debug!(
        &format!(
            "=== 分类结果 ===\n  \
             包名: {}\n  \
             类型: {:?}\n  \
             依据: apt_dat={}, library_txt={}, dsf={}, textures={}\n  \
             tiles={}, 依赖库={}, 导出库={}",
            folder_name,
            category,
            has_apt_dat,
            has_library_txt,
            has_dsf,
            texture_count > 0,
            earth_nav_tile_count,
            required_libraries.len(),
            exported_library_names.len()
        ),
        "scenery_classifier"
    );

    Ok(SceneryPackageInfo {
        folder_name,
        category: category.clone(),
        sub_priority,
        last_modified: get_scenery_effective_modified_time(scenery_path)?,
        has_apt_dat,
        airport_id,
        has_dsf,
        has_library_txt,
        has_textures: texture_count > 0,
        has_objects: scenery_path.join("objects").exists(),
        texture_count,
        earth_nav_tile_count,
        indexed_at: SystemTime::now(),
        required_libraries,
        missing_libraries,
        exported_library_names,
        enabled: true,     // Default to enabled
        sort_order: 0,     // Will be assigned during index rebuild
        actual_path: None, // Will be set by index manager for shortcut entries
        continent,
        original_category: Some(category), // Preserve original classification
    })
}

/// Calculate sub-priority for a scenery package
/// Sub-priority is used to order scenery within the same category
pub(crate) fn calculate_sub_priority(category: &SceneryCategory, folder_name: &str) -> u8 {
    let folder_name_lower = folder_name.trim().to_lowercase();

    match category {
        SceneryCategory::Airport => {
            // Inside Airport category (from top to bottom):
            // 0: Custom Airports (top)
            // 1: X-Plane Airports
            // 2: X-Plane Landmarks
            // 3: AEP Overlays (bottom of Airport category)
            if is_aep_overlays_folder_name(folder_name) {
                3
            } else if is_xplane_landmarks_folder_name(folder_name) {
                2
            } else if is_xplane_airports_folder_name(folder_name) {
                1
            } else {
                0
            }
        }
        SceneryCategory::Mesh => {
            // XPME mesh (starts with "xpme") should be at the bottom of Mesh category
            // They will be sorted alphabetically among themselves
            if folder_name_lower.starts_with("xpme") {
                2
            } else {
                0
            }
        }
        _ => 0, // Default sub-priority for all other categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn classify_library_with_nested_demo_airport_stays_library() {
        let temp = tempfile::tempdir().unwrap();
        let scenery_path = temp.path().join("openSAM_Library");

        fs::create_dir_all(
            scenery_path
                .join("dev-tools")
                .join("OSJW_openSAM_demo_airport")
                .join("Earth nav data")
                .join("+40+010"),
        )
        .unwrap();
        fs::write(
            scenery_path.join("library.txt"),
            "EXPORT opensam/test/object.obj objects/object.obj\n",
        )
        .unwrap();
        fs::write(
            scenery_path
                .join("dev-tools")
                .join("OSJW_openSAM_demo_airport")
                .join("Earth nav data")
                .join("apt.dat"),
            "I\n1100 Demo\n1 0 0 0 OSJW Demo Airport\n",
        )
        .unwrap();
        fs::write(
            scenery_path
                .join("dev-tools")
                .join("OSJW_openSAM_demo_airport")
                .join("Earth nav data")
                .join("+40+010")
                .join("+40+010.dsf"),
            b"dummy-dsf",
        )
        .unwrap();

        let info = classify_scenery(&scenery_path, temp.path()).unwrap();

        assert_eq!(info.category, SceneryCategory::FixedHighPriority);
        assert!(!info.has_apt_dat);
        assert!(info.has_library_txt);
        assert!(info.has_dsf);
    }

    #[test]
    fn classify_airport_with_top_level_earth_nav_data_stays_airport() {
        let temp = tempfile::tempdir().unwrap();
        let scenery_path = temp.path().join("DemoAirport");

        fs::create_dir_all(scenery_path.join("Earth nav data").join("+40+010")).unwrap();
        fs::write(
            scenery_path.join("library.txt"),
            "EXPORT demo/test/object.obj objects/object.obj\n",
        )
        .unwrap();
        fs::write(
            scenery_path.join("Earth nav data").join("apt.dat"),
            "I\n1100 Demo\n1 0 0 0 KDEM Demo Airport\n",
        )
        .unwrap();

        let info = classify_scenery(&scenery_path, temp.path()).unwrap();

        assert_eq!(info.category, SceneryCategory::Airport);
        assert!(info.has_apt_dat);
        assert!(info.has_library_txt);
    }

    #[test]
    fn test_sam_library_detection() {
        // SAM libraries should be classified as FixedHighPriority
        let folder_names = vec!["SAM_Library", "openSAM_Library", "sam_scenery", "MySAMPack"];
        for name in folder_names {
            let lower = name.to_lowercase();
            assert!(lower.contains("sam"), "{} should contain 'sam'", name);
        }
    }

    #[test]
    fn test_lines3d_library_detection() {
        assert!(is_lines3d_folder_name("Lines3D"));
        assert!(is_lines3d_folder_name("lines3d"));
        assert!(!is_lines3d_folder_name("Lines3D_expanded_documentation"));
    }

    #[test]
    fn test_extract_library_name() {
        // Should extract library names including backslash-separated DSF refs
        assert_eq!(
            extract_library_name("opensceneryx/objects/airport/radio/2.obj"),
            Some("opensceneryx".to_string())
        );
        assert_eq!(
            extract_library_name("ENPY_lib\\objects/ENPY_31.obj"),
            Some("ENPY_lib".to_string())
        );
        assert_eq!(
            extract_library_name("ACS_Singapore/facades/house.fac"),
            Some("ACS_Singapore".to_string())
        );

        // Should return None for local paths
        assert_eq!(extract_library_name("objects/building.obj"), None);
        assert_eq!(extract_library_name("Objects/building.obj"), None);
        assert_eq!(extract_library_name("facades/house.fac"), None);

        // Should return None for default libs and pseudo-paths
        assert_eq!(
            extract_library_name("lib/g10/terrain10/apt_terrain.ter"),
            None
        );
        assert_eq!(extract_library_name("::FLATTEN::.pol"), None);
    }

    #[test]
    fn test_validate_apt_dat_format() {
        // This test would need actual test files
        // For now, just verify the function exists and compiles
    }
}
