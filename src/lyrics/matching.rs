use crate::player::Track;
use std::time::Duration;

const DURATION_TOLERANCE: Duration = Duration::from_secs(3);

pub(crate) struct RemoteLyricsCandidate<'a> {
    pub track_name: Option<&'a str>,
    pub artist_names: &'a [&'a str],
    pub album_name: Option<&'a str>,
    pub duration: Option<Duration>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct MatchOptions {
    pub allow_chinese_duration_fallback: bool,
}

/// A match score tagged with whether it was granted solely through the
/// Chinese duration fallback. Callers use the flag to tier matches: a
/// duration-only fallback match must never outrank a match with textual
/// agreement, no matter what bonuses are layered on top of the score.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RemoteLyricsScore {
    pub score: u16,
    pub duration_only_fallback: bool,
}

pub(crate) fn track_cache_key(track: &Track) -> String {
    format!(
        "{}|{}|{}|{}",
        track.artist,
        track.name,
        track.album,
        track.duration.as_secs()
    )
}

pub(crate) fn remote_lyrics_match_score(
    track: &Track,
    candidate: &RemoteLyricsCandidate<'_>,
) -> Option<u16> {
    remote_lyrics_match_score_with_options(track, candidate, MatchOptions::default())
        .map(|score| score.score)
}

pub(crate) fn remote_lyrics_match_score_with_options(
    track: &Track,
    candidate: &RemoteLyricsCandidate<'_>,
    options: MatchOptions,
) -> Option<RemoteLyricsScore> {
    let candidate_title = candidate.track_name?;
    let title_matches = normalized_eq(&track.name, candidate_title);

    let candidate_has_album = candidate
        .album_name
        .map(|album| !normalize_text(album).is_empty())
        .unwrap_or(false);
    let album_matches = candidate
        .album_name
        .map(|album| normalized_eq(&track.album, album))
        .unwrap_or(false);
    let duration_matches = candidate
        .duration
        .map(|duration| duration_within_tolerance(track.duration, duration))
        .unwrap_or(false);
    let has_disambiguator = candidate_has_album || candidate.duration.is_some();
    let artist_matches = artist_matches(&track.artist, candidate.artist_names);
    let chinese_duration_fallback = duration_matches
        && !title_matches
        && !artist_matches
        && !album_matches
        && options.allow_chinese_duration_fallback
        && english_metadata_can_match_chinese_candidate(track, candidate);

    if duration_matches
        && !(title_matches || artist_matches || album_matches || chinese_duration_fallback)
    {
        return None;
    }

    // Decoy guard for the Netease fallback-enabled path only: a Latin-titled
    // duration+title match with a disagreeing artist is likely a same-title
    // decoy there. On the default path (LRCLIB) this candidate remains valid.
    if options.allow_chinese_duration_fallback
        && duration_matches
        && title_matches
        && !artist_matches
        && !album_matches
        && latin_title_has_metadata_conflict(&track.name, candidate.artist_names)
    {
        return None;
    }

    if !duration_matches && !title_matches {
        return None;
    }

    if !duration_matches && !artist_matches {
        return None;
    }

    if !duration_matches && artist_matches && has_disambiguator && !album_matches {
        return None;
    }

    let mut score = 100;
    if duration_matches {
        score += 100;
    }
    if title_matches {
        score += 50;
    }
    if artist_matches {
        score += 30;
    }
    if album_matches {
        score += 20;
    }
    if !has_disambiguator {
        score -= 20;
    }

    Some(RemoteLyricsScore {
        score,
        duration_only_fallback: chinese_duration_fallback,
    })
}

