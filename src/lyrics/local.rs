use crate::lyrics::parser::parse_lrc;
use crate::lyrics::provider::LyricsProvider;
use crate::lyrics::Lyrics;
use crate::player::Track;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

pub struct LocalProvider {
    lyrics_dir: PathBuf,
}

impl LocalProvider {
    pub fn new(lyrics_dir: PathBuf) -> Self {
        Self { lyrics_dir }
    }
}

#[async_trait]
impl LyricsProvider for LocalProvider {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
        if !self.lyrics_dir.exists() {
            return Ok(None);
        }

        let patterns = [
            format!("{} - {}.lrc", track.artist, track.name),
            format!("{}.lrc", track.name),
            format!("{} - {}.lrc", track.name, track.artist),
        ];

        for pattern in &patterns {
            let path = self.lyrics_dir.join(pattern);
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await?;
                return Ok(Some(parse_lrc(&content)?));
            }
        }

        Ok(None)
    }

    fn priority(&self) -> u8 {
        1 // Highest priority - local files first
    }

    fn name(&self) -> &'static str {
        "local"
    }
}
