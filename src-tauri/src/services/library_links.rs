use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime};

use crate::logger;

/// Remote JSON schema for library download links
#[derive(Debug, Deserialize)]
struct LibraryLinksData {
    // Retained for schema version compatibility checks (future use)
    #[allow(dead_code)]
    version: u32,
    // Retained for cache freshness validation (future use)
    #[allow(dead_code)]
    updated: String,
    libraries: HashMap<String, String>,
}

/// Cached library links with expiration
struct CachedLinks {
    links: HashMap<String, String>,
    fetched_at: SystemTime,
}

/// Global in-memory cache for library download links
static CACHE: LazyLock<Mutex<Option<CachedLinks>>> = LazyLock::new(|| Mutex::new(None));

/// Cache TTL: 24 hours (matching updater pattern)
const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

fn normalize_library_key(raw: &str) -> String {
    let trimmed = raw
        .trim()
        .trim_matches(|c| c == '/' || c == '\\' || c == '\0');
    let first_component = trimmed.split(&['/', '\\'][..]).next().unwrap_or(trimmed);
    first_component.trim().to_lowercase()
}

fn find_library_url(links_db: &HashMap<String, String>, name: &str) -> Option<String> {
    let normalized = normalize_library_key(name);
    if normalized.is_empty() {
        return None;
    }

    links_db.get(&normalized).cloned().or_else(|| {
        let dashed = normalized.replace('_', "-");
        if dashed != normalized {
            return links_db.get(&dashed).cloned();
        }

        let underscored = normalized.replace('-', "_");
        if underscored != normalized {
            return links_db.get(&underscored).cloned();
        }

        None
    })
}

/// Returns the hardcoded fallback library links database.
/// Used when the remote fetch fails (network unavailable, timeout, etc.).
fn hardcoded_links() -> HashMap<String, String> {
    let embedded_json = include_str!("../../../data/library_links.json");

    match serde_json::from_str::<LibraryLinksData>(embedded_json) {
        Ok(data) => data
            .libraries
            .into_iter()
            .map(|(k, v)| (normalize_library_key(&k), v))
            .filter(|(k, _)| !k.is_empty())
            .collect(),
        Err(e) => {
            logger::log_info(
                &format!(
                    "Failed to parse embedded data/library_links.json: {}, using empty fallback",
                    e
                ),
                Some("library_links"),
            );
            HashMap::new()
        }
    }
}

/// Fetch library links from the remote JSON, with in-memory caching.
/// Returns an error on fetch/parse failure.
async fn get_remote_links(force_refresh: bool) -> Result<HashMap<String, String>, String> {
    // Check cache first
    if !force_refresh {
        let cache = CACHE
            .lock()
            .expect("library links cache lock poisoned during read");
        if let Some(ref cached) = *cache {
            if cached.fetched_at.elapsed().unwrap_or(CACHE_TTL) < CACHE_TTL {
                return Ok(cached.links.clone());
            }
        }
    }

    // Try remote fetch
    match fetch_remote_links().await {
        Ok(links) => {
            logger::log_info(
                &format!("Fetched {} library download links from remote", links.len()),
                Some("library_links"),
            );
            let mut cache = CACHE
                .lock()
                .expect("library links cache lock poisoned during write");
            *cache = Some(CachedLinks {
                links: links.clone(),
                fetched_at: SystemTime::now(),
            });
            Ok(links)
        }
        Err(e) => Err(e),
    }
}

/// Fetch library links JSON from the remote proxy service.
async fn fetch_remote_links() -> Result<HashMap<String, String>, String> {
    let client = reqwest::Client::builder()
        .user_agent("XFast Manager")
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let remote_url = crate::vercel_api::endpoint_with_override(
        "library-links-data",
        "XFAST_LIBRARY_LINKS_API_URL",
    );
    let response = client
        .get(&remote_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP status: {}", response.status()));
    }

    let data: LibraryLinksData = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Normalize all keys to lowercase for case-insensitive lookup
    let links: HashMap<String, String> = data
        .libraries
        .into_iter()
        .map(|(k, v)| (normalize_library_key(&k), v))
        .filter(|(k, _)| !k.is_empty())
        .collect();

    Ok(links)
}

