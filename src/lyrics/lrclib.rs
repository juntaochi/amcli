use crate::lyrics::LyricsProvider;

pub struct LrclibProvider;

impl LrclibProvider {
    pub fn new() -> Self {
        Self
    }
}

impl LyricsProvider for LrclibProvider {}
