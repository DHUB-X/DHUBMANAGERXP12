use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub letter: String,
    pub path: String,
    pub label: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub free_formatted: String,
    pub total_formatted: String,
    pub is_system_or_game_drive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneryStorageDetail {
    pub folder_name: String,
    pub is_linked: bool,
    pub link_target: Option<String>,
    pub target_drive: Option<String>,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub current_drive: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveSceneryResult {
    pub success: bool,
    pub folder_name: String,
    pub target_path: String,
    pub bytes_moved: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSceneryResult {
    pub success: bool,
    pub folder_name: String,
    pub restored_path: String,
    pub bytes_moved: u64,
    pub message: String,
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn clean_path_string(path: &Path) -> String {
    let s = path.to_string_lossy().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        s
    }
}

#[cfg(windows)]
fn get_volume_label(root_path: &str) -> String {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::fileapi::GetVolumeInformationW;

    let root_wide: Vec<u16> = OsStr::new(root_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut volume_name = [0u16; 261];
    let res = unsafe {
        GetVolumeInformationW(
            root_wide.as_ptr(),
            volume_name.as_mut_ptr(),
            volume_name.len() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        )
    };
    if res != 0 {
        let len = volume_name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(volume_name.len());
        let name = String::from_utf16_lossy(&volume_name[..len])
            .trim()
            .to_string();
        if !name.is_empty() {
            return name;
        }
    }
    "Local Disk".to_string()
}

#[cfg(not(windows))]
fn get_volume_label(_root_path: &str) -> String {
    "Local Disk".to_string()
}

pub fn list_available_drives(xplane_path: &str) -> Vec<DriveInfo> {
    let mut drives = Vec::new();
    let xp_drive = Path::new(xplane_path)
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_uppercase())
        .unwrap_or_default();

    for letter in b'A'..=b'Z' {
        let drive_root = format!("{}:\\", letter as char);
        let path = Path::new(&drive_root);
        if path.exists() {
            let free = fs2::free_space(path).unwrap_or(0);
            let total = fs2::total_space(path).unwrap_or(0);
            if total > 0 {
                let letter_str = format!("{}:", letter as char);
                let is_xp_drive = !xp_drive.is_empty() && xp_drive.starts_with(&letter_str);
                let label = get_volume_label(&drive_root);

                drives.push(DriveInfo {
                    letter: letter_str,
                    path: drive_root,
                    label,
                    free_bytes: free,
                    total_bytes: total,
                    free_formatted: format_bytes(free),
                    total_formatted: format_bytes(total),
                    is_system_or_game_drive: is_xp_drive,
                });
            }
        }
    }
    drives
}

pub fn calculate_folder_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

pub fn get_scenery_storage_info(
    xplane_path: &Path,
    folder_name: &str,
) -> Result<SceneryStorageDetail, String> {
    let scenery_folder = xplane_path.join("Custom Scenery").join(folder_name);
    if !scenery_folder.exists() {
        return Err(format!("Scenery folder does not exist: {}", folder_name));
    }

    let is_linked = is_reparse_or_link(&scenery_folder);
    let link_target = if is_linked {
        get_link_target(&scenery_folder)
    } else {
        None
    };

    let target_drive = link_target.as_ref().and_then(|t| {
        Path::new(t)
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
    });

    let current_drive = scenery_folder
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_default();

    let size_bytes = calculate_folder_size(&scenery_folder);

    Ok(SceneryStorageDetail {
        folder_name: folder_name.to_string(),
        is_linked,
        link_target,
        target_drive,
        size_bytes,
        size_formatted: format_bytes(size_bytes),
        current_drive,
    })
}

pub fn is_reparse_or_link(path: &Path) -> bool {
    #[cfg(windows)]
    {
        if junction::exists(path).unwrap_or(false) {
            return true;
        }
    }
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return true;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if (meta.file_attributes() & 0x400) != 0 {
                return true;
            }
        }
    }
    false
}

