// src/lyrics/lrclib.rs
use crate::lyrics::matching::{remote_lyrics_match_score, RemoteLyricsCandidate};
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

    fn headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(concat!(
                "AMCLI v",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/juntaochi/amcli)"
            )),
        );
        headers.insert(
            "Lrclib-Client",
            HeaderValue::from_static(concat!("AMCLI v", env!("CARGO_PKG_VERSION"))),
        );
        headers
    }

    fn extract_lyrics(json: &Value) -> Result<Option<Lyrics>> {
        if let Some(synced_lyrics) = json["syncedLyrics"].as_str() {
            if !synced_lyrics.trim().is_empty() {
                return Ok(Some(parse_lrc(synced_lyrics)?));
            }
        }

        if let Some(plain_lyrics) = json["plainLyrics"].as_str() {
            if !plain_lyrics.trim().is_empty() {
                return Ok(Some(parse_lrc(plain_lyrics)?));
            }
        }

        Ok(None)
    }

    fn extract_lyrics_for_track(json: &Value, track: &Track) -> Result<Option<Lyrics>> {
        if Self::record_match_score(json, track).is_none() {
            return Ok(None);
        }

        Self::extract_lyrics(json)
    }

    fn select_best_record<'a>(records: &'a [Value], track: &Track) -> Option<&'a Value> {
        records
            .iter()
            .filter(|record| Self::has_lyrics(record))
            .filter_map(|record| {
                Self::record_match_score(record, track).map(|score| (score, record))
            })
            .max_by_key(|(score, _)| *score)
            .map(|(_, record)| record)
    }

    fn record_match_score(json: &Value, track: &Track) -> Option<u16> {
        let artist_name = string_field(json, "artistName")?;
        let artist_names = [artist_name];
        let candidate = RemoteLyricsCandidate {
            track_name: string_field(json, "trackName").or_else(|| string_field(json, "name")),
            artist_names: &artist_names,
            album_name: string_field(json, "albumName"),
            duration: duration_seconds_field(json, "duration"),
        };

        remote_lyrics_match_score(track, &candidate)
    }

    fn has_lyrics(json: &Value) -> bool {
        json["syncedLyrics"]
            .as_str()
            .map(|lyrics| !lyrics.trim().is_empty())
            .unwrap_or(false)
            || json["plainLyrics"]
                .as_str()
                .map(|lyrics| !lyrics.trim().is_empty())
                .unwrap_or(false)
    }
}

fn string_field<'a>(json: &'a Value, field: &str) -> Option<&'a str> {
    json[field]
        .as_str()
        .filter(|value| !value.trim().is_empty())
}

fn duration_seconds_field(json: &Value, field: &str) -> Option<Duration> {
    let seconds = json[field].as_f64()?;
    if seconds.is_sign_negative() {
        return None;
    }

    Some(Duration::from_secs_f64(seconds))
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
            .headers(Self::headers())
            .send()
            .await;

        if let Ok(resp) = precise_result {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(lyrics) = Self::extract_lyrics_for_track(&json, track)? {
                        tracing::info!("LRCLIB: Found lyrics via precise match");
                        return Ok(Some(lyrics));
                    }
                }
            }
        }

        // Stage 2: Search candidates and choose the best local match.
        tracing::debug!("LRCLIB: Precise match failed, searching candidates");
        let url_search = format!(
            "https://lrclib.net/api/search?artist_name={}&track_name={}",
            urlencoding::encode(&track.artist),
            urlencoding::encode(&track.name)
        );

        let search_result = self
            .client
            .get(&url_search)
            .headers(Self::headers())
            .send()
            .await;

        if let Ok(resp) = search_result {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(records) = json.as_array() {
                        if let Some(record) = Self::select_best_record(records, track) {
                            if let Some(lyrics) = Self::extract_lyrics(record)? {
                                tracing::info!("LRCLIB: Found lyrics via scored search match");
                                return Ok(Some(lyrics));
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_use_crate_version() {
        let headers = LrclibProvider::headers();
        let version = env!("CARGO_PKG_VERSION");

        assert_eq!(
            headers.get(USER_AGENT).unwrap().to_str().unwrap(),
            format!("AMCLI v{} (https://github.com/juntaochi/amcli)", version)
        );
        assert_eq!(
            headers.get("Lrclib-Client").unwrap().to_str().unwrap(),
            format!("AMCLI v{}", version)
        );
    }

    fn track() -> Track {
        Track {
            name: "Same Song".into(),
            artist: "Same Artist".into(),
            album: "Studio Album".into(),
            duration: Duration::from_secs(240),
            position: Duration::ZERO,
        }
    }

    #[test]
    fn selects_matching_lrclib_record_instead_of_first_loose_result() {
        let records = serde_json::json!([
            {
                "id": 1,
                "trackName": "Same Song",
                "artistName": "Same Artist",
                "albumName": "Live Album",
                "duration": 260,
                "syncedLyrics": "[00:01.00]wrong version"
            },
            {
                "id": 2,
                "trackName": "Same Song",
                "artistName": "Same Artist",
                "albumName": "Studio Album",
                "duration": 240,
                "syncedLyrics": "[00:01.00]right version"
            }
        ]);

        let selected = LrclibProvider::select_best_record(records.as_array().unwrap(), &track())
            .expect("expected matching LRCLIB record");

        assert_eq!(selected["id"].as_i64(), Some(2));
    }

    #[test]
    fn rejects_lrclib_record_with_same_title_artist_but_wrong_version() {
        let record = serde_json::json!({
            "id": 1,
            "trackName": "Same Song",
            "artistName": "Same Artist",
            "albumName": "Live Album",
            "duration": 260,
            "syncedLyrics": "[00:01.00]wrong version"
        });

        let lyrics = LrclibProvider::extract_lyrics_for_track(&record, &track()).unwrap();

        assert!(lyrics.is_none());
    }
}
