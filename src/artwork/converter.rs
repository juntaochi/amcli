use anyhow::Result;
use image::DynamicImage;
use ratatui_image::protocol::StatefulProtocol;

#[derive(Clone)]
pub struct ArtworkConverter;

impl ArtworkConverter {
    pub fn with_mode(_mode: &str) -> Result<Self> {
        Ok(Self)
    }

    pub fn create_protocol(&self, _img: DynamicImage) -> StatefulProtocol {
        // We panic here because we can't easily construct StatefulProtocol in a stub without more context/dependencies setup.
        // If tests fail, we will revisit.
        todo!("Stub implementation")
    }
}
