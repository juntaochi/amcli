pub mod converter;
use anyhow::Result;
use image::DynamicImage;
use ratatui::style::Color;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ArtworkManager;

impl ArtworkManager {
    pub fn new(_path: PathBuf) -> Self {
        Self
    }

    pub async fn get_artwork_themed_v2(
        &self,
        _url: &str,
        _dim: Color,
        _primary: Color,
        _theme: &str,
        _mosaic: bool,
        _retro: bool,
    ) -> Result<DynamicImage> {
        Ok(DynamicImage::new_rgb8(1, 1))
    }
}
