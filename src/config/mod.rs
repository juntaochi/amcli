use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub artwork: ArtworkConfig,
    pub ui: UIConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtworkConfig {
    pub enabled: bool,
    pub cache_size: usize,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UIConfig {
    pub color_theme: String,
    pub show_help_on_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            artwork: ArtworkConfig {
                enabled: true,
                cache_size: 100,
                mode: "auto".into(),
            },
            ui: UIConfig {
                color_theme: "default".into(),
                show_help_on_start: true,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("amcli");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            let content = toml::to_string_pretty(&config)?;
            fs::write(config_path, content)?;
            Ok(config)
        }
    }
}
