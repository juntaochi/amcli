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

const SEARCH_RANK_BONUS: u16 = 120;
const SEARCH_RANK_DECAY: u16 = 6;
const DURATION_MATCH_BONUS: u16 = 200;
const DURATION_TOLERANCE: Duration = Duration::from_secs(3);

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
            .enumerate()
            .filter_map(|(rank, song)| {
                let id = song["id"].as_i64()?;
                Self::ranked_song_match_score(song, track, rank).map(|score| (score, rank, id))
            })
            .max_by(|(left_score, left_rank, _), (right_score, right_rank, _)| {
                left_score
                    .cmp(right_score)
                    .then_with(|| right_rank.cmp(left_rank))
            })
            .map(|(_, _, id)| id)
    }

    #[cfg(test)]
    fn select_song_id_from_results(results: &[Value], track: &Track) -> Option<i64> {
        results
            .iter()
            .find_map(|json| Self::select_song_id(json, track))
    }

    fn search_queries(track: &Track) -> Vec<String> {
        let mut queries = Vec::new();
        Self::push_search_query(&mut queries, &[&track.name, &track.album, &track.artist]);
        Self::push_search_query(&mut queries, &[&track.name, &track.artist]);
        Self::push_search_query(&mut queries, &[&track.name]);
        queries
    }

    fn push_search_query(queries: &mut Vec<String>, parts: &[&str]) {
        let query = parts
            .iter()
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        if !query.is_empty() && !queries.iter().any(|existing| existing == &query) {
            queries.push(query);
        }
    }

    fn search_url(query: &str) -> String {
        format!(
            "https://music.163.com/api/cloudsearch/pc?s={}&type=1&limit=20",
            urlencoding::encode(query)
        )
    }

    fn song_match_score(song: &Value, track: &Track) -> Option<u16> {
        let artist_names = Self::artist_names(song);
        let candidate = RemoteLyricsCandidate {
            track_name: song["name"].as_str(),
            artist_names: &artist_names,
            album_name: song["al"]["name"]
                .as_str()
                .or_else(|| song["album"]["name"].as_str()),
            duration: Self::song_duration(song),
        };

        remote_lyrics_match_score(track, &candidate)
    }

    fn ranked_song_match_score(song: &Value, track: &Track, rank: usize) -> Option<u16> {
        Some(
            Self::song_match_score(song, track)?
                + Self::search_rank_score(rank)
                + Self::duration_score(song, track),
        )
    }

    fn search_rank_score(rank: usize) -> u16 {
        let rank = u16::try_from(rank).unwrap_or(u16::MAX);
        SEARCH_RANK_BONUS.saturating_sub(rank.saturating_mul(SEARCH_RANK_DECAY))
    }

    fn duration_score(song: &Value, track: &Track) -> u16 {
        let Some(duration) = Self::song_duration(song) else {
            return 0;
        };
        let diff = track.duration.abs_diff(duration);
        if diff > DURATION_TOLERANCE {
            return 0;
        }

        let penalty = u16::try_from(diff.as_millis() / 100).unwrap_or(u16::MAX);
        DURATION_MATCH_BONUS.saturating_sub(penalty)
    }

    fn song_duration(song: &Value) -> Option<Duration> {
        song["dt"]
            .as_u64()
            .or_else(|| song["duration"].as_u64())
            .map(Duration::from_millis)
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
        let mut song_id = None;
        for query in Self::search_queries(track) {
            let response = self.client.get(Self::search_url(&query)).send().await?;
            let json = response.json::<Value>().await?;

            if let Some(id) = Self::select_song_id(&json, track) {
                song_id = Some(id);
                break;
            }
        }

        let song_id = match song_id {
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
#[path = "netease_tests.rs"]
mod tests;
