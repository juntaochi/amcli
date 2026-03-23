// src/lyrics/provider.rs
use crate::lyrics::Lyrics;
use crate::player::Track;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LyricsProvider: Send + Sync {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>>;
    fn priority(&self) -> u8;
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}
