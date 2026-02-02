use anyhow::Result;
use image::DynamicImage;
use ratatui_image::protocol::StatefulProtocol;

#[derive(Clone)]
pub struct ArtworkConverter;

impl ArtworkConverter {
    pub fn with_mode(_mode: &str) -> Result<Self> {
        Ok(Self)
    }

    pub fn create_protocol(&self, _img: DynamicImage) -> Option<StatefulProtocol> {
        // Stub implementation: return None to avoid panic
        None
    }
}