pub fn get_link_target(path: &Path) -> Option<String> {
    #[cfg(windows)]
    {
        if let Ok(target) = junction::get_target(path) {
            return Some(clean_path_string(&target));
        }
    }
    if let Ok(target) = fs::read_link(path) {
        return Some(clean_path_string(&target));
    }
    None
}

pub fn move_scenery_to_drive(
    xplane_path: &Path,
    folder_name: &str,
    target_destination: &str,
) -> Result<MoveSceneryResult, String> {
    let source_path = xplane_path.join("Custom Scenery").join(folder_name);
    if !source_path.exists() {
        return Err(format!("Scenery folder does not exist: {}", folder_name));
    }

    if is_reparse_or_link(&source_path) {
        return Err(format!(
            "Scenery '{}' is already linked to another location! Restore it first before moving again.",
            folder_name
        ));
    }

    let dest_input = PathBuf::from(target_destination.trim());
    let dest_dir = if dest_input.components().count() <= 1
        || target_destination.ends_with(':')
        || target_destination.ends_with(":\\")
    {
        // User provided a drive root like "D:" or "D:\" -> use standard subfolder
        dest_input.join("X-Plane_Custom_Scenery_Storage")
    } else {
        dest_input
    };

    let target_path = dest_dir.join(folder_name);
    if target_path.exists() {
        return Err(format!(
            "Destination folder already exists: {}",
            target_path.display()
        ));
    }

    let size_bytes = calculate_folder_size(&source_path);

    // Ensure destination parent directory exists
    if let Err(e) = fs::create_dir_all(&dest_dir) {
        return Err(format!("Failed to create destination directory: {}", e));
    }

    // Try fast move (rename) if on same filesystem, otherwise copy then remove
    let rename_res = fs::rename(&source_path, &target_path);
    if let Err(_rename_err) = rename_res {
        // Fall back to recursive copy + remove
        if let Err(copy_err) = copy_dir_recursive(&source_path, &target_path) {
            let _ = fs::remove_dir_all(&target_path);
            return Err(format!("Failed to copy files to target drive: {}", copy_err));
        }
        if let Err(remove_err) = fs::remove_dir_all(&source_path) {
            return Err(format!(
                "Copied files successfully, but failed to clean up original directory: {}",
                remove_err
            ));
        }
    }

    // Now create NTFS junction
    #[cfg(windows)]
    {
        if let Err(junc_err) = junction::create(&target_path, &source_path) {
            return Err(format!(
                "Files moved to {}, but failed to create junction at {}: {}",
                target_path.display(),
                source_path.display(),
                junc_err
            ));
        }
    }

    #[cfg(not(windows))]
    {
        if let Err(sym_err) = std::os::unix::fs::symlink(&target_path, &source_path) {
            return Err(format!(
                "Files moved to {}, but failed to create symlink at {}: {}",
                target_path.display(),
                source_path.display(),
                sym_err
            ));
        }
    }

    Ok(MoveSceneryResult {
        success: true,
        folder_name: folder_name.to_string(),
        target_path: target_path.display().to_string(),
        bytes_moved: size_bytes,
        message: format!(
            "Successfully moved '{}' ({}) to {} and created junction link.",
            folder_name,
            format_bytes(size_bytes),
            target_path.display()
        ),
    })
}

