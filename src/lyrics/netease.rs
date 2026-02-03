use crate::lyrics::LyricsProvider;

pub struct NeteaseProvider;

impl NeteaseProvider {
    pub fn new() -> Self {
        Self
    }
}

impl LyricsProvider for NeteaseProvider {}
