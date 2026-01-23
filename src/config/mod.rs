use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "jp")]
    Japanese,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Japanese => "jp",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::English => Language::Japanese,
            Language::Japanese => Language::English,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub artwork: ArtworkConfig,
    pub ui: UIConfig,
    #[serde(default)]
    pub general: GeneralConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default)]
    pub language: Language,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: Language::English,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtworkConfig {
    pub enabled: bool,
    pub cache_size: usize,
    pub mode: String,
    #[serde(default = "default_mosaic")]
    pub mosaic: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UIConfig {
    pub color_theme: String,
    pub show_help_on_start: bool,
}

fn default_mosaic() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            artwork: ArtworkConfig {
                enabled: true,
                cache_size: 100,
                mode: "auto".into(),
                mosaic: true,
            },
            ui: UIConfig {
                color_theme: "default".into(),
                show_help_on_start: true,
            },
            general: GeneralConfig {
                language: Language::English,
            },
        }
    }
}

impl Config {
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("amcli");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
