use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub last_prefix_path: Option<String>,

    #[serde(default = "default_window_x")]
    pub window_x: f32,

    #[serde(default = "default_window_y")]
    pub window_y: f32,

    #[serde(default)]
    pub character_name_cache: HashMap<u64, String>,
}

fn default_window_x() -> f32 {
    100.0
}

fn default_window_y() -> f32 {
    100.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_prefix_path: None,
            window_x: default_window_x(),
            window_y: default_window_y(),
            character_name_cache: HashMap::new(),
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("pack-preferences");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(config_dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents =
                fs::read_to_string(&config_path).context("Failed to read config file")?;
            let config: Config =
                toml::from_str(&contents).context("Failed to parse config file")?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&config_path, contents).context("Failed to write config file")?;
        Ok(())
    }

    pub fn cache_character_name(&mut self, character_id: u64, name: String) {
        self.character_name_cache.insert(character_id, name);
    }

    pub fn get_cached_name(&self, character_id: u64) -> Option<&String> {
        self.character_name_cache.get(&character_id)
    }
}
