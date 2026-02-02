use super::LyricsProvider;
use std::path::PathBuf;

pub struct LocalProvider;

impl LocalProvider {
    pub fn new(_path: PathBuf) -> Self {
        Self
    }
}

impl LyricsProvider for LocalProvider {}
