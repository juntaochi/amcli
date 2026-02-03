// src/lyrics/provider.rs
use crate::lyrics::Lyrics;
use crate::player::Track;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LyricsProvider: Send + Sync {
    /// Try to find lyrics for the given track
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>>;

    /// Priority of the provider (Lower = higher priority)
    fn priority(&self) -> u8;

    /// Name of the provider
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}
