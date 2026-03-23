// src/lyrics/lrclib.rs
use crate::lyrics::parser::parse_lrc;
use crate::lyrics::provider::LyricsProvider;
use crate::lyrics::Lyrics;
use crate::player::Track;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct LrclibProvider {
    client: Client,
}

impl LrclibProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("AMCLI v1.0.0 (https://github.com/juntaochi/amcli)"),
        );
        headers.insert("Lrclib-Client", HeaderValue::from_static("AMCLI v1.0.0"));
        headers
    }

    fn extract_lyrics(json: &Value) -> Result<Option<Lyrics>> {
        if let Some(synced_lyrics) = json["syncedLyrics"].as_str() {
            if !synced_lyrics.is_empty() {
                return Ok(Some(parse_lrc(synced_lyrics)?));
            }
        }

        if let Some(plain_lyrics) = json["plainLyrics"].as_str() {
            if !plain_lyrics.is_empty() {
                return Ok(Some(parse_lrc(plain_lyrics)?));
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl LyricsProvider for LrclibProvider {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
        // Stage 1: Try precise match with album and duration
        let url_precise = format!(
            "https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}&duration={}",
            urlencoding::encode(&track.artist),
            urlencoding::encode(&track.name),
            urlencoding::encode(&track.album),
            track.duration.as_secs()
        );

        tracing::debug!(
            "LRCLIB: Trying precise match for '{} - {}'",
            track.artist,
            track.name
        );

        let precise_result = self
            .client
            .get(&url_precise)
            .headers(self.headers())
            .send()
            .await;

        if let Ok(resp) = precise_result {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(lyrics) = Self::extract_lyrics(&json)? {
                        tracing::info!("LRCLIB: Found lyrics via precise match");
                        return Ok(Some(lyrics));
                    }
                }
            }
        }

        // Stage 2: Fallback to loose match (only artist and track)
        tracing::debug!("LRCLIB: Precise match failed, trying loose match");
        let url_loose = format!(
            "https://lrclib.net/api/get?artist_name={}&track_name={}",
            urlencoding::encode(&track.artist),
            urlencoding::encode(&track.name)
        );

        let loose_result = self
            .client
            .get(&url_loose)
            .headers(self.headers())
            .send()
            .await;

        if let Ok(resp) = loose_result {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(lyrics) = Self::extract_lyrics(&json)? {
                        tracing::info!("LRCLIB: Found lyrics via loose match");
                        return Ok(Some(lyrics));
                    }
                }
            }
        }

        tracing::debug!(
            "LRCLIB: No lyrics found for '{} - {}'",
            track.artist,
            track.name
        );
        Ok(None)
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &'static str {
        "lrclib"
    }
}