pub fn restore_scenery_to_ssd(
    xplane_path: &Path,
    folder_name: &str,
) -> Result<RestoreSceneryResult, String> {
    let link_path = xplane_path.join("Custom Scenery").join(folder_name);
    if !link_path.exists() {
        return Err(format!("Scenery folder does not exist: {}", folder_name));
    }

    if !is_reparse_or_link(&link_path) {
        return Err(format!(
            "Scenery '{}' is not a junction or symlink. It is already on the primary drive.",
            folder_name
        ));
    }

    let actual_target_str = get_link_target(&link_path).ok_or_else(|| {
        format!(
            "Unable to determine target location for junction: {}",
            folder_name
        )
    })?;

    let actual_target = PathBuf::from(&actual_target_str);
    if !actual_target.exists() {
        return Err(format!(
            "Target directory no longer exists on external drive: {}",
            actual_target_str
        ));
    }

    let size_bytes = calculate_folder_size(&actual_target);

    // Remove the junction/symlink link itself
    #[cfg(windows)]
    {
        if let Err(e) = junction::delete(&link_path) {
            // Also try fs::remove_dir in case junction::delete is not applicable
            if let Err(e2) = fs::remove_dir(&link_path) {
                return Err(format!("Failed to remove junction link: {} / {}", e, e2));
            }
        }
    }
    #[cfg(not(windows))]
    {
        if let Err(e) = fs::remove_file(&link_path) {
            return Err(format!("Failed to remove symlink: {}", e));
        }
    }

    // Now move the actual files back to link_path
    let rename_res = fs::rename(&actual_target, &link_path);
    if let Err(_rename_err) = rename_res {
        // Cross drive copy + remove
        if let Err(copy_err) = copy_dir_recursive(&actual_target, &link_path) {
            // If copy failed, try to restore junction so user doesn't lose anything
            #[cfg(windows)]
            let _ = junction::create(&actual_target, &link_path);
            return Err(format!("Failed to copy files back to primary drive: {}", copy_err));
        }
        let _ = fs::remove_dir_all(&actual_target);
    }

    Ok(RestoreSceneryResult {
        success: true,
        folder_name: folder_name.to_string(),
        restored_path: link_path.display().to_string(),
        bytes_moved: size_bytes,
        message: format!(
            "Successfully restored '{}' ({}) back to primary SSD.",
            folder_name,
            format_bytes(size_bytes)
        ),
    })
}

pub fn unlink_scenery_only(xplane_path: &Path, folder_name: &str) -> Result<String, String> {
    let link_path = xplane_path.join("Custom Scenery").join(folder_name);
    if !link_path.exists() {
        return Err(format!("Scenery folder does not exist: {}", folder_name));
    }

    if !is_reparse_or_link(&link_path) {
        return Err(format!("'{}' is not a junction or symlink.", folder_name));
    }

    #[cfg(windows)]
    {
        if let Err(e) = junction::delete(&link_path) {
            if let Err(e2) = fs::remove_dir(&link_path) {
                return Err(format!("Failed to unlink junction: {} / {}", e, e2));
            }
        }
    }
    #[cfg(not(windows))]
    {
        if let Err(e) = fs::remove_file(&link_path) {
            return Err(format!("Failed to unlink: {}", e));
        }
    }

    Ok(format!("Unlinked '{}' from Custom Scenery.", folder_name))
}

pub fn link_external_folder(
    xplane_path: &Path,
    source_folder_path: &str,
    link_folder_name: Option<String>,
) -> Result<String, String> {
    let external_dir = PathBuf::from(source_folder_path.trim());
    if !external_dir.exists() || !external_dir.is_dir() {
        return Err("Source external folder does not exist or is not a directory.".to_string());
    }

    let folder_name = match link_folder_name {
        Some(name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => external_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| "Could not determine folder name from source path.".to_string())?,
    };

    let destination_junction = xplane_path.join("Custom Scenery").join(&folder_name);
    if destination_junction.exists() {
        return Err(format!(
            "Folder or junction '{}' already exists in Custom Scenery.",
            folder_name
        ));
    }

    #[cfg(windows)]
    {
        junction::create(&external_dir, &destination_junction).map_err(|e| {
            format!(
                "Failed to create junction from '{}' to '{}': {}",
                external_dir.display(),
                destination_junction.display(),
                e
            )
        })?;
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(&external_dir, &destination_junction).map_err(|e| {
            format!(
                "Failed to create symlink from '{}' to '{}': {}",
                external_dir.display(),
                destination_junction.display(),
                e
            )
        })?;
    }

    Ok(format!(
        "Successfully linked '{}' to Custom Scenery as '{}'.",
        external_dir.display(),
        folder_name
    ))
}
