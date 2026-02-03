use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
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
    #[serde(default = "default_album")]
    pub album: bool,
    #[serde(default = "default_mosaic")]
    pub mosaic: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UIConfig {
    pub color_theme: String,
    pub show_help_on_start: bool,
}

fn default_album() -> bool {
    true
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
                album: true,
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
    async fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("amcli");

        if !tokio::fs::try_exists(&config_dir).await.unwrap_or(false) {
            tokio::fs::create_dir_all(&config_dir).await?;
        }

        Ok(config_dir.join("config.toml"))
    }

    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path().await?;

        if tokio::fs::try_exists(&config_path).await.unwrap_or(false) {
            let content = tokio::fs::read_to_string(config_path).await?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save().await?;
            Ok(config)
        }
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path().await?;
        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(config_path, content).await?;
        Ok(())
    }
}
