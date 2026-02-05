use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DetectedPrefix {
    pub path: PathBuf,
}

/// Scans /proc for running EVE Online processes and extracts Wine prefixes.
pub fn detect_eve_prefixes() -> Result<Vec<DetectedPrefix>> {
    let mut prefixes = Vec::new();

    let proc_dir = fs::read_dir("/proc")?;

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Skip non-numeric entries
        let _pid: u32 = match name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // Try to read cmdline
        let cmdline_path = entry.path().join("cmdline");
        if let Ok(cmdline) = fs::read(&cmdline_path) {
            // cmdline is null-byte delimited
            let cmdline_str = String::from_utf8_lossy(&cmdline);

            // Look for eve-online.exe (case-insensitive)
            let lower = cmdline_str.to_lowercase();
            if lower.contains("eve-online.exe") || lower.contains("exefile.exe") {
                // Extract prefix: everything up to and including "drive_c"
                if let Some(prefix) = extract_prefix(&cmdline_str) {
                    prefixes.push(DetectedPrefix { path: prefix });
                }
            }
        }
    }

    // Deduplicate by path
    prefixes.sort_by(|a, b| a.path.cmp(&b.path));
    prefixes.dedup_by(|a, b| a.path == b.path);

    Ok(prefixes)
}

/// Extracts the Wine prefix from a command line containing drive_c.
fn extract_prefix(cmdline: &str) -> Option<PathBuf> {
    // Split on null bytes to get arguments
    for arg in cmdline.split('\0') {
        let lower = arg.to_lowercase();
        if let Some(pos) = lower.find("drive_c") {
            // Include "drive_c" in the prefix
            let prefix = &arg[..pos + 7];
            return Some(PathBuf::from(prefix));
        }
    }
    None
}

/// Finds EVE settings directories within a Wine prefix.
pub fn find_settings_dirs(prefix: &Path) -> Result<Vec<PathBuf>> {
    let eve_base = prefix
        .join("users")
        .join("steamuser")
        .join("AppData")
        .join("Local")
        .join("CCP")
        .join("EVE");

    let mut settings_dirs = Vec::new();

    if eve_base.exists() {
        for entry in fs::read_dir(&eve_base)? {
            let entry = entry?;
            let settings_default = entry.path().join("settings_Default");
            if settings_default.is_dir() {
                settings_dirs.push(settings_default);
            }
        }
    }

    Ok(settings_dirs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prefix() {
        let cmdline = "/home/user/Games/Eve/drive_c/eve/exefile.exe\0--arg";
        let prefix = extract_prefix(cmdline);
        assert_eq!(prefix, Some(PathBuf::from("/home/user/Games/Eve/drive_c")));
    }

    #[test]
    fn test_extract_prefix_mixed_case() {
        let cmdline = "/home/user/Games/Eve/Drive_C/eve/exefile.exe";
        let prefix = extract_prefix(cmdline);
        // Should still work with mixed case
        assert!(prefix.is_some());
    }

    #[test]
    fn test_extract_prefix_no_match() {
        let cmdline = "/usr/bin/firefox";
        let prefix = extract_prefix(cmdline);
        assert_eq!(prefix, None);
    }
}
