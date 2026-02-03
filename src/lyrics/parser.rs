// src/lyrics/parser.rs
use crate::lyrics::{LyricLine, Lyrics};
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::time::Duration;

lazy_static! {
    // Matches [mm:ss.xx] or [mm:ss.xxx]
    static ref TIME_REGEX: Regex = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\]").unwrap();
    // Matches [key:value]
    static ref META_REGEX: Regex = Regex::new(r"\[([a-z]+):(.*)\]").unwrap();
}

pub fn parse_lrc(content: &str) -> Result<Lyrics> {
    let mut lyrics = Lyrics::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(caps) = META_REGEX.captures(line) {
            let key = caps[1].to_string();
            let value = caps[2].trim().to_string();

            if key == "offset" {
                lyrics.offset = value.parse().unwrap_or(0);
            } else {
                lyrics.metadata.insert(key, value);
            }

            if !TIME_REGEX.is_match(line) {
                continue;
            }
        }

        if !TIME_REGEX.is_match(line) {
            continue;
        }

        let text = TIME_REGEX.replace_all(line, "").trim().to_string();

        if text.is_empty() {
            continue;
        }

        for caps in TIME_REGEX.captures_iter(line) {
            let min: u64 = caps[1].parse()?;
            let sec: u64 = caps[2].parse()?;
            let ms_str = &caps[3];

            let ms: u64 = if ms_str.len() == 2 {
                ms_str.parse::<u64>()? * 10
            } else {
                ms_str.parse::<u64>()?
            };

            let total_ms = (min * 60 + sec) * 1000 + ms;
            lyrics.lines.push(LyricLine {
                timestamp: Duration::from_millis(total_ms),
                text: text.clone(),
            });
        }
    }

    lyrics.lines.sort_by_key(|l| l.timestamp);

    if lyrics.offset != 0 {
        let offset_dur = Duration::from_millis(lyrics.offset.abs() as u64);
        for line in lyrics.lines.iter_mut() {
            if lyrics.offset > 0 {
                line.timestamp += offset_dur;
            } else {
                line.timestamp = line.timestamp.saturating_sub(offset_dur);
            }
        }
    }

    Ok(lyrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let lrc = "[00:12.34]Hello world";
        let lyrics = parse_lrc(lrc).unwrap();
        assert_eq!(lyrics.lines.len(), 1);
        assert_eq!(lyrics.lines[0].timestamp, Duration::from_millis(12340));
        assert_eq!(lyrics.lines[0].text, "Hello world");
    }

    #[test]
    fn test_parse_multiple_timestamps() {
        let lrc = "[00:12.34][00:15.00]Repeated line";
        let lyrics = parse_lrc(lrc).unwrap();
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.lines[0].timestamp, Duration::from_millis(12340));
        assert_eq!(lyrics.lines[1].timestamp, Duration::from_millis(15000));
        assert_eq!(lyrics.lines[0].text, "Repeated line");
        assert_eq!(lyrics.lines[1].text, "Repeated line");
    }

    #[test]
    fn test_parse_metadata() {
        let lrc = "[ti:Title]\n[ar:Artist]\n[00:01.00]Lyrics";
        let lyrics = parse_lrc(lrc).unwrap();
        assert_eq!(lyrics.metadata.get("ti").unwrap(), "Title");
        assert_eq!(lyrics.metadata.get("ar").unwrap(), "Artist");
        assert_eq!(lyrics.lines[0].text, "Lyrics");
    }

    #[test]
    fn test_parse_offset() {
        let lrc = "[offset:500]\n[00:01.00]Lyrics";
        let lyrics = parse_lrc(lrc).unwrap();
        assert_eq!(lyrics.offset, 500);
        assert_eq!(lyrics.lines[0].timestamp, Duration::from_millis(1500));
    }

    #[test]
    fn test_parse_negative_offset() {
        let lrc = "[offset:-500]\n[00:01.00]Lyrics";
        let lyrics = parse_lrc(lrc).unwrap();
        assert_eq!(lyrics.offset, -500);
        assert_eq!(lyrics.lines[0].timestamp, Duration::from_millis(500));
    }

    #[test]
    fn test_filter_non_timestamped_lines() {
        let lrc = "作词 : 周杰伦\n作曲 : 周杰伦\n[00:12.34]真正的歌词\n纯文本行\n[00:15.00]第二行";
        let lyrics = parse_lrc(lrc).unwrap();
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.lines[0].text, "真正的歌词");
        assert_eq!(lyrics.lines[1].text, "第二行");
    }
}