fn normalized_eq(left: &str, right: &str) -> bool {
    let left_norm = normalize_text(left);
    if left_norm.is_empty() {
        return false;
    }

    if left_norm == normalize_text(right) {
        return true;
    }

    let left_base = normalize_text(strip_release_type_suffix(strip_trailing_qualifiers(left)));
    let right_base = normalize_text(strip_release_type_suffix(strip_trailing_qualifiers(right)));
    !left_base.is_empty() && left_base == right_base
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn strip_trailing_qualifiers(value: &str) -> &str {
    let mut trimmed = value.trim();
    loop {
        let Some(stripped) = strip_one_trailing_qualifier(trimmed) else {
            return trimmed;
        };
        trimmed = stripped.trim_end();
    }
}

fn strip_one_trailing_qualifier(value: &str) -> Option<&str> {
    for (open, close) in [("(", ")"), ("（", "）"), ("[", "]"), ("【", "】")] {
        if !value.ends_with(close) {
            continue;
        }

        let open_index = value.rfind(open)?;
        if open_index == 0 {
            return None;
        }

        return Some(&value[..open_index]);
    }

    None
}

fn strip_release_type_suffix(value: &str) -> &str {
    let trimmed = value.trim();
    let lower = trimmed.to_lowercase();
    for suffix in [" - ep", " - single", " ep", " single"] {
        if !lower.ends_with(suffix) {
            continue;
        }
        let prefix = &trimmed[..trimmed.len() - suffix.len()];
        if !prefix.trim().is_empty() {
            return prefix.trim_end();
        }
    }

    trimmed
}

fn artist_matches(track_artist: &str, candidate_artists: &[&str]) -> bool {
    if candidate_artists.is_empty() {
        return false;
    }

    let track_full = normalize_text(track_artist);
    if track_full.is_empty() {
        return false;
    }

    let candidate_joined = normalize_text(&candidate_artists.join(""));
    if candidate_joined == track_full {
        return true;
    }

    let track_parts = artist_parts(track_artist);
    let candidate_parts: Vec<_> = candidate_artists
        .iter()
        .flat_map(|artist| artist_parts(artist))
        .collect();

    match (track_parts.first(), candidate_parts.first()) {
        (Some(track_primary), Some(candidate_primary)) => track_primary == candidate_primary,
        _ => false,
    }
}

fn artist_parts(value: &str) -> Vec<String> {
    let mut normalized = value.to_lowercase();
    for marker in [
        " featuring ",
        " feat. ",
        " feat ",
        " ft. ",
        " ft ",
        " with ",
        " x ",
    ] {
        normalized = normalized.replace(marker, "|");
    }

    for separator in ['&', ',', '/', ';', '+', '、', '，'] {
        normalized = normalized.replace(separator, "|");
    }

    normalized
        .split('|')
        .map(normalize_text)
        .filter(|part| !part.is_empty())
        .collect()
}

fn latin_title_has_metadata_conflict(track_title: &str, candidate_artists: &[&str]) -> bool {
    is_latin_script_text(track_title)
        && candidate_artists
            .iter()
            .any(|artist| !normalize_text(artist).is_empty())
}

fn english_metadata_can_match_chinese_candidate(
    track: &Track,
    candidate: &RemoteLyricsCandidate<'_>,
) -> bool {
    track_has_latin_metadata(track) && candidate_has_chinese_metadata(candidate)
}

fn track_has_latin_metadata(track: &Track) -> bool {
    [&track.name, &track.artist, &track.album]
        .iter()
        .any(|value| is_latin_script_text(value))
}

fn candidate_has_chinese_metadata(candidate: &RemoteLyricsCandidate<'_>) -> bool {
    let mut has_han = false;

    for value in candidate
        .track_name
        .into_iter()
        .chain(candidate.artist_names.iter().copied())
        .chain(candidate.album_name)
    {
        for ch in value.chars() {
            if is_kana_or_hangul(ch) {
                return false;
            }
            if is_han(ch) {
                has_han = true;
            }
        }
    }

    has_han
}

fn is_latin_script_text(value: &str) -> bool {
    value.chars().any(|c| c.is_ascii_alphabetic())
        && !value
            .chars()
            .any(|c| c.is_alphabetic() && !c.is_ascii_alphabetic())
}

fn is_han(ch: char) -> bool {
    ('\u{3400}'..='\u{4dbf}').contains(&ch)
        || ('\u{4e00}'..='\u{9fff}').contains(&ch)
        || ('\u{f900}'..='\u{faff}').contains(&ch)
}

fn is_kana_or_hangul(ch: char) -> bool {
    ('\u{3040}'..='\u{309f}').contains(&ch)
        || ('\u{30a0}'..='\u{30ff}').contains(&ch)
        || ('\u{ff66}'..='\u{ff9f}').contains(&ch)
        || ('\u{1100}'..='\u{11ff}').contains(&ch)
        || ('\u{3130}'..='\u{318f}').contains(&ch)
        || ('\u{ac00}'..='\u{d7af}').contains(&ch)
}

fn duration_within_tolerance(left: Duration, right: Duration) -> bool {
    left.abs_diff(right) <= DURATION_TOLERANCE
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
    fn rejects_same_title_artist_when_album_and_duration_disagree() {
        let artist_names = ["Same Artist"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("Same Song"),
            artist_names: &artist_names,
            album_name: Some("Live Album"),
            duration: Some(Duration::from_secs(260)),
        };

        assert_eq!(remote_lyrics_match_score(&track(), &candidate), None);
    }

    #[test]
    fn scores_exact_title_artist_and_matching_duration() {
        let artist_names = ["Same Artist"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("Same Song"),
            artist_names: &artist_names,
            album_name: Some("Other Album"),
            duration: Some(Duration::from_secs(242)),
        };

        assert!(remote_lyrics_match_score(&track(), &candidate).is_some());
    }

    #[test]
    fn matches_split_artist_lists_against_joined_track_artist() {
        let target = Track {
            artist: "Same Artist & Guest".into(),
            ..track()
        };
        let artist_names = ["Same Artist", "Guest"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("Same Song"),
            artist_names: &artist_names,
            album_name: Some("Studio Album"),
            duration: Some(Duration::from_secs(240)),
        };

        assert!(remote_lyrics_match_score(&target, &candidate).is_some());
    }

    #[test]
    fn accepts_localized_artist_name_when_album_and_duration_are_exact() {
        let target = Track {
            name: "LIGHT IT UP!".into(),
            artist: "YUZUHA".into(),
            album: "Light It Up!".into(),
            duration: Duration::from_millis(261_275),
            position: Duration::ZERO,
        };
        let artist_names = ["柚子花"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("Light It Up!"),
            artist_names: &artist_names,
            album_name: Some("Light It Up!"),
            duration: Some(Duration::from_millis(261_275)),
        };

        assert!(remote_lyrics_match_score(&target, &candidate).is_some());
    }

    #[test]
    fn rejects_artist_mismatch_without_album_and_duration_agreement() {
        let target = Track {
            name: "LIGHT IT UP!".into(),
            artist: "YUZUHA".into(),
            album: "Light It Up!".into(),
            duration: Duration::from_millis(261_275),
            position: Duration::ZERO,
        };
        let artist_names = ["Different Artist"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("Light It Up!"),
            artist_names: &artist_names,
            album_name: Some("Different Album"),
            duration: Some(Duration::from_secs(185)),
        };

        assert_eq!(remote_lyrics_match_score(&target, &candidate), None);
    }

    #[test]
    fn rejects_same_latin_title_and_duration_when_latin_artist_disagrees() {
        let target = Track {
            name: "ONE DAY".into(),
            artist: "A-YUE CHANG".into(),
            album: "我想要的感觉".into(),
            duration: Duration::from_secs(240),
            position: Duration::ZERO,
        };
        let artist_names = ["Different Artist"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("One Day"),
            artist_names: &artist_names,
            album_name: Some("Different Album"),
            duration: Some(Duration::from_secs(240)),
        };

        assert_eq!(
            remote_lyrics_match_score_with_options(
                &target,
                &candidate,
                MatchOptions {
                    allow_chinese_duration_fallback: true,
                },
            ),
            None
        );
    }

    #[test]
    fn rejects_latin_title_duration_match_when_non_latin_artist_and_album_disagree() {
        let target = Track {
            name: "ONE DAY".into(),
            artist: "A-YUE CHANG".into(),
            album: "我想要的感觉".into(),
            duration: Duration::from_secs(240),
            position: Duration::ZERO,
        };
        let artist_names = ["小野リサ"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("One Day"),
            artist_names: &artist_names,
            album_name: Some("One Day"),
            duration: Some(Duration::from_secs(240)),
        };

        assert_eq!(
            remote_lyrics_match_score_with_options(
                &target,
                &candidate,
                MatchOptions {
                    allow_chinese_duration_fallback: true,
                },
            ),
            None
        );
    }

    #[test]
    fn default_options_accept_latin_title_duration_match_with_artist_mismatch() {
        // LRCLIB regression gate: with default options the latin-title decoy
        // guard must NOT fire — behavior identical to git HEAD (Some(250):
        // 100 base + 100 duration + 50 title).
        let target = Track {
            name: "ONE DAY".into(),
            artist: "A-YUE CHANG".into(),
            album: "我想要的感觉".into(),
            duration: Duration::from_secs(240),
            position: Duration::ZERO,
        };

        let latin_artist_names = ["Different Artist"];
        let latin_candidate = RemoteLyricsCandidate {
            track_name: Some("One Day"),
            artist_names: &latin_artist_names,
            album_name: Some("Different Album"),
            duration: Some(Duration::from_secs(240)),
        };
        assert_eq!(
            remote_lyrics_match_score(&target, &latin_candidate),
            Some(250)
        );

        let non_latin_artist_names = ["小野リサ"];
        let non_latin_candidate = RemoteLyricsCandidate {
            track_name: Some("One Day"),
            artist_names: &non_latin_artist_names,
            album_name: Some("One Day"),
            duration: Some(Duration::from_secs(240)),
        };
        assert_eq!(
            remote_lyrics_match_score(&target, &non_latin_candidate),
            Some(250)
        );
    }

    #[test]
    fn accepts_english_apple_metadata_against_chinese_netease_metadata_by_duration() {
        let target = Track {
            name: "One Day".into(),
            artist: "A-YUE CHANG".into(),
            album: "The Feeling I Want".into(),
            duration: Duration::from_secs(240),
            position: Duration::ZERO,
        };
        let artist_names = ["张震岳"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("有一天"),
            artist_names: &artist_names,
            album_name: Some("我想要的感觉"),
            duration: Some(Duration::from_secs(240)),
        };

        let score = remote_lyrics_match_score_with_options(
            &target,
            &candidate,
            MatchOptions {
                allow_chinese_duration_fallback: true,
            },
        )
        .expect("duration fallback should match");
        assert!(score.duration_only_fallback);
    }

    #[test]
    fn accepts_localized_artist_name_when_title_has_catalog_suffix_and_duration_matches() {
        let target = Track {
            name: "小情歌".into(),
            artist: "SODAGREEN".into(),
            album: "小宇宙".into(),
            duration: Duration::from_millis(276_626),
            position: Duration::ZERO,
        };
        let artist_names = ["苏打绿"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("小情歌 (苏打绿版)"),
            artist_names: &artist_names,
            album_name: Some("小宇宙 (苏打绿版)"),
            duration: Some(Duration::from_millis(276_626)),
        };

        assert!(remote_lyrics_match_score(&target, &candidate).is_some());
    }

    #[test]
    fn accepts_localized_artist_name_when_duration_matches_even_if_album_is_translated() {
        let target = Track {
            name: "小情歌".into(),
            artist: "SODAGREEN".into(),
            album: "Little Universe".into(),
            duration: Duration::from_millis(276_626),
            position: Duration::ZERO,
        };
        let artist_names = ["苏打绿"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("小情歌 (苏打绿版)"),
            artist_names: &artist_names,
            album_name: Some("小宇宙 (苏打绿版)"),
            duration: Some(Duration::from_millis(276_626)),
        };

        assert!(remote_lyrics_match_score(&target, &candidate).is_some());
    }

    #[test]
    fn rejects_suffix_title_match_with_localized_artist_when_duration_disagrees() {
        let target = Track {
            name: "小情歌".into(),
            artist: "SODAGREEN".into(),
            album: "小宇宙".into(),
            duration: Duration::from_millis(276_626),
            position: Duration::ZERO,
        };
        let artist_names = ["苏打绿"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("小情歌 (Live)"),
            artist_names: &artist_names,
            album_name: None,
            duration: Some(Duration::from_millis(258_396)),
        };

        assert_eq!(remote_lyrics_match_score(&target, &candidate), None);
    }

    #[test]
    fn accepts_traditional_title_with_localized_artist_when_duration_matches() {
        let target = Track {
            name: "愛".into(),
            artist: "KAREN MOK".into(),
            album: "[i]".into(),
            duration: Duration::from_millis(198_333),
            position: Duration::ZERO,
        };
        let artist_names = ["莫文蔚"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("爱"),
            artist_names: &artist_names,
            album_name: Some("[i]"),
            duration: Some(Duration::from_millis(198_333)),
        };

        assert!(remote_lyrics_match_score(&target, &candidate).is_some());
    }

    #[test]
    fn accepts_album_with_trailing_release_type_when_duration_matches() {
        let target = Track {
            name: "Roll-Cigg".into(),
            artist: "Amazing Show".into(),
            album: "Sound Check - EP".into(),
            duration: Duration::from_millis(232_705),
            position: Duration::ZERO,
        };
        let artist_names = ["美秀集团"];
        let candidate = RemoteLyricsCandidate {
            track_name: Some("卷烟"),
            artist_names: &artist_names,
            album_name: Some("Sound Check"),
            duration: Some(Duration::from_millis(232_705)),
        };

        assert!(remote_lyrics_match_score(&target, &candidate).is_some());
    }
}
