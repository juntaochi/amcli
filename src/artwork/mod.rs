use anyhow::Result;
use image::DynamicImage;
use ratatui::style::Color;
use std::path::PathBuf;

pub mod converter;

#[derive(Clone, Debug)]
pub struct ArtworkManager;

impl ArtworkManager {
    pub fn new(_cache_dir: PathBuf) -> Self {
        Self
    }

    pub async fn get_artwork_themed_v2(
        &self,
        _url: &str,
        _primary: Color,
        _dim: Color,
        _theme_name: &str,
        _mosaic: bool,
        _is_retro: bool,
    ) -> Result<DynamicImage> {
        Ok(DynamicImage::new_rgb8(1, 1))
    }
}
