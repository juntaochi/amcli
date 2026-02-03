use crate::player::Track;
use anyhow::Result;
use std::time::Duration;

pub mod local;
pub mod lrclib;
pub mod netease;

#[derive(Clone, Debug)]
pub struct LyricLine {
    pub text: String,
    #[allow(dead_code)]
    pub timestamp: Duration,
}

#[derive(Clone, Debug)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
}

impl Lyrics {
    pub fn find_index(&self, _position: Duration) -> usize {
        0
    }
}

pub trait LyricsProvider: Send + Sync {}

#[derive(Clone)]
pub struct LyricsManager;

impl LyricsManager {
    pub fn new(_capacity: usize) -> Self {
        Self
    }

    pub fn add_provider(&mut self, _provider: Box<dyn LyricsProvider>) {}

    pub async fn get_lyrics(&self, _track: &Track) -> Result<Option<Lyrics>> {
        Ok(None)
    }
}