/// Look up download links for a list of missing library names.
/// Returns a map of library_name -> Option<download_url>.
/// Lookup is case-insensitive.
pub async fn lookup_library_links_local(
    library_names: Vec<String>,
) -> HashMap<String, Option<String>> {
    let links_db = hardcoded_links();

    library_names
        .into_iter()
        .map(|name| {
            let url = find_library_url(&links_db, &name);
            (name, url)
        })
        .collect()
}

/// Look up download links using remote JSON source.
/// Uses in-memory cache and returns error when remote is unavailable.
pub async fn lookup_library_links_remote(
    library_names: Vec<String>,
    force_refresh: bool,
) -> Result<HashMap<String, Option<String>>, String> {
    let links_db = get_remote_links(force_refresh).await?;

    Ok(library_names
        .into_iter()
        .map(|name| {
            let url = find_library_url(&links_db, &name);
            (name, url)
        })
        .collect())
}

fn sanitize_archive_rel_path(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut out = std::path::PathBuf::new();
    for comp in path.components() {
        if let std::path::Component::Normal(os_str) = comp {
            let s = os_str.to_string_lossy();
            if !s.is_empty() && s != "." && s != ".." {
                out.push(os_str);
            }
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

fn copy_dir_recursive_sync(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir {:?}: {}", dst, e))?;
    for entry in walkdir::WalkDir::new(src).follow_links(false).into_iter().flatten() {
        let rel = entry
            .path()
            .strip_prefix(src)
            .map_err(|e| format!("Strip prefix error: {}", e))?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| format!("Failed to create directory {:?}: {}", target, e))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent {:?}: {}", parent, e))?;
            }
            std::fs::copy(entry.path(), &target)
                .map_err(|e| format!("Failed to copy file to {:?}: {}", target, e))?;
        }
    }
    Ok(())
}

fn extract_archive_to_temp(
    archive_path: &std::path::Path,
    dest_dir: &std::path::Path,
) -> Result<(), String> {
    let format = crate::archive_input::detect_archive_format(archive_path)
        .ok_or_else(|| "File bukan arsip .zip atau .7z yang valid (NEEDS_BROWSER)".to_string())?;

    let prepared = crate::archive_input::prepare_archive_for_read(archive_path, format)
        .map_err(|e| format!("Failed to prepare archive: {}", e))?;
    let read_path = prepared.read_path();

    match format {
        crate::archive_input::ArchiveFormat::Zip => {
            let file = std::fs::File::open(read_path)
                .map_err(|e| format!("Failed to open ZIP: {}", e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Invalid ZIP archive: {}", e))?;
            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| format!("ZIP entry error: {}", e))?;
                let Some(enclosed) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
                    continue;
                };
                let Some(rel) = sanitize_archive_rel_path(&enclosed) else {
                    continue;
                };
                let out_path = dest_dir.join(rel);
                if entry.is_dir() {
                    let _ = std::fs::create_dir_all(&out_path);
                } else {
                    if let Some(parent) = out_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let mut out_file = std::fs::File::create(&out_path)
                        .map_err(|e| format!("Failed to create extracted file: {}", e))?;
                    std::io::copy(&mut entry, &mut out_file)
                        .map_err(|e| format!("Failed to write extracted file: {}", e))?;
                }
            }
            Ok(())
        }
        crate::archive_input::ArchiveFormat::SevenZ => {
            let mut reader = sevenz_rust2::ArchiveReader::open(
                read_path,
                sevenz_rust2::Password::empty(),
            )
            .map_err(|e| format!("Failed to open 7z archive: {}", e))?;
            reader
                .for_each_entries(|entry, entry_reader| {
                    let entry_name = entry.name().replace('\\', "/");
                    let Some(rel) = sanitize_archive_rel_path(std::path::Path::new(&entry_name))
                    else {
                        return Ok(true);
                    };
                    let out_path = dest_dir.join(rel);
                    if entry.is_directory() {
                        std::fs::create_dir_all(&out_path)?;
                    } else {
                        if let Some(parent) = out_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        let mut out_file = std::fs::File::create(&out_path)?;
                        std::io::copy(entry_reader, &mut out_file)?;
                    }
                    Ok(true)
                })
                .map_err(|e| format!("Failed to extract 7z archive: {}", e))?;
            Ok(())
        }
        crate::archive_input::ArchiveFormat::Rar => {
            Err("Format .rar silakan di-install melalui menu Install utama atau ekstrak ke .zip".to_string())
        }
    }
}

