use anyhow::Result;
use image::DynamicImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;

pub struct ArtworkConverter {
    picker: Picker,
}

impl ArtworkConverter {
    pub fn with_mode(mode: &str) -> Result<Self> {
        let is_zellij = std::env::var("ZELLIJ").is_ok();

        let picker = match mode.to_lowercase().as_str() {
            "halfblocks" => Picker::halfblocks(),
            "sixel" => {
                // Try and query for sixel, fallback to halfblocks but try to be high-res
                Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks())
            }
            _ => {
                if is_zellij && (mode == "auto" || mode.is_empty()) {
                    Picker::halfblocks()
                } else {
                    // Modern terminals: query for best protocol and font-size
                    Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks())
                }
            }
        };
        Ok(Self { picker })
    }

    pub fn create_protocol(&mut self, img: DynamicImage) -> StatefulProtocol {
        self.picker.new_resize_protocol(img)
    }
}
