use crate::lyrics::parser::parse_lrc;
use crate::lyrics::provider::LyricsProvider;
use crate::lyrics::Lyrics;
use crate::player::Track;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
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
}

#[async_trait]
impl LyricsProvider for NeteaseProvider {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
        let query = format!("{} {}", track.name, track.artist);

        let search_url = format!(
            "https://music.163.com/api/cloudsearch/pc?s={}&type=1&limit=1",
            urlencoding::encode(&query)
        );

        let response = self.client.get(&search_url).send().await?;
        let json = response.json::<serde_json::Value>().await?;

        let song_id = match json["result"]["songs"][0]["id"].as_i64() {
            Some(id) => id,
            None => {
                tracing::debug!("Song not found on Netease");
                return Ok(None);
            }
        };

        let lyrics_url = format!(
            "https://music.163.com/api/song/lyric?id={}&lv=-1&kv=-1&tv=-1",
            song_id
        );

        let response = self.client.get(&lyrics_url).send().await?;
        let json = response.json::<serde_json::Value>().await?;

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
