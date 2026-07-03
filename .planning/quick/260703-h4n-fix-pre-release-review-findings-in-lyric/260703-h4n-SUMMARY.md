---
phase: quick
plan: 260703-h4n
subsystem: lyrics, ui
tags: [lyrics-matching, netease, lrclib, marquee, clippy, cjk]
requires: []
provides:
  - "Trust-gated, tiered Chinese duration fallback for Netease lyrics matching"
  - "LRCLIB matching behavior identical to git HEAD (regression-gated by test)"
  - "Display-cell-measured lyrics marquee with CJK test coverage"
affects: [src/lyrics, src/ui]
tech-stack:
  added: []
  patterns:
    - "Tier-first match comparison via SongMatch::tier_key() = (!fallback, score)"
    - "Semantic query trust (SearchQuery { query, trusted }) instead of positional rank gates"
key-files:
  created: []
  modified:
    - src/lyrics/matching.rs
    - src/lyrics/netease.rs
    - src/lyrics/netease_tests.rs
    - src/lyrics/lrclib.rs
    - src/lyrics/mod.rs
    - src/ui/mod.rs
decisions:
  - "Finding 1 fixed with #[allow(clippy::too_many_arguments)] per D-05 (codebase convention over param struct)"
  - "Finding 2 fixed structurally (tier tuple), not by score capping — Netease bonuses (+320) make a scalar cap impossible"
  - "Finding 4 guard gated on options.allow_chinese_duration_fallback so LRCLIB default path scores 250 exactly as git HEAD"
metrics:
  duration: "8min"
  tasks: 3
  files: 6
  completed: "2026-07-03T05:59:19Z"
---

# Quick Task 260703-h4n: Fix Pre-Release Review Findings in Lyrics Summary

Trust-gated Netease Chinese duration fallback behind semantic query trust and real-match tiering, restored the lost matching.rs from the rustdoc recovery artifact, and covered the display-cell marquee with CJK unit tests — all 5 review findings closed, `make verify` green.

## Findings Addressed

| Finding | Fix | Where |
|---------|-----|-------|
| F1 (clippy 8-arg) | `#[allow(clippy::too_many_arguments)]` + rationale comment on `draw_lyrics` | src/ui/mod.rs |
| F2 (fallback masks real match) | `RemoteLyricsScore { score, duration_only_fallback }`; all comparisons tier-first via `SongMatch::tier_key()` — non-fallback always outranks fallback | src/lyrics/matching.rs, netease.rs |
| F3 (positional trust gate defeated by dedup) | `SearchQuery { query, trusted }` — trusted iff built with non-empty artist; bare-title never trusted; `allows_chinese_duration_fallback_for_query` deleted | src/lyrics/netease.rs |
| F4 (guard hits LRCLIB) | Latin-title conflict guard now conditional on `options.allow_chinese_duration_fallback`; default path scores `Some(250)` = git HEAD | src/lyrics/matching.rs |
| F5 (marquee char-count width) | Code fix pre-existed (UnicodeWidth*); added 4 unit tests: borrowed fit, ASCII frame-0 window, CJK overflow-by-cells, CJK window width bounds | src/ui/mod.rs |

## Commits

- `ee41245` fix(lyrics): guard chinese duration fallback behind trusted queries and real-match tiering (findings 2/3/4; includes the pending feature work per D-04)
- `0e449ad` fix(ui): measure lyrics marquee in display cells and allow 8-arg draw_lyrics (findings 1/5; includes the pending marquee feature work per D-04)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Working tree drifted past the plan's Task 1 baseline**
- **Found during:** Task 1
- **Issue:** The plan expected the restored `matching.rs` + working-tree `netease.rs` to compile and pass tests (they were byte-consistent at planning time). Since planning, `netease.rs`/`netease_tests.rs` had been further modified with the Finding 2/3 fixes already applied on their side (trusted `SearchQuery`, an unused `SongMatch` struct, an import of the not-yet-existing `RemoteLyricsScore`), so Task 1's "restore then green" state was unreachable — `cargo check` failed on `E0432: no RemoteLyricsScore in lyrics::matching`.
- **Fix:** Restored the recovered artifact (sha1 verified `5552ec8...`), then completed the matching.rs side of the contract netease.rs already coded against (Findings 2 and 4) instead of running tests on a non-compiling intermediate state. Task 1 and the matching.rs portion of Task 2 effectively merged; all netease-side Task 2 tests already existed in the tree and pass unchanged.
- **Files modified:** src/lyrics/matching.rs
- **Commit:** ee41245

No other deviations — findings, tests, commit structure, and gate all executed as planned.

## Verification

- `cargo test --all-features`: 74 passed, 0 failed (70 pre-existing/restored + 4 new scroll_text tests)
- `cargo clippy --all-features -- -D warnings`: clean
- `make verify`: exits 0 (fmt check, clippy, test, build, doc)
- `git status src/`: clean; no file deletions in either commit
- LRCLIB regression gate: `default_options_accept_latin_title_duration_match_with_artist_mismatch` asserts `Some(250)` on the default path; lrclib.rs still calls plain `remote_lyrics_match_score`
- Feature intent (D-01): `selects_chinese_metadata_match_over_japanese_same_title_decoy` still selects the Han candidate

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary surface. T-quick-01 mitigation (trust gating + tiering) implemented as registered.

## Self-Check: PASSED

- src/lyrics/matching.rs contains `allow_chinese_duration_fallback`, >500 lines: FOUND
- src/lyrics/netease.rs contains `trusted`: FOUND
- src/ui/mod.rs contains `allow(clippy::too_many_arguments)`: FOUND
- Commit ee41245: FOUND
- Commit 0e449ad: FOUND
