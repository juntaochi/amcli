use anyhow::Result;
use image::DynamicImage;
use ratatui_image::protocol::StatefulProtocol;

pub struct ArtworkConverter;

impl ArtworkConverter {
    pub fn with_mode(_mode: &str) -> Result<Self> {
        Ok(Self)
    }

    pub fn create_protocol(&self, _img: DynamicImage) -> StatefulProtocol {
        unimplemented!("Stubbed ArtworkConverter::create_protocol")
    }
}
