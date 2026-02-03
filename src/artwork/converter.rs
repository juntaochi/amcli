use anyhow::Result;
use image::DynamicImage;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

pub struct ArtworkConverter {
    picker: Picker,
}

impl ArtworkConverter {
    pub fn with_mode(mode: &str) -> Result<Self> {
        // "auto" tries to query the active terminal; fall back to halfblocks for maximum
        // compatibility.
        let picker = match mode {
            // Keep accepting historical values like "kitty"/"iterm2"/"sixel" in config;
            // with our current feature set we rely on auto-detection.
            "auto" | "kitty" | "iterm2" | "sixel" => {
                Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks())
            }
            "halfblocks" => Picker::halfblocks(),
            _ => Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks()),
        };

        Ok(Self { picker })
    }

    pub fn create_protocol(&self, img: DynamicImage) -> StatefulProtocol {
        self.picker.new_resize_protocol(img)
    }
}
