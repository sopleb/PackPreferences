use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Character,
    User,
}

#[derive(Debug, Clone)]
pub struct CharacterFile {
    pub path: PathBuf,
    pub character_id: u64,
    pub file_type: FileType,
}

/// Discovers character and user settings files in a settings directory.
pub fn discover_character_files(settings_dir: &Path) -> Result<Vec<CharacterFile>> {
    let mut files = Vec::new();

    if !settings_dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(settings_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        // Parse character files: core_char_*.dat
        if let Some(id) = parse_char_file(&filename) {
            files.push(CharacterFile {
                path,
                character_id: id,
                file_type: FileType::Character,
            });
        }
        // Parse user files: core_user_*.dat
        else if let Some(id) = parse_user_file(&filename) {
            files.push(CharacterFile {
                path,
                character_id: id,
                file_type: FileType::User,
            });
        }
    }

    // Sort by character ID
    files.sort_by_key(|f| f.character_id);

    Ok(files)
}

/// Parses core_char_*.dat filename and returns character ID.
/// Excludes system file core_char__.dat
fn parse_char_file(filename: &str) -> Option<u64> {
    if !filename.starts_with("core_char_") || !filename.ends_with(".dat") {
        return None;
    }

    let id_part = filename.strip_prefix("core_char_")?.strip_suffix(".dat")?;

    // Exclude system file (empty ID)
    if id_part.is_empty() {
        return None;
    }

    id_part.parse().ok()
}

/// Parses core_user_*.dat filename and returns user ID.
/// Excludes system file core_user__.dat
fn parse_user_file(filename: &str) -> Option<u64> {
    if !filename.starts_with("core_user_") || !filename.ends_with(".dat") {
        return None;
    }

    let id_part = filename.strip_prefix("core_user_")?.strip_suffix(".dat")?;

    // Exclude system file (empty ID)
    if id_part.is_empty() {
        return None;
    }

    id_part.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_char_file() {
        assert_eq!(parse_char_file("core_char_123456789.dat"), Some(123456789));
        assert_eq!(parse_char_file("core_char__.dat"), None);
        assert_eq!(parse_char_file("core_user_123.dat"), None);
        assert_eq!(parse_char_file("other.dat"), None);
    }

    #[test]
    fn test_parse_user_file() {
        assert_eq!(parse_user_file("core_user_987654321.dat"), Some(987654321));
        assert_eq!(parse_user_file("core_user__.dat"), None);
        assert_eq!(parse_user_file("core_char_123.dat"), None);
    }
}
