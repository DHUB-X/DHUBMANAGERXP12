#[cfg(target_os = "windows")]
use anyhow::Result;
#[cfg(target_os = "windows")]
use std::env;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[cfg(target_os = "windows")]
pub fn register_context_menu() -> Result<()> {
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    // Use HKEY_CURRENT_USER instead of HKEY_CLASSES_ROOT to avoid requiring admin privileges
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Clean up legacy keys if they exist
    let _ = hkcu.delete_subkey_all(r"Software\Classes\*\shell\XFast Manager");
    let _ = hkcu.delete_subkey_all(r"Software\Classes\Directory\shell\XFast Manager");

    // Register for files (*)
    let (shell_key, _) = hkcu.create_subkey(r"Software\Classes\*\shell\DHUB Manager")?;
    shell_key.set_value("", &"Install to X-Plane")?;
    shell_key.set_value("Icon", &exe_path_str.to_string())?;

    let (command_key, _) = hkcu.create_subkey(r"Software\Classes\*\shell\DHUB Manager\command")?;
    command_key.set_value("", &format!("\"{}\" \"%1\"", exe_path_str))?;

    // Register for directories
    let (dir_shell_key, _) =
        hkcu.create_subkey(r"Software\Classes\Directory\shell\DHUB Manager")?;
    dir_shell_key.set_value("", &"Install to X-Plane")?;
    dir_shell_key.set_value("Icon", &exe_path_str.to_string())?;

    let (dir_command_key, _) =
        hkcu.create_subkey(r"Software\Classes\Directory\shell\DHUB Manager\command")?;
    dir_command_key.set_value("", &format!("\"{}\" \"%1\"", exe_path_str))?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn unregister_context_menu() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Try to delete for files
    let _ = hkcu.delete_subkey_all(r"Software\Classes\*\shell\DHUB Manager");
    let _ = hkcu.delete_subkey_all(r"Software\Classes\*\shell\XFast Manager");

    // Try to delete for directories
    let _ = hkcu.delete_subkey_all(r"Software\Classes\Directory\shell\DHUB Manager");
    let _ = hkcu.delete_subkey_all(r"Software\Classes\Directory\shell\XFast Manager");

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn is_context_menu_registered() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // Check if the registry key exists
    hkcu.open_subkey(r"Software\Classes\*\shell\DHUB Manager")
        .is_ok()
}

/// Get the exe path currently stored in the registry
#[cfg(target_os = "windows")]
fn get_registered_exe_path() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let command_key = hkcu
        .open_subkey(r"Software\Classes\*\shell\DHUB Manager\command")
        .ok()?;
    let value: String = command_key.get_value("").ok()?;
    // Parse "\"C:\path\to\exe.exe\" \"%1\"" format
    let path = value.trim_start_matches('"');
    let end = path.find("\" \"")?;
    Some(path[..end].to_string())
}

/// Sync registry paths on startup
/// Returns Ok(true) if paths were updated, Ok(false) if no update needed
#[cfg(target_os = "windows")]
pub fn sync_registry_paths() -> Result<bool> {
    if !is_context_menu_registered() {
        return Ok(false);
    }

    let current_path = env::current_exe()?.to_string_lossy().to_string();

    if let Some(registered_path) = get_registered_exe_path() {
        if registered_path != current_path {
            // Path mismatch, re-register to update paths
            register_context_menu()?;
            return Ok(true);
        }
    }

    Ok(false)
}

/// Clean up stale or duplicate uninstall registry entries left by previous installers or updates
#[cfg(target_os = "windows")]
pub fn clean_stale_uninstall_entries(current_version: &str) -> Result<()> {
    let hives = [
        (HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"Software\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    let current_exe = env::current_exe().ok();
    let current_dir = current_exe
        .as_ref()
        .and_then(|p| p.parent())
        .map(|p| p.to_string_lossy().to_lowercase());

    for (predef, path) in hives {
        let base_key = match RegKey::predef(predef).open_subkey_with_flags(path, KEY_READ | KEY_WRITE) {
            Ok(k) => k,
            Err(_) => continue,
        };

        let subkey_names: Vec<String> = base_key.enum_keys().filter_map(|k| k.ok()).collect();
        for subkey_name in subkey_names {
            let subkey = match base_key.open_subkey(&subkey_name) {
                Ok(k) => k,
                Err(_) => continue,
            };

            let display_name: String = subkey.get_value::<String, _>("DisplayName").unwrap_or_default();
            let lower_name = display_name.to_lowercase();
            if !lower_name.contains("dhub manager") {
                continue;
            }

            let display_version: String = subkey.get_value::<String, _>("DisplayVersion").unwrap_or_default();
            let install_loc: String = subkey.get_value::<String, _>("InstallLocation").unwrap_or_default().to_lowercase();
            let uninstall_str: String = subkey.get_value::<String, _>("UninstallString").unwrap_or_default().to_lowercase();

            // Check if this registry entry is tied to our install folder
            let matches_our_folder = if let Some(ref our_dir) = current_dir {
                (!install_loc.is_empty() && install_loc.contains(our_dir))
                    || (!uninstall_str.is_empty() && uninstall_str.contains(our_dir))
            } else {
                false
            };

            // Check if the uninstaller exe referenced actually exists
            let uninstaller_missing = if !uninstall_str.is_empty() {
                let raw_path = uninstall_str.trim().trim_matches('"');
                let exe_candidate = if let Some(first_quote_end) = uninstall_str.find("\" ") {
                    &uninstall_str[1..first_quote_end]
                } else if let Some(space_pos) = raw_path.find(" /") {
                    &raw_path[..space_pos]
                } else {
                    raw_path
                };
                !std::path::Path::new(exe_candidate).exists()
            } else {
                false
            };

            // If it's an older version of the same app, or uninstaller points to deleted files, or version mismatch in same folder:
            let is_older_version = if !display_version.is_empty() && display_version != current_version {
                match (
                    semver::Version::parse(&display_version),
                    semver::Version::parse(current_version),
                ) {
                    (Ok(v_reg), Ok(v_curr)) => v_reg < v_curr,
                    _ => matches_our_folder,
                }
            } else {
                false
            };

            if is_older_version || (matches_our_folder && display_version != current_version) || uninstaller_missing {
                let _ = base_key.delete_subkey_all(&subkey_name);
                crate::logger::log_info(
                    &format!(
                        "Cleaned stale/duplicate Windows uninstall registry entry: {} (v{}, key={})",
                        display_name, display_version, subkey_name
                    ),
                    Some("app"),
                );
            }
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clean_stale_uninstall_entries(_current_version: &str) -> Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
use anyhow::Result;

#[cfg(not(target_os = "windows"))]
pub fn register_context_menu() -> Result<()> {
    Err(anyhow::anyhow!(
        "Context menu registration is only supported on Windows"
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn unregister_context_menu() -> Result<()> {
    Err(anyhow::anyhow!(
        "Context menu unregistration is only supported on Windows"
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn is_context_menu_registered() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn sync_registry_paths() -> Result<bool> {
    Ok(false)
}
