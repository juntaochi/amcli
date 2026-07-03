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

fn localized_title_track() -> Track {
    Track {
        name: "Not Good Enough For You".into(),
        artist: "Jay Chou".into(),
        album: "Jay Chou On The Run".into(),
        duration: Duration::from_millis(288_760),
        position: Duration::ZERO,
    }
}

fn english_metadata_chinese_song_track() -> Track {
    Track {
        name: "One Day".into(),
        artist: "A-YUE CHANG".into(),
        album: "The Feeling I Want".into(),
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
            SearchQuery {
                query: "小情歌 小宇宙 SODAGREEN".to_string(),
                trusted: true,
            },
            SearchQuery {
                query: "小情歌 SODAGREEN".to_string(),
                trusted: true,
            },
            SearchQuery {
                query: "小情歌".to_string(),
                trusted: false,
            },
        ]
    );
}

#[test]
fn search_queries_keep_bare_title_untrusted_when_dedup_shifts_ranks() {
    // The old positional gate (query_rank < 2) wrongly trusted the bare-title
    // query whenever dedup shortened the list. Trust must be semantic.
    let expected = vec![
        SearchQuery {
            query: "Same Song Same Artist".to_string(),
            trusted: true,
        },
        SearchQuery {
            query: "Same Song".to_string(),
            trusted: false,
        },
    ];

    let empty_album = Track {
        album: "".into(),
        ..track()
    };
    assert_eq!(NeteaseProvider::search_queries(&empty_album), expected);

    let self_titled = Track {
        album: "Same Song".into(),
        ..track()
    };
    assert_eq!(NeteaseProvider::search_queries(&self_titled), expected);
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
fn prefers_clean_query_over_earlier_same_title_decoy_with_close_duration() {
    let target = Track {
        name: "Love me now (feat. zoe wees)".into(),
        artist: "Kygo".into(),
        album: "Thrill Of The Chase".into(),
        duration: Duration::from_millis(196_100),
        position: Duration::ZERO,
    };
    // Album-bearing query: real track absent, only a same-title song by a
    // different artist whose duration coincidentally lands within tolerance.
    let album_query_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1295064974i64,
                    "name": "Love Me Now (feat. Se.A)",
                    "ar": [{"name": "FIXL"}, {"name": "Rothchild"}, {"name": "Se.A"}],
                    "al": {"name": "Love Me Now"},
                    "dt": 198700
                }
            ]
        }
    });
    // Cleaner name+artist query holds the correct track.
    let artist_query_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1868378791i64,
                    "name": "Love Me Now",
                    "ar": [{"name": "Kygo"}, {"name": "Zoe Wees"}],
                    "al": {"name": "Love Me Now"},
                    "dt": 196100
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_song_id_from_results(
            &[album_query_results, artist_query_results],
            &target
        ),
        Some(1868378791)
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

#[test]
fn selects_chinese_metadata_match_over_japanese_same_title_decoy() {
    let json = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1,
                    "name": "One Day",
                    "ar": [{"name": "小野リサ"}],
                    "al": {"name": "One Day"},
                    "dt": 240000
                },
                {
                    "id": 2,
                    "name": "有一天",
                    "ar": [{"name": "张震岳"}],
                    "al": {"name": "我想要的感觉"},
                    "dt": 240000
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_song_id(&json, &english_metadata_chinese_song_track()),
        Some(2)
    );
}

#[test]
fn rejects_title_only_chinese_duration_match_for_english_song() {
    let empty_results = serde_json::json!({
        "result": {
            "songs": []
        }
    });
    let title_only_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1,
                    "name": "有一天",
                    "ar": [{"name": "张震岳"}],
                    "al": {"name": "我想要的感觉"},
                    "dt": 240000
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_song_id_from_search_results(
            &[
                (empty_results.clone(), true),
                (empty_results, true),
                (title_only_results, false)
            ],
            &english_metadata_chinese_song_track()
        ),
        None
    );
}

#[test]
fn prefers_textual_match_over_higher_scoring_duration_fallback_across_queries() {
    // Result set A (trusted): a genuine title+artist+album match whose duration
    // is 4s off (outside tolerance -> no duration bonus, raw score stays low).
    let genuine_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 10,
                    "name": "One Day",
                    "ar": [{"name": "A-YUE CHANG"}],
                    "al": {"name": "The Feeling I Want"},
                    "dt": 244000
                }
            ]
        }
    });
    // Result set B (trusted): a Han-metadata duration-only fallback candidate
    // whose rank + duration bonuses would out-score the genuine match.
    let fallback_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 20,
                    "name": "有一天",
                    "ar": [{"name": "张震岳"}],
                    "al": {"name": "我想要的感觉"},
                    "dt": 240000
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_song_id_from_search_results(
            &[(genuine_results, true), (fallback_results, true)],
            &english_metadata_chinese_song_track()
        ),
        Some(10)
    );
}

