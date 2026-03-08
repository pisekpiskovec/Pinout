use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: ThemeConfig,
    pub bridge_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                mode: "Dark".to_string(),
            },
            bridge_address: "127.0.0.1:9000".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = Self::get_config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;

        toml::from_str(&contents).map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_config_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to seialize config: {}", e))?;

        fs::write(&path, toml_string).map_err(|e| format!("Failed to write config: {}", e))
    }

    fn get_config_path() -> Result<PathBuf, String> {
        let mut path =
            dirs::config_dir().unwrap_or_else(|| PathBuf::from(std::env::home_dir().unwrap()));
        path.push("Breadboard");
        fs::create_dir_all(&path).ok();
        path.push("pinout.toml");
        Ok(path)
    }
}
