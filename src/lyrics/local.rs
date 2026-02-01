use std::path::PathBuf;
use crate::lyrics::LyricsProvider;

pub struct LocalProvider;

impl LocalProvider {
    pub fn new(_path: PathBuf) -> Self {
        Self
    }
}

impl LyricsProvider for LocalProvider {}
