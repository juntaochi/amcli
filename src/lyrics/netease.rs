use crate::lyrics::matching::{remote_lyrics_match_score, RemoteLyricsCandidate};
use crate::lyrics::parser::parse_lrc;
use crate::lyrics::provider::LyricsProvider;
use crate::lyrics::Lyrics;
use crate::player::Track;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct NeteaseProvider {
    client: Client,
}

impl NeteaseProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .build()
                .unwrap_or_default(),
        }
    }

    fn select_song_id(json: &Value, track: &Track) -> Option<i64> {
        json["result"]["songs"]
            .as_array()?
            .iter()
            .filter_map(|song| {
                let id = song["id"].as_i64()?;
                Self::song_match_score(song, track).map(|score| (score, id))
            })
            .max_by_key(|(score, _)| *score)
            .map(|(_, id)| id)
    }

    fn song_match_score(song: &Value, track: &Track) -> Option<u16> {
        let artist_names = Self::artist_names(song);
        let candidate = RemoteLyricsCandidate {
            track_name: song["name"].as_str(),
            artist_names: &artist_names,
            album_name: song["al"]["name"]
                .as_str()
                .or_else(|| song["album"]["name"].as_str()),
            duration: song["dt"]
                .as_u64()
                .or_else(|| song["duration"].as_u64())
                .map(Duration::from_millis),
        };

        remote_lyrics_match_score(track, &candidate)
    }

    fn artist_names(song: &Value) -> Vec<&str> {
        song["ar"]
            .as_array()
            .or_else(|| song["artists"].as_array())
            .map(|artists| {
                artists
                    .iter()
                    .filter_map(|artist| artist["name"].as_str())
                    .filter(|name| !name.trim().is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[async_trait]
impl LyricsProvider for NeteaseProvider {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
        let query = format!("{} {}", track.name, track.artist);

        let search_url = format!(
            "https://music.163.com/api/cloudsearch/pc?s={}&type=1&limit=10",
            urlencoding::encode(&query)
        );

        let response = self.client.get(&search_url).send().await?;
        let json = response.json::<Value>().await?;

        let song_id = match Self::select_song_id(&json, track) {
            Some(id) => id,
            None => {
                tracing::debug!("No confident Netease song match found");
                return Ok(None);
            }
        };

        let lyrics_url = format!(
            "https://music.163.com/api/song/lyric?id={}&lv=-1&kv=-1&tv=-1",
            song_id
        );

        let response = self.client.get(&lyrics_url).send().await?;
        let json = response.json::<Value>().await?;

        if let Some(lrc_text) = json["lrc"]["lyric"].as_str() {
            if !lrc_text.is_empty() {
                return Ok(Some(parse_lrc(lrc_text)?));
            }
        }

        tracing::debug!("No lyric content found");
        Ok(None)
    }

    fn priority(&self) -> u8 {
        10
    }

    fn name(&self) -> &'static str {
        "netease"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn selects_matching_netease_song_instead_of_first_search_result() {
        let json = serde_json::json!({
            "result": {
                "songs": [
                    {
                        "id": 1,
                        "name": "Same Song",
                        "ar": [{"name": "Same Artist"}],
                        "al": {"name": "Live Album"},
                        "dt": 260000
                    },
                    {
                        "id": 2,
                        "name": "Same Song",
                        "ar": [{"name": "Same Artist"}],
                        "al": {"name": "Studio Album"},
                        "dt": 240000
                    }
                ]
            }
        });

        assert_eq!(NeteaseProvider::select_song_id(&json, &track()), Some(2));
    }

    #[test]
    fn rejects_netease_song_with_same_title_artist_but_wrong_version() {
        let json = serde_json::json!({
            "result": {
                "songs": [
                    {
                        "id": 1,
                        "name": "Same Song",
                        "ar": [{"name": "Same Artist"}],
                        "al": {"name": "Live Album"},
                        "dt": 260000
                    }
                ]
            }
        });

        assert_eq!(NeteaseProvider::select_song_id(&json, &track()), None);
    }

    #[test]
    fn selects_netease_song_with_localized_artist_when_album_and_duration_match() {
        let target = Track {
            name: "LIGHT IT UP!".into(),
            artist: "YUZUHA".into(),
            album: "Light It Up!".into(),
            duration: Duration::from_millis(261_275),
            position: Duration::ZERO,
        };
        let json = serde_json::json!({
            "result": {
                "songs": [
                    {
                        "id": 2703859971i64,
                        "name": "Light It Up!",
                        "ar": [{"name": "柚子花"}],
                        "al": {"name": "Light It Up!"},
                        "dt": 261275
                    }
                ]
            }
        });

        assert_eq!(
            NeteaseProvider::select_song_id(&json, &target),
            Some(2703859971)
        );
    }
}