fn install_extracted_libraries(
    extracted_root: &std::path::Path,
    custom_scenery_path: &std::path::Path,
    fallback_folder_name: &str,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(custom_scenery_path)
        .map_err(|e| format!("Failed to create Custom Scenery directory: {}", e))?;

    let mut lib_dirs: Vec<std::path::PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(extracted_root)
        .max_depth(5)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.eq_ignore_ascii_case("library.txt"))
        {
            if let Some(parent) = entry.path().parent() {
                let parent_buf = parent.to_path_buf();
                if !lib_dirs.contains(&parent_buf) {
                    lib_dirs.push(parent_buf);
                }
            }
        }
    }

    if lib_dirs.is_empty() {
        return Err("Tidak ditemukan file library.txt di dalam arsip ini.".to_string());
    }

    let mut installed_names = Vec::new();
    for lib_dir in lib_dirs {
        let folder_name = if lib_dir == extracted_root {
            fallback_folder_name
                .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .to_string()
        } else {
            lib_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(fallback_folder_name)
                .to_string()
        };

        let safe_name = if folder_name.is_empty() {
            "Installed_Library".to_string()
        } else {
            folder_name
        };

        let target_dir = custom_scenery_path.join(&safe_name);
        copy_dir_recursive_sync(&lib_dir, &target_dir)?;
        if !installed_names.contains(&safe_name) {
            installed_names.push(safe_name);
        }
    }

    Ok(installed_names)
}

async fn resolve_direct_download_url(
    client: &reqwest::Client,
    raw_url: &str,
) -> Result<String, String> {
    let trimmed = raw_url.trim();

    // Handle GitHub /releases/latest
    if trimmed.contains("github.com/") && trimmed.contains("/releases") {
        let parts: Vec<&str> = trimmed
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .collect();
        if parts.len() >= 3 && parts[0].eq_ignore_ascii_case("github.com") {
            let owner = parts[1];
            let repo = parts[2].trim_end_matches(".git");
            let api_url = format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                owner, repo
            );
            if let Ok(resp) = client
                .get(&api_url)
                .header("Accept", "application/vnd.github+json")
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(assets) = json.get("assets").and_then(|v| v.as_array()) {
                            for asset in assets {
                                if let Some(dl) = asset
                                    .get("browser_download_url")
                                    .and_then(|v| v.as_str())
                                {
                                    let lower = dl.to_lowercase();
                                    if lower.ends_with(".zip") || lower.ends_with(".7z") {
                                        return Ok(dl.to_string());
                                    }
                                }
                            }
                        }
                        if let Some(zipball) = json.get("zipball_url").and_then(|v| v.as_str()) {
                            return Ok(zipball.to_string());
                        }
                    }
                }
            }
        }
    }

    // Handle GitHub repo root or .git URL
    if trimmed.starts_with("https://github.com/") && !trimmed.contains("/archive/") && !trimmed.contains("/releases/download/") {
        let clean = trimmed.trim_end_matches('/').trim_end_matches(".git");
        let segments: Vec<&str> = clean
            .trim_start_matches("https://github.com/")
            .split('/')
            .collect();
        if segments.len() == 2 {
            return Ok(format!(
                "https://github.com/{}/{}/archive/refs/heads/master.zip",
                segments[0], segments[1]
            ));
        }
    }

    // Handle Google Drive /file/d/<id>/view
    if trimmed.contains("drive.google.com/file/d/") {
        if let Some(after) = trimmed.split("drive.google.com/file/d/").nth(1) {
            if let Some(file_id) = after.split(&['/', '?', '&'][..]).next() {
                if !file_id.is_empty() {
                    return Ok(format!(
                        "https://drive.google.com/uc?export=download&id={}&confirm=t",
                        file_id
                    ));
                }
            }
        }
    }

    Ok(trimmed.to_string())
}

