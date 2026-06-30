use crate::player::Track;
use anyhow::{anyhow, Result};
use futures::StreamExt;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::Duration;

const PROVIDER_TIMEOUT: Duration = Duration::from_secs(12);

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
    cache: std::sync::Arc<Mutex<LruCache<String, Lyrics>>>,
    // Session-calibrated primary provider, chosen on the first race that yields a
    // clear winner. `None` until then, so early lookups keep racing.
    primary: std::sync::Arc<Mutex<Option<usize>>>,
}

// Outcome of querying a single provider for one track.
enum Probe {
    Hit(Lyrics),
    Miss,
    Fail,
}

impl LyricsManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            providers: Vec::new(),
            cache: std::sync::Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("lyrics cache capacity must be non-zero"),
            ))),
            primary: std::sync::Arc::new(Mutex::new(None)),
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
                return Ok(Some(cached.clone()));
            }
        }

        tracing::debug!(
            "Lyrics cache miss, querying providers for: {} - {}",
            track.name,
            track.artist
        );

        // Once a provider has won a race it becomes the session primary: a single
        // request in the common case. Until then, race every provider concurrently.
        let result = match self.calibrated_primary() {
            Some(primary) => self.fetch_sequential(track, primary).await,
            None => self.fetch_race(track).await,
        };

        match &result {
            Ok(Some(lyrics)) => {
                if let Ok(mut cache) = self.cache.lock() {
                    cache.put(cache_key, lyrics.clone());
                }
            }
            Ok(None) => {
                tracing::debug!("No lyrics found for: {} - {}", track.name, track.artist);
            }
            Err(e) => {
                tracing::debug!(
                    "Lyrics providers unreachable for {} - {}: {}",
                    track.name,
                    track.artist,
                    e
                );
            }
        }

        result
    }

    fn calibrated_primary(&self) -> Option<usize> {
        self.primary.lock().ok().and_then(|p| *p)
    }

    // Provider indices ordered by their static priority (lower first). Used as the
    // race ordering and as the fallback order once a primary is calibrated.
    fn priority_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.providers.len()).collect();
        order.sort_by_key(|&i| self.providers[i].priority());
        order
    }

    // Calibrated path: try the primary first, then the rest as fallback.
    async fn fetch_sequential(&self, track: &Track, primary: usize) -> Result<Option<Lyrics>> {
        let mut order = vec![primary];
        order.extend(self.priority_order().into_iter().filter(|&i| i != primary));

        let mut saw_miss = false;
        let mut saw_fail = false;
        for idx in order {
            match probe_provider(self.providers[idx].clone(), track).await {
                Probe::Hit(lyrics) => {
                    tracing::debug!("Lyrics found via provider: {}", self.providers[idx].name());
                    return Ok(Some(lyrics));
                }
                Probe::Miss => saw_miss = true,
                Probe::Fail => saw_fail = true,
            }
        }
        unreachable_or_empty(saw_fail, saw_miss)
    }

    // Uncalibrated path: query every provider concurrently and return the first one
    // that actually yields lyrics — a true race, so a fast provider never waits on a
    // slow or hung rival. Arrival order through `FuturesUnordered` is the latency
    // order, which is all calibration needs. Lock the winner as the session primary
    // unless a healthy rival answered faster with no match (the "cold song" case):
    // there the latency signal is ambiguous, so leave calibration open and re-race
    // next track. A rival that failed or timed out before the hit is a reachability
    // win and locks immediately.
    async fn fetch_race(&self, track: &Track) -> Result<Option<Lyrics>> {
        let mut probes: futures::stream::FuturesUnordered<_> = self
            .priority_order()
            .into_iter()
            .map(|idx| {
                let provider = self.providers[idx].clone();
                async move { (idx, probe_provider(provider, track).await) }
            })
            .collect();

        let mut saw_fail = false;
        let mut saw_miss = false;

        while let Some((idx, outcome)) = probes.next().await {
            match outcome {
                Probe::Hit(lyrics) => {
                    // `saw_miss` here means a healthy rival answered (empty) before
                    // this hit — the ambiguous "cold song" case — so we don't lock.
                    if saw_fail || !saw_miss {
                        if let Ok(mut primary) = self.primary.lock() {
                            *primary = Some(idx);
                        }
                        tracing::debug!(
                            "Lyrics provider calibrated: primary = {}",
                            self.providers[idx].name()
                        );
                    }
                    return Ok(Some(lyrics));
                }
                Probe::Miss => saw_miss = true,
                Probe::Fail => saw_fail = true,
            }
        }

        unreachable_or_empty(saw_fail, saw_miss)
    }
}

