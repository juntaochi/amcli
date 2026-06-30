use crate::player::Track;
use anyhow::Result;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::Duration;

pub mod local;
pub mod lrclib;
pub(crate) mod matching;
pub mod netease;
pub mod parser;
pub mod provider;

#[derive(Clone, Debug)]
pub struct LyricLine {
    pub text: String,
    pub timestamp: Duration,
}

#[derive(Clone, Debug)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub metadata: HashMap<String, String>,
    pub offset: i64,
}

impl Lyrics {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            metadata: HashMap::new(),
            offset: 0,
        }
    }

    pub fn find_index(&self, position: Duration) -> usize {
        let mut idx = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if line.timestamp <= position {
                idx = i;
            } else {
                break;
            }
        }
        idx
    }
}

#[derive(Clone)]
pub struct LyricsManager {
    providers: Vec<std::sync::Arc<dyn provider::LyricsProvider>>,
    cache: std::sync::Arc<Mutex<LruCache<String, Option<Lyrics>>>>,
}

impl LyricsManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            providers: Vec::new(),
            cache: std::sync::Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("lyrics cache capacity must be non-zero"),
            ))),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn provider::LyricsProvider>) {
        self.providers.push(std::sync::Arc::from(provider));
    }

    pub async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
        let cache_key = matching::track_cache_key(track);

        // Check cache
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                tracing::debug!("Lyrics cache hit for: {} - {}", track.name, track.artist);
                return Ok(cached.clone());
            }
        }

        tracing::debug!(
            "Lyrics cache miss, querying providers for: {} - {}",
            track.name,
            track.artist
        );

        // Try each provider in priority order
        let mut sorted_providers: Vec<_> = self.providers.iter().collect();
        sorted_providers.sort_by_key(|p| p.priority());

        for provider in sorted_providers {
            match tokio::time::timeout(Duration::from_secs(5), provider.get_lyrics(track)).await {
                Ok(Ok(Some(lyrics))) if !lyrics.lines.is_empty() => {
                    tracing::debug!("Lyrics found via provider: {}", provider.name());
                    if let Ok(mut cache) = self.cache.lock() {
                        cache.put(cache_key, Some(lyrics.clone()));
                    }
                    return Ok(Some(lyrics));
                }
                Ok(Ok(_)) => {
                    tracing::debug!("Provider {} returned no lyrics", provider.name());
                    continue;
                }
                Ok(Err(e)) => {
                    tracing::debug!("Provider {} failed: {}", provider.name(), e);
                    continue;
                }
                Err(_) => {
                    tracing::debug!("Provider {} timed out after 2s", provider.name());
                    continue;
                }
            }
        }

        tracing::debug!("No lyrics found for: {} - {}", track.name, track.artist);
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(cache_key, None);
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lyrics::provider::LyricsProvider;
    use async_trait::async_trait;

    struct AlbumEchoProvider;

    #[async_trait]
    impl LyricsProvider for AlbumEchoProvider {
        async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
            Ok(Some(Lyrics {
                lines: vec![LyricLine {
                    text: track.album.clone(),
                    timestamp: Duration::ZERO,
                }],
                metadata: HashMap::new(),
                offset: 0,
            }))
        }

        fn priority(&self) -> u8 {
            1
        }

        fn name(&self) -> &'static str {
            "album-echo"
        }
    }

    fn track(album: &str, duration_secs: u64) -> Track {
        Track {
            name: "Same Song".into(),
            artist: "Same Artist".into(),
            album: album.into(),
            duration: Duration::from_secs(duration_secs),
            position: Duration::ZERO,
        }
    }

    #[tokio::test]
    async fn cache_distinguishes_same_title_artist_across_album_versions() {
        let mut manager = LyricsManager::new(4);
        manager.add_provider(Box::new(AlbumEchoProvider));

        let first = manager
            .get_lyrics(&track("Studio Album", 240))
            .await
            .unwrap()
            .unwrap();
        let second = manager
            .get_lyrics(&track("Live Album", 260))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(first.lines[0].text, "Studio Album");
        assert_eq!(second.lines[0].text, "Live Album");
    }
}
