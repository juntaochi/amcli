use crate::player::Track;
use std::time::Duration;

const DURATION_TOLERANCE: Duration = Duration::from_secs(3);

pub(crate) struct RemoteLyricsCandidate<'a> {
    pub track_name: Option<&'a str>,
    pub artist_names: &'a [&'a str],
    pub album_name: Option<&'a str>,
    pub duration: Option<Duration>,
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
    let candidate_title = candidate.track_name?;
    if !normalized_eq(&track.name, candidate_title) {
        return None;
    }

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

    if !(artist_matches || album_matches && duration_matches) {
        return None;
    }

    if artist_matches && has_disambiguator && !album_matches && !duration_matches {
        return None;
    }

    let mut score = 100;
    if artist_matches {
        score += 25;
    }
    if album_matches {
        score += 25;
    }
    if duration_matches {
        score += 25;
    }
    if !has_disambiguator {
        score -= 20;
    }

    Some(score)
}

fn normalized_eq(left: &str, right: &str) -> bool {
    let left = normalize_text(left);
    !left.is_empty() && left == normalize_text(right)
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_alphanumeric())
        .collect()
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
}