// No provider produced lyrics. If every provider that responded failed (none merely
// reported "no match"), the sources are unreachable — surface that as an error so the
// caller can distinguish "no signal" from "no lyrics". Otherwise it is a genuine miss.
fn unreachable_or_empty(saw_fail: bool, saw_miss: bool) -> Result<Option<Lyrics>> {
    if saw_fail && !saw_miss {
        Err(anyhow!("all lyrics providers were unreachable"))
    } else {
        Ok(None)
    }
}

async fn probe_provider(
    provider: std::sync::Arc<dyn provider::LyricsProvider>,
    track: &Track,
) -> Probe {
    match tokio::time::timeout(PROVIDER_TIMEOUT, provider.get_lyrics(track)).await {
        Ok(Ok(Some(lyrics))) if !lyrics.lines.is_empty() => Probe::Hit(lyrics),
        Ok(Ok(_)) => {
            tracing::debug!("Provider {} returned no lyrics", provider.name());
            Probe::Miss
        }
        Ok(Err(e)) => {
            tracing::debug!("Provider {} failed: {}", provider.name(), e);
            Probe::Fail
        }
        Err(_) => {
            tracing::debug!(
                "Provider {} timed out after {:?}",
                provider.name(),
                PROVIDER_TIMEOUT
            );
            Probe::Fail
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lyrics::provider::LyricsProvider;
    use crate::lyrics::{lrclib::LrclibProvider, netease::NeteaseProvider};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    struct RecoveringProvider {
        calls: AtomicUsize,
    }

    struct SlowProvider {
        delay: Duration,
    }

    #[async_trait]
    impl LyricsProvider for RecoveringProvider {
        async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
            if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
                return Ok(None);
            }

            Ok(Some(Lyrics {
                lines: vec![LyricLine {
                    text: track.name.clone(),
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
            "recovering"
        }
    }

    #[async_trait]
    impl LyricsProvider for SlowProvider {
        async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
            tokio::time::sleep(self.delay).await;
            Ok(Some(Lyrics {
                lines: vec![LyricLine {
                    text: track.name.clone(),
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
            "slow"
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

    #[tokio::test]
    async fn does_not_cache_empty_lookup_so_later_attempt_can_recover() {
        let mut manager = LyricsManager::new(4);
        manager.add_provider(Box::new(RecoveringProvider {
            calls: AtomicUsize::new(0),
        }));

        assert!(manager
            .get_lyrics(&track("Studio Album", 240))
            .await
            .unwrap()
            .is_none());

        let recovered = manager
            .get_lyrics(&track("Studio Album", 240))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(recovered.lines[0].text, "Same Song");
    }

    #[tokio::test]
    async fn provider_timeout_allows_slow_remote_lookup() {
        let mut manager = LyricsManager::new(4);
        manager.add_provider(Box::new(SlowProvider {
            delay: Duration::from_secs(6),
        }));

        let lyrics = manager
            .get_lyrics(&track("Studio Album", 240))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(lyrics.lines[0].text, "Same Song");
    }

    #[test]
    fn provider_priorities_try_netease_before_lrclib() {
        let lrclib = LrclibProvider::new();
        let netease = NeteaseProvider::new();

        assert!(netease.priority() < lrclib.priority());
    }

    enum TestOutcome {
        Hit,
        Miss,
        Fail,
    }

    struct ProbeProvider {
        name: &'static str,
        priority: u8,
        delay: Duration,
        outcome: TestOutcome,
        calls: std::sync::Arc<AtomicUsize>,
    }

    #[async_trait]
    impl LyricsProvider for ProbeProvider {
        async fn get_lyrics(&self, _track: &Track) -> Result<Option<Lyrics>> {
            tokio::time::sleep(self.delay).await;
            self.calls.fetch_add(1, Ordering::SeqCst);
            match self.outcome {
                TestOutcome::Hit => Ok(Some(Lyrics {
                    lines: vec![LyricLine {
                        text: self.name.to_string(),
                        timestamp: Duration::ZERO,
                    }],
                    metadata: HashMap::new(),
                    offset: 0,
                })),
                TestOutcome::Miss => Ok(None),
                TestOutcome::Fail => Err(anyhow::anyhow!("probe failure")),
            }
        }

        fn priority(&self) -> u8 {
            self.priority
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    fn probe_provider_for(
        name: &'static str,
        priority: u8,
        delay_ms: u64,
        outcome: TestOutcome,
    ) -> (Box<ProbeProvider>, std::sync::Arc<AtomicUsize>) {
        let calls = std::sync::Arc::new(AtomicUsize::new(0));
        let provider = Box::new(ProbeProvider {
            name,
            priority,
            delay: Duration::from_millis(delay_ms),
            outcome,
            calls: calls.clone(),
        });
        (provider, calls)
    }

    #[tokio::test]
    async fn race_locks_faster_provider_and_reuses_it() {
        let (fast, fast_calls) = probe_provider_for("fast", 5, 0, TestOutcome::Hit);
        let (slow, slow_calls) = probe_provider_for("slow", 10, 50, TestOutcome::Hit);

        let mut manager = LyricsManager::new(8);
        manager.add_provider(fast);
        manager.add_provider(slow);

        // First lookup races both; the fast hit returns immediately and is locked as
        // primary — the slow rival is cancelled before it even records a call, which
        // is exactly the "don't wait on the slow source" property we want.
        let first = manager.get_lyrics(&track("A", 1)).await.unwrap().unwrap();
        assert_eq!(first.lines[0].text, "fast");
        assert_eq!(fast_calls.load(Ordering::SeqCst), 1);
        assert_eq!(slow_calls.load(Ordering::SeqCst), 0);

        // Second lookup (new track to dodge the cache) only touches the primary.
        let second = manager.get_lyrics(&track("B", 2)).await.unwrap().unwrap();
        assert_eq!(second.lines[0].text, "fast");
        assert_eq!(fast_calls.load(Ordering::SeqCst), 2);
        assert_eq!(slow_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn cold_song_does_not_lock_calibration() {
        // Only the rival has this song; both stay healthy, so there is no clean
        // latency signal and calibration must remain open.
        // miss answers first (fast, no match); the hit arrives a touch later.
        let (miss, miss_calls) = probe_provider_for("miss", 5, 0, TestOutcome::Miss);
        let (hit, hit_calls) = probe_provider_for("hit", 10, 20, TestOutcome::Hit);

        let mut manager = LyricsManager::new(8);
        manager.add_provider(miss);
        manager.add_provider(hit);

        let first = manager.get_lyrics(&track("A", 1)).await.unwrap().unwrap();
        assert_eq!(first.lines[0].text, "hit");
        assert_eq!(miss_calls.load(Ordering::SeqCst), 1);
        assert_eq!(hit_calls.load(Ordering::SeqCst), 1);

        // Not locked: the next lookup races both providers again.
        let second = manager.get_lyrics(&track("B", 2)).await.unwrap().unwrap();
        assert_eq!(second.lines[0].text, "hit");
        assert_eq!(miss_calls.load(Ordering::SeqCst), 2);
        assert_eq!(hit_calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn failure_locks_the_working_provider() {
        // A failing rival is a reachability signal: lock the working provider even
        // though only one returned lyrics.
        // down fails fast; the working provider answers a touch later.
        let (down, down_calls) = probe_provider_for("down", 5, 0, TestOutcome::Fail);
        let (up, up_calls) = probe_provider_for("up", 10, 20, TestOutcome::Hit);

        let mut manager = LyricsManager::new(8);
        manager.add_provider(down);
        manager.add_provider(up);

        let first = manager.get_lyrics(&track("A", 1)).await.unwrap().unwrap();
        assert_eq!(first.lines[0].text, "up");
        assert_eq!(down_calls.load(Ordering::SeqCst), 1);
        assert_eq!(up_calls.load(Ordering::SeqCst), 1);

        // Locked onto the working provider: the failing one is not retried.
        let second = manager.get_lyrics(&track("B", 2)).await.unwrap().unwrap();
        assert_eq!(second.lines[0].text, "up");
        assert_eq!(down_calls.load(Ordering::SeqCst), 1);
        assert_eq!(up_calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn all_providers_unreachable_surface_as_error() {
        let (down, _) = probe_provider_for("down", 5, 0, TestOutcome::Fail);
        let mut manager = LyricsManager::new(4);
        manager.add_provider(down);

        // Every provider failed (none merely reported "no match") → error, not Ok(None).
        assert!(manager.get_lyrics(&track("A", 1)).await.is_err());
    }

    #[tokio::test]
    async fn unreachable_provider_with_a_healthy_miss_is_not_an_error() {
        let (down, _) = probe_provider_for("down", 5, 0, TestOutcome::Fail);
        let (miss, _) = probe_provider_for("miss", 10, 0, TestOutcome::Miss);
        let mut manager = LyricsManager::new(4);
        manager.add_provider(down);
        manager.add_provider(miss);

        // One source is down, but a reachable source simply had no match → Ok(None).
        assert!(manager.get_lyrics(&track("A", 1)).await.unwrap().is_none());
    }
}
