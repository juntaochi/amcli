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

#[test]
fn selects_netease_song_with_localized_artist_and_catalog_suffix() {
    let target = Track {
        name: "小情歌".into(),
        artist: "SODAGREEN".into(),
        album: "小宇宙".into(),
        duration: Duration::from_millis(276_626),
        position: Duration::ZERO,
    };
    let json = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1952285469i64,
                    "name": "小情歌 (苏打绿版)",
                    "ar": [{"name": "苏打绿"}],
                    "al": {"name": "小宇宙 (苏打绿版)"},
                    "dt": 276626
                },
                {
                    "id": 2077816523i64,
                    "name": "小情歌 (Live)",
                    "ar": [{"name": "苏打绿"}],
                    "al": {"name": ""},
                    "dt": 258396
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_song_id(&json, &target),
        Some(1952285469)
    );
}

#[test]
fn builds_album_artist_search_before_fallbacks() {
    let target = Track {
        name: "小情歌".into(),
        artist: "SODAGREEN".into(),
        album: "小宇宙".into(),
        duration: Duration::from_millis(276_626),
        position: Duration::ZERO,
    };

    assert_eq!(
        NeteaseProvider::search_queries(&target),
        vec![
            "小情歌 小宇宙 SODAGREEN".to_string(),
            "小情歌 SODAGREEN".to_string(),
            "小情歌".to_string()
        ]
    );
}

#[test]
fn prefers_earlier_netease_result_when_candidate_scores_tie() {
    let json = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1,
                    "name": "Same Song",
                    "ar": [{"name": "Same Artist"}],
                    "al": {"name": "Studio Album"},
                    "dt": 240000
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

    assert_eq!(NeteaseProvider::select_song_id(&json, &track()), Some(1));
}

#[test]
fn selects_match_from_later_title_only_search_results() {
    let target = Track {
        name: "红豆".into(),
        artist: "Faye Wong".into(),
        album: "唱游".into(),
        duration: Duration::from_millis(256_026),
        position: Duration::ZERO,
    };
    let title_artist_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1,
                    "name": "红豆",
                    "ar": [{"name": "Cover Artist"}],
                    "al": {"name": "红豆"},
                    "dt": 204179
                }
            ]
        }
    });
    let title_only_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 299936i64,
                    "name": "红豆",
                    "ar": [{"name": "王菲"}],
                    "al": {"name": "唱游"},
                    "dt": 256026
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_song_id_from_results(
            &[title_artist_results, title_only_results],
            &target
        ),
        Some(299936)
    );
}

#[test]
fn selects_netease_song_with_traditional_title_and_english_artist() {
    let target = Track {
        name: "愛".into(),
        artist: "KAREN MOK".into(),
        album: "[i]".into(),
        duration: Duration::from_millis(198_333),
        position: Duration::ZERO,
    };
    let json = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 277302i64,
                    "name": "爱",
                    "ar": [{"name": "莫文蔚"}],
                    "al": {"name": "[i]"},
                    "dt": 198333
                },
                {
                    "id": 863489597i64,
                    "name": "愛",
                    "ar": [{"name": "Xero"}],
                    "al": {"name": "愛"},
                    "dt": 194168
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_song_id(&json, &target),
        Some(277302)
    );
}
