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
    /// True if this is a default file (core_char__.dat or core_user__.dat)
    pub is_default: bool,
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
        if let Some((id, is_default)) = parse_char_file(&filename) {
            files.push(CharacterFile {
                path,
                character_id: id,
                file_type: FileType::Character,
                is_default,
            });
        }
        // Parse user files: core_user_*.dat
        else if let Some((id, is_default)) = parse_user_file(&filename) {
            files.push(CharacterFile {
                path,
                character_id: id,
                file_type: FileType::User,
                is_default,
            });
        }
    }

    // Sort by character ID, with defaults at the end
    files.sort_by_key(|f| (f.is_default, f.character_id));

    Ok(files)
}

/// Parses core_char_*.dat filename and returns (character ID, is_default).
/// Returns id=0 for default file core_char__.dat
fn parse_char_file(filename: &str) -> Option<(u64, bool)> {
    if !filename.starts_with("core_char_") || !filename.ends_with(".dat") {
        return None;
    }

    let id_part = filename.strip_prefix("core_char_")?.strip_suffix(".dat")?;

    // Default file has underscore as placeholder (core_char__.dat -> "_")
    if id_part.is_empty() || id_part == "_" {
        return Some((0, true));
    }

    id_part.parse().ok().map(|id| (id, false))
}

/// Parses core_user_*.dat filename and returns (user ID, is_default).
/// Returns id=0 for default file core_user__.dat
fn parse_user_file(filename: &str) -> Option<(u64, bool)> {
    if !filename.starts_with("core_user_") || !filename.ends_with(".dat") {
        return None;
    }

    let id_part = filename.strip_prefix("core_user_")?.strip_suffix(".dat")?;

    // Default file has underscore as placeholder (core_user__.dat -> "_")
    if id_part.is_empty() || id_part == "_" {
        return Some((0, true));
    }

    id_part.parse().ok().map(|id| (id, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_char_file() {
        assert_eq!(
            parse_char_file("core_char_123456789.dat"),
            Some((123456789, false))
        );
        assert_eq!(parse_char_file("core_char__.dat"), Some((0, true)));
        assert_eq!(parse_char_file("core_user_123.dat"), None);
        assert_eq!(parse_char_file("other.dat"), None);
    }

    #[test]
    fn test_parse_user_file() {
        assert_eq!(
            parse_user_file("core_user_987654321.dat"),
            Some((987654321, false))
        );
        assert_eq!(parse_user_file("core_user__.dat"), Some((0, true)));
        assert_eq!(parse_user_file("core_char_123.dat"), None);
    }
}
