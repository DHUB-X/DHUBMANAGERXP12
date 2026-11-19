//! Windows Defender exclusion management module
//!
//! Provides functionality to check, add, and remove Windows Defender exclusions
//! for the X-Plane directory. Elevation is requested on-demand via UAC when
//! changing exclusions, allowing the main application to run without administrator rights.

use anyhow::Result;

#[cfg(target_os = "windows")]
fn run_elevated_powershell(inner_script: &str) -> Result<bool> {
    use base64::{engine::general_purpose, Engine as _};
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // Encode inner script as UTF-16LE Base64 for PowerShell -EncodedCommand
    let utf16_bytes: Vec<u8> = inner_script
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect();
    let encoded_inner = general_purpose::STANDARD.encode(&utf16_bytes);

    // Outer script launches powershell elevated with RunAs, waits for it, and returns the exit code.
    // If the user cancels the UAC prompt, Start-Process throws and we exit with 1223 (ERROR_CANCELLED).
    let outer_script = format!(
        r#"
try {{
    $proc = Start-Process powershell.exe -Verb RunAs -ArgumentList '-NoProfile -NonInteractive -WindowStyle Hidden -EncodedCommand {encoded}' -Wait -PassThru -ErrorAction Stop
    if ($proc.ExitCode -ne $null) {{
        exit $proc.ExitCode
    }} else {{
        exit 0
    }}
}} catch {{
    exit 1223
}}
"#,
        encoded = encoded_inner
    );

    let output = std::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args(&[
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &outer_script,
        ])
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);

    if exit_code == 0 {
        Ok(true)
    } else if exit_code == 1223 {
        Err(anyhow::anyhow!("Administrator permission was cancelled by user"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "Failed to update Windows Defender exclusion (exit code {}): {}",
            exit_code,
            stderr.trim()
        ))
    }
}

/// Check if the specified directory is already in Windows Defender exclusions
#[cfg(target_os = "windows")]
pub fn check_defender_exclusion(xplane_path: &str) -> Result<bool> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let clean_path = xplane_path.trim().trim_end_matches(['\\', '/']);
    if clean_path.is_empty() {
        return Ok(false);
    }

    let script = format!(
        r#"
$p = '{}'
try {{
    $pref = Get-MpPreference -ErrorAction Stop
    $paths = $pref.ExclusionPath
    if ($paths) {{
        $target = $p.TrimEnd('\')
        foreach ($item in $paths) {{
            if ($item -and $item.TrimEnd('\') -ieq $target) {{
                exit 0
            }}
        }}
    }}
    exit 1
}} catch {{
    exit 2
}}
"#,
        clean_path.replace('\'', "''")
    );

    let output = std::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args(&[
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code == 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Add the specified directory to Windows Defender exclusions (triggers UAC on demand)
#[cfg(target_os = "windows")]
pub fn add_defender_exclusion(xplane_path: &str) -> Result<bool> {
    let clean_path = xplane_path.trim().trim_end_matches(['\\', '/']);
    if clean_path.is_empty() {
        return Err(anyhow::anyhow!("X-Plane path cannot be empty"));
    }

    let inner_script = format!(
        r#"
$p = '{}'
try {{
    Add-MpPreference -ExclusionPath $p -ErrorAction Stop
    exit 0
}} catch {{
    exit 1
}}
"#,
        clean_path.replace('\'', "''")
    );

    run_elevated_powershell(&inner_script)
}

/// Remove the specified directory from Windows Defender exclusions (triggers UAC on demand)
#[cfg(target_os = "windows")]
pub fn remove_defender_exclusion(xplane_path: &str) -> Result<bool> {
    let clean_path = xplane_path.trim().trim_end_matches(['\\', '/']);
    if clean_path.is_empty() {
        return Err(anyhow::anyhow!("X-Plane path cannot be empty"));
    }

    let inner_script = format!(
        r#"
$p = '{}'
try {{
    Remove-MpPreference -ExclusionPath $p -ErrorAction Stop
    exit 0
}} catch {{
    exit 1
}}
"#,
        clean_path.replace('\'', "''")
    );

    run_elevated_powershell(&inner_script)
}

// Fallbacks for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn check_defender_exclusion(_xplane_path: &str) -> Result<bool> {
    Ok(false)
}

#[cfg(not(target_os = "windows"))]
pub fn add_defender_exclusion(_xplane_path: &str) -> Result<bool> {
    Err(anyhow::anyhow!("Windows Defender exclusions are only supported on Windows"))
}

#[cfg(not(target_os = "windows"))]
pub fn remove_defender_exclusion(_xplane_path: &str) -> Result<bool> {
    Err(anyhow::anyhow!("Windows Defender exclusions are only supported on Windows"))
}
