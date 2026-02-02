pub mod local;
pub mod lrclib;
pub mod netease;
use crate::player::Track;
use anyhow::Result;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Lyrics {
    pub lines: Vec<LyricsLine>,
}

impl Lyrics {
    pub fn find_index(&self, _pos: Duration) -> usize {
        0
    }
}

#[derive(Clone, Debug)]
pub struct LyricsLine {
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct LyricsManager;

impl LyricsManager {
    pub fn new(_size: usize) -> Self {
        Self
    }

    pub fn add_provider(&mut self, _provider: Box<dyn LyricsProvider>) {}

    pub async fn get_lyrics(&self, _track: &Track) -> Result<Option<Lyrics>> {
        Ok(None)
    }
}

pub trait LyricsProvider: Send + Sync {}