/// Download a missing library from its URL (if direct archive) and install into Custom Scenery.
/// Returns Err("NEEDS_BROWSER") when the link points to an HTML page (e.g. X-Plane.org forum)
/// that requires browser login/session.
pub async fn download_and_install_library(
    xplane_path: &std::path::Path,
    library_name: &str,
    url: &str,
) -> Result<Vec<String>, String> {
    let lower_url = url.trim().to_lowercase();
    if lower_url.contains("forums.x-plane.org")
        || lower_url.contains("mega.nz")
        || lower_url.contains("mediafire.com")
        || lower_url.ends_with(".html")
        || lower_url.ends_with(".htm")
    {
        return Err("NEEDS_BROWSER".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("DHUB-Manager/1.2.5")
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resolved_url = resolve_direct_download_url(&client, url).await?;
    logger::log_info(
        &format!(
            "Auto-installing library '{}' from {}",
            library_name, resolved_url
        ),
        Some("library_links"),
    );

    let mut response = client
        .get(&resolved_url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    // Fallback to main.zip if master.zip returned 404 on GitHub
    if response.status() == reqwest::StatusCode::NOT_FOUND
        && resolved_url.ends_with("/archive/refs/heads/master.zip")
    {
        let main_url = resolved_url.replace(
            "/archive/refs/heads/master.zip",
            "/archive/refs/heads/main.zip",
        );
        response = client
            .get(&main_url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;
    }

    if !response.status().is_success() {
        return Err("NEEDS_BROWSER".to_string());
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    if content_type.contains("text/html") {
        return Err("NEEDS_BROWSER".to_string());
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    if bytes.len() < 64 || bytes.starts_with(b"<!DOCTYPE") || bytes.starts_with(b"<html") || bytes.starts_with(b"<HTML") {
        return Err("NEEDS_BROWSER".to_string());
    }

    let xplane_root = xplane_path.to_path_buf();
    let lib_name_owned = library_name.to_string();
    let ext = if resolved_url.to_lowercase().ends_with(".7z") {
        "7z"
    } else {
        "zip"
    };

    tokio::task::spawn_blocking(move || {
        let temp_dir =
            tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let archive_file = temp_dir.path().join(format!("download.{}", ext));
        std::fs::write(&archive_file, &bytes)
            .map_err(|e| format!("Failed to write temp archive: {}", e))?;

        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir_all(&extract_dir)
            .map_err(|e| format!("Failed to create extract dir: {}", e))?;

        extract_archive_to_temp(&archive_file, &extract_dir)?;

        let custom_scenery = xplane_root.join("Custom Scenery");
        install_extracted_libraries(&extract_dir, &custom_scenery, &lib_name_owned)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Install one or more X-Plane libraries directly from a local .zip or .7z archive file.
pub fn install_library_from_archive_path(
    xplane_path: &std::path::Path,
    archive_path: &std::path::Path,
) -> Result<Vec<String>, String> {
    if !archive_path.exists() {
        return Err(format!("File tidak ditemukan: {}", archive_path.display()));
    }

    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let extract_dir = temp_dir.path().join("extracted");
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract dir: {}", e))?;

    extract_archive_to_temp(archive_path, &extract_dir)?;

    let fallback_name = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Installed_Library");
    let custom_scenery = xplane_path.join("Custom Scenery");
    install_extracted_libraries(&extract_dir, &custom_scenery, fallback_name)
}
