use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "jp")]
    Japanese,
}

impl Language {
    #[allow(dead_code)]
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
pub struct GeneralConfig {
    pub language: Language,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtworkConfig {
    pub enabled: bool,
    pub cache_size: usize,
    pub mode: String,
    pub album: bool,
    pub mosaic: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub color_theme: String,
    pub show_help_on_start: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub artwork: ArtworkConfig,
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                language: Language::English,
            },
            artwork: ArtworkConfig {
                enabled: true,
                cache_size: 100,
                mode: "auto".to_string(),
                album: true,
                mosaic: true,
            },
            ui: UiConfig {
                color_theme: "default".to_string(),
                show_help_on_start: true,
            },
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?
            .join("amcli");
        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            let config = Config::default();
            config.save().await?;
            return Ok(config);
        }

        let content = fs::read_to_string(config_path)
            .await
            .context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    pub async fn save(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?
            .join("amcli");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .await
                .context("Failed to create config directory")?;
        }

        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(config_path, content)
            .await
            .context("Failed to write config file")?;

        Ok(())
    }
}
