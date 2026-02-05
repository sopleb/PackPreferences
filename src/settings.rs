use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

use crate::discovery::CharacterFile;

/// Result of a sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub target_file: PathBuf,
    pub success: bool,
    pub message: String,
}

/// Creates a backup of the settings directory.
pub fn create_backup(settings_dir: &Path) -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let parent = settings_dir
        .parent()
        .context("Settings directory has no parent")?;

    let dir_name = settings_dir
        .file_name()
        .context("Settings directory has no name")?
        .to_string_lossy();

    let backup_name = format!("{}_backup_{}", dir_name, timestamp);
    let backup_path = parent.join(backup_name);

    copy_dir_recursive(settings_dir, &backup_path)?;

    Ok(backup_path)
}

/// Copies a directory recursively.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Lists available backups for a settings directory.
pub fn list_backups(settings_dir: &Path) -> Result<Vec<PathBuf>> {
    let parent = settings_dir
        .parent()
        .context("Settings directory has no parent")?;

    let dir_name = settings_dir
        .file_name()
        .context("Settings directory has no name")?
        .to_string_lossy();

    let backup_prefix = format!("{}_backup_", dir_name);

    let mut backups = Vec::new();

    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with(&backup_prefix) && entry.path().is_dir() {
            backups.push(entry.path());
        }
    }

    // Sort by name (which includes timestamp) in reverse order (newest first)
    backups.sort();
    backups.reverse();

    Ok(backups)
}

/// Restores a backup to the settings directory.
pub fn restore_backup(backup_path: &Path, settings_dir: &Path) -> Result<()> {
    // First, create a backup of current state
    let _current_backup = create_backup(settings_dir)?;

    // Remove current settings directory contents
    for entry in fs::read_dir(settings_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    // Copy backup contents to settings directory
    for entry in fs::read_dir(backup_path)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = settings_dir.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Syncs settings from a source character to target characters.
/// If dry_run is true, returns what would be done without modifying files.
pub fn sync_settings(
    source: &CharacterFile,
    targets: &[&CharacterFile],
    dry_run: bool,
) -> Result<Vec<SyncResult>> {
    let mut results = Vec::new();

    for target in targets {
        // Only sync matching file types (char to char, user to user)
        if source.file_type != target.file_type {
            continue;
        }

        let result = if dry_run {
            SyncResult {
                target_file: target.path.clone(),
                success: true,
                message: "Would copy".to_string(),
            }
        } else {
            match copy_file_atomic(&source.path, &target.path) {
                Ok(()) => SyncResult {
                    target_file: target.path.clone(),
                    success: true,
                    message: "Copied successfully".to_string(),
                },
                Err(e) => SyncResult {
                    target_file: target.path.clone(),
                    success: false,
                    message: format!("Failed: {}", e),
                },
            }
        };

        results.push(result);
    }

    Ok(results)
}

/// Copies a file atomically using a temporary file and rename.
fn copy_file_atomic(src: &Path, dst: &Path) -> Result<()> {
    let tmp_path = dst.with_extension("tmp");

    // Copy to temporary file
    fs::copy(src, &tmp_path)
        .with_context(|| format!("Failed to copy to temp file: {:?}", tmp_path))?;

    // Rename temporary file to destination
    fs::rename(&tmp_path, dst)
        .with_context(|| format!("Failed to rename temp file to: {:?}", dst))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_copy_file_atomic() -> Result<()> {
        let dir = tempdir()?;
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");

        let mut file = File::create(&src)?;
        file.write_all(b"test content")?;

        copy_file_atomic(&src, &dst)?;

        let content = fs::read_to_string(&dst)?;
        assert_eq!(content, "test content");

        // Temp file should not exist
        assert!(!dir.path().join("dest.tmp").exists());

        Ok(())
    }
}
