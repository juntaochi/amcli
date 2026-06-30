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

    fn best_song_match(json: &Value, track: &Track) -> Option<(u16, i64)> {
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
            .map(|(score, _, id)| (score, id))
    }

    #[cfg(test)]
    fn select_song_id(json: &Value, track: &Track) -> Option<i64> {
        Self::best_song_match(json, track).map(|(_, id)| id)
    }

    #[cfg(test)]
    fn select_song_id_from_results(results: &[Value], track: &Track) -> Option<i64> {
        results
            .iter()
            .filter_map(|json| Self::best_song_match(json, track))
            .reduce(|best, candidate| {
                if candidate.0 > best.0 {
                    candidate
                } else {
                    best
                }
            })
            .map(|(_, id)| id)
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

    fn artist_search_url(query: &str) -> String {
        format!(
            "https://music.163.com/api/cloudsearch/pc?s={}&type=100&limit=5",
            urlencoding::encode(query)
        )
    }

    fn artist_albums_url(artist_id: i64) -> String {
        format!(
            "https://music.163.com/api/artist/albums/{}?id={}&offset=0&limit=50",
            artist_id, artist_id
        )
    }

    fn album_url(album_id: i64) -> String {
        format!("https://music.163.com/api/v1/album/{}", album_id)
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

        duration_score_from_diff(diff)
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

    fn select_artist_id(json: &Value, track: &Track) -> Option<i64> {
        json["result"]["artists"]
            .as_array()?
            .iter()
            .find_map(|artist| {
                let id = artist["id"].as_i64()?;
                if entity_alias_matches(&track.artist, artist, false) {
                    Some(id)
                } else {
                    None
                }
            })
    }

    fn select_album_ids(json: &Value, track: &Track) -> Vec<i64> {
        json["hotAlbums"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|album| {
                let id = album["id"].as_i64()?;
                if entity_alias_matches(&track.album, album, true) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }

    fn select_song_id_by_album_duration(json: &Value, track: &Track) -> Option<i64> {
        json["songs"]
            .as_array()?
            .iter()
            .enumerate()
            .filter_map(|(rank, song)| {
                let id = song["id"].as_i64()?;
                let duration = Self::song_duration(song)?;
                let diff = track.duration.abs_diff(duration);
                if diff > DURATION_TOLERANCE {
                    return None;
                }

                Some((duration_score_from_diff(diff), rank, id))
            })
            .max_by(|(left_score, left_rank, _), (right_score, right_rank, _)| {
                left_score
                    .cmp(right_score)
                    .then_with(|| right_rank.cmp(left_rank))
            })
            .map(|(_, _, id)| id)
    }

    #[cfg(test)]
    fn select_album_alias_song_id(
        albums_json: &Value,
        album_jsons: &[Value],
        track: &Track,
    ) -> Option<i64> {
        Self::select_album_ids(albums_json, track)
            .iter()
            .zip(album_jsons)
            .find_map(|(_, album_json)| Self::select_song_id_by_album_duration(album_json, track))
    }

    async fn get_album_alias_song_id(&self, track: &Track) -> Result<Option<i64>> {
        let artist_response = self
            .client
            .get(Self::artist_search_url(&track.artist))
            .send()
            .await?;
        let artist_json = artist_response.json::<Value>().await?;
        let Some(artist_id) = Self::select_artist_id(&artist_json, track) else {
            return Ok(None);
        };

        let albums_response = self
            .client
            .get(Self::artist_albums_url(artist_id))
            .send()
            .await?;
        let albums_json = albums_response.json::<Value>().await?;

        for album_id in Self::select_album_ids(&albums_json, track) {
            let album_response = self.client.get(Self::album_url(album_id)).send().await?;
            let album_json = album_response.json::<Value>().await?;
            if let Some(song_id) = Self::select_song_id_by_album_duration(&album_json, track) {
                return Ok(Some(song_id));
            }
        }

        Ok(None)
    }

    fn parse_lyrics(lrc_text: &str) -> Result<Lyrics> {
        let mut lyrics = parse_lrc(lrc_text)?;
        strip_leading_credit_lines(&mut lyrics);
        Ok(lyrics)
    }
}

fn duration_score_from_diff(diff: Duration) -> u16 {
    let penalty = u16::try_from(diff.as_millis() / 100).unwrap_or(u16::MAX);
    DURATION_MATCH_BONUS.saturating_sub(penalty)
}

fn entity_alias_matches(target: &str, entity: &Value, allow_suffix: bool) -> bool {
    metadata_value_matches(target, entity["name"].as_str(), allow_suffix)
        || entity["alias"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|alias| metadata_value_matches(target, Some(alias), allow_suffix))
        || entity["transNames"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|alias| metadata_value_matches(target, Some(alias), allow_suffix))
}

fn metadata_value_matches(target: &str, candidate: Option<&str>, allow_suffix: bool) -> bool {
    let target = normalize_metadata_text(target);
    let candidate = normalize_metadata_text(candidate.unwrap_or_default());

    !target.is_empty()
        && !candidate.is_empty()
        && (target == candidate
            || (allow_suffix && candidate.len() >= 4 && target.ends_with(&candidate)))
}

fn normalize_metadata_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn strip_leading_credit_lines(lyrics: &mut Lyrics) {
    let first_lyric_index = lyrics
        .lines
        .iter()
        .position(|line| !is_credit_line(&line.text))
        .unwrap_or(lyrics.lines.len());

    if first_lyric_index > 0 {
        lyrics.lines.drain(0..first_lyric_index);
    }
}

fn is_credit_line(text: &str) -> bool {
    text.contains(':') || text.contains('：')
}

#[async_trait]
impl LyricsProvider for NeteaseProvider {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
        // Run every search query concurrently and keep the highest-scoring match
        // across all of them. Stopping at the first query that returns *a* match
        // lets a polluted result set lock onto a same-title decoy (e.g. a generic
        // "Love Me Now" by another artist whose duration happens to land within
        // tolerance), even when a cleaner query holds the correct track.
        let searches = Self::search_queries(track).into_iter().map(|query| {
            let client = &self.client;
            async move {
                let json = client
                    .get(Self::search_url(&query))
                    .send()
                    .await
                    .ok()?
                    .json::<Value>()
                    .await
                    .ok()?;
                Self::best_song_match(&json, track)
            }
        });

        let best_match = futures::future::join_all(searches)
            .await
            .into_iter()
            .flatten()
            .reduce(|best, candidate| {
                if candidate.0 > best.0 {
                    candidate
                } else {
                    best
                }
            });

        let mut song_id = best_match.map(|(_, id)| id);
        if song_id.is_none() {
            song_id = self.get_album_alias_song_id(track).await?;
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
                return Ok(Some(Self::parse_lyrics(lrc_text)?));
            }
        }

        tracing::debug!("No lyric content found");
        Ok(None)
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &'static str {
        "netease"
    }
}

#[cfg(test)]
#[path = "netease_tests.rs"]
mod tests;