#[test]
fn rejects_untrusted_duration_fallback_regardless_of_query_rank() {
    let title_only_results = serde_json::json!({
        "result": {
            "songs": [
                {
                    "id": 1,
                    "name": "有一天",
                    "ar": [{"name": "张震岳"}],
                    "al": {"name": "我想要的感觉"},
                    "dt": 240000
                }
            ]
        }
    });

    // Duration-only Han fallback candidates in UNTRUSTED result sets must be
    // rejected even at ranks 0 and 1, where the old positional gate allowed them.
    assert_eq!(
        NeteaseProvider::select_song_id_from_search_results(
            &[
                (title_only_results.clone(), false),
                (title_only_results, false)
            ],
            &english_metadata_chinese_song_track()
        ),
        None
    );
}

#[test]
fn selects_artist_id_from_netease_alias() {
    let artists = serde_json::json!({
        "result": {
            "artists": [
                {
                    "id": 6452,
                    "name": "周杰伦",
                    "alias": ["Jay Chou", "周董"]
                }
            ]
        }
    });

    assert_eq!(
        NeteaseProvider::select_artist_id(&artists, &localized_title_track()),
        Some(6452)
    );
}

#[test]
fn selects_song_from_album_alias_by_duration_for_localized_titles() {
    let albums = serde_json::json!({
        "hotAlbums": [
            {
                "id": 18886,
                "name": "我很忙",
                "artist": {
                    "name": "周杰伦",
                    "alias": ["Jay Chou"]
                },
                "alias": ["On The Run!"],
                "size": 10
            }
        ]
    });
    let album = serde_json::json!({
        "songs": [
            {
                "id": 185807,
                "name": "牛仔很忙",
                "al": {"name": "我很忙"},
                "dt": 168000
            },
            {
                "id": 185818,
                "name": "我不配",
                "al": {"name": "我很忙"},
                "dt": 288000
            }
        ]
    });

    assert_eq!(
        NeteaseProvider::select_album_alias_song_id(&albums, &[album], &localized_title_track()),
        Some(185818)
    );
}

#[test]
fn rejects_album_song_when_duration_is_outside_tolerance() {
    let album = serde_json::json!({
        "songs": [
            {
                "id": 185818,
                "name": "我不配",
                "al": {"name": "我很忙"},
                "dt": 281000
            }
        ]
    });

    assert_eq!(
        NeteaseProvider::select_song_id_by_album_duration(&album, &localized_title_track()),
        None
    );
}

#[test]
fn alias_lookup_urls_do_not_use_song_search() {
    assert_eq!(
        NeteaseProvider::artist_search_url("Jay Chou"),
        "https://music.163.com/api/cloudsearch/pc?s=Jay%20Chou&type=100&limit=5"
    );
    assert_eq!(
        NeteaseProvider::artist_albums_url(6452),
        "https://music.163.com/api/artist/albums/6452?id=6452&offset=0&limit=50"
    );
    assert_eq!(
        NeteaseProvider::album_url(18886),
        "https://music.163.com/api/v1/album/18886"
    );
}

#[test]
fn strips_leading_colon_lines_from_netease_lyrics() {
    let lrc = "\
[00:00.00] 作词 : 方文山
[00:01.00] 统筹：某人
[00:02.00] Lyrics : Someone
[00:03.00] Publisher：Someone
[00:18.63]这街上太拥挤
[00:20.86]太多人有秘密
[00:30.00]作词：这句已经是正文";

    let lyrics = NeteaseProvider::parse_lyrics(lrc).unwrap();

    assert_eq!(lyrics.lines.len(), 3);
    assert_eq!(lyrics.lines[0].text, "这街上太拥挤");
    assert_eq!(lyrics.lines[1].text, "太多人有秘密");
    assert_eq!(lyrics.lines[2].text, "作词：这句已经是正文");
}

#[test]
fn ignores_timestamp_colons_when_stripping_netease_credits() {
    let lrc = "\
[00:18.63]这街上太拥挤
[00:20.86]太多人有秘密";

    let lyrics = NeteaseProvider::parse_lyrics(lrc).unwrap();

    assert_eq!(lyrics.lines.len(), 2);
    assert_eq!(lyrics.lines[0].text, "这街上太拥挤");
    assert_eq!(lyrics.lines[1].text, "太多人有秘密");
}
