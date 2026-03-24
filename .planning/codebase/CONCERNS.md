# Codebase Concerns

**Analysis Date:** 2026-03-24

## Tech Debt

**Monolithic `draw()` function (455+ lines):**
- Issue: The `draw()` function in `src/ui/mod.rs` (lines 617-1071) handles all rendering logic in a single function: chassis, screen, artwork area, metadata, lyrics, progress bar, controls, and settings overlay. This makes modification risky and comprehension difficult.
- Files: `src/ui/mod.rs`
- Impact: Any layout change risks breaking adjacent rendering. Difficult to test individual UI sections in isolation.
- Fix approach: Extract rendering into sub-functions: `draw_chassis()`, `draw_metadata()`, `draw_artwork()`, `draw_progress()`, `draw_controls()`. Each takes `Frame`, area `Rect`, and relevant `App` state. The `draw()` function becomes a layout orchestrator only.

**Monolithic `src/ui/mod.rs` (1212 lines):**
- Issue: This single file contains the `App` struct (all application state), `Theme` definitions, `MetadataCache`, the `draw()` function, `draw_lyrics()`, helper functions, and all tests. It serves as both state manager and rendering engine.
- Files: `src/ui/mod.rs`
- Impact: Merge conflicts when multiple changes touch UI. Hard to locate specific functionality. The `App` struct mixes domain state (track, volume) with rendering state (artwork_protocol, throbber_state, animation_frame).
- Fix approach: Split into `src/ui/app.rs` (App struct and methods), `src/ui/theme.rs` (Theme definitions and constants), `src/ui/render.rs` (draw functions), and keep `src/ui/mod.rs` as the module root with re-exports.

**Settings menu uses hard-coded positional indices:**
- Issue: `update_language()` uses `get_mut(0)`, `update_theme()` uses `get(1)`/`get_mut(1)`, `update_album()` uses `get_mut(2)`, `update_mosaic()` uses `get_mut(3)`. Adding or reordering menu items will silently break updates.
- Files: `src/ui/settings.rs` (lines 126-153)
- Impact: Silently incorrect behavior if items are reordered. No compile-time protection.
- Fix approach: Replace positional indexing with iteration over `self.items` using pattern matching to find and update the correct variant, similar to the existing `should_skip_current_item()` pattern on line 106.

**Excessive `#[allow(dead_code)]` annotations (20 instances):**
- Issue: Twenty `#[allow(dead_code)]` annotations scattered across the codebase indicate either premature API surface or vestigial code from previous iterations.
- Files: `src/player/mod.rs` (7 instances), `src/ui/mod.rs` (5 instances), `src/ui/settings.rs` (2 instances), `src/artwork/cache.rs` (2 instances), `src/artwork/mod.rs` (1 instance), `src/config/mod.rs` (1 instance), `src/lyrics/provider.rs` (1 instance)
- Impact: Unclear which code is actively maintained vs. abandoned. Dead code increases cognitive load and may mask real unused-code warnings.
- Fix approach: Audit each annotation. Remove truly dead code. For intentional public API surface (trait methods), document the rationale instead of suppressing the warning.

**`config` crate dependency unused:**
- Issue: `Cargo.toml` lists `config = "0.13"` as a dependency, but the actual config loading in `src/config/mod.rs` uses `toml` + `serde` directly. The `config` crate is never imported.
- Files: `Cargo.toml` (line 36), `src/config/mod.rs`
- Impact: Unnecessary compile-time cost and binary size increase.
- Fix approach: Remove `config = "0.13"` from `Cargo.toml`.

**Unused dependencies (`chrono`, `unicode-width`, `tui-big-text`, `futures`, `rgb`):**
- Issue: Several dependencies in `Cargo.toml` have no corresponding `use` statements in any source file: `chrono`, `unicode-width`, `tui-big-text`, `futures`, `rgb`. These appear to be leftovers from earlier development.
- Files: `Cargo.toml` (lines 23, 50-51, 58)
- Impact: Increased compile time, larger dependency tree, unnecessary supply-chain surface.
- Fix approach: Run `cargo udeps` or manually verify each, then remove unused entries from `Cargo.toml`.

**Hardcoded User-Agent version string:**
- Issue: `LrclibProvider` uses `"AMCLI v1.0.0"` as the User-Agent, but the actual crate version is `0.2.0`.
- Files: `src/lyrics/lrclib.rs` (lines 30-33)
- Impact: Misrepresents the client version to the LRCLIB API. Will drift further as the project versions.
- Fix approach: Use `env!("CARGO_PKG_VERSION")` to generate the version string at compile time via `format!("AMCLI v{}", env!("CARGO_PKG_VERSION"))` or a `const` constructed with `concat!`.

## Security Considerations

**`.gitignore` patterns could shadow source directories:**
- Risk: The `.gitignore` contains `/artwork/` (line 53) and `/lyrics/` (line 54). These are prefixed with `/` so they only match at the repo root, which is correct. However, if the patterns were ever changed to `artwork/` or `lyrics/` (without leading slash), they would exclude `src/artwork/` and `src/lyrics/` from git, silently losing source code.
- Files: `.gitignore` (lines 53-54)
- Current mitigation: Leading `/` prefix limits matching to repo root.
- Recommendations: Add a comment in `.gitignore` explaining the leading slash is intentional: `# Leading / is critical -- without it, src/artwork/ and src/lyrics/ would be excluded`.

**No input sanitization for AppleScript injection:**
- Risk: Track metadata (name, artist, album) from Apple Music is interpolated into AppleScript strings via `format!()` in `get_artwork_url()`. While this data originates from the local Music app (not user input), malformed metadata with AppleScript control characters could cause unexpected behavior.
- Files: `src/player/apple_music.rs` (line 264 -- `track_key` uses raw metadata for cache key, and line 274 for URL construction)
- Current mitigation: The `urlencoding::encode()` on line 277 sanitizes the HTTP query, but the cache key on line 264 is unsanitized.
- Recommendations: Low risk since data comes from local Music app, but document the trust boundary.

## Performance Bottlenecks

**Artwork URL fetch blocks the update loop:**
- Problem: `App::update()` calls `self.player.get_artwork_url(track).await` (line 412) synchronously within the update cycle. This makes an HTTP request to the iTunes Search API on every track change, blocking the update loop for up to 3 seconds (the timeout set on line 280).
- Files: `src/ui/mod.rs` (lines 411-424), `src/player/apple_music.rs` (lines 263-294)
- Cause: The artwork URL lookup uses `reqwest::get()` wrapped in a `tokio::time::timeout()` of 3 seconds. While async, this is awaited inline in `update()`, blocking the entire update cycle.
- Improvement path: Spawn the artwork URL fetch as a background task (similar to how `artwork_task` and `lyrics_task` are handled). Return a `JoinHandle` and poll for completion in subsequent `update()` calls.

**`ArtworkManager::get_artwork_themed_v2` does HTTP fetch without timeout:**
- Problem: `reqwest::get(url).await?` on line 39 of `src/artwork/mod.rs` has no timeout, unlike the iTunes API call which uses `tokio::time::timeout()`. A slow CDN could block the artwork processing task indefinitely.
- Files: `src/artwork/mod.rs` (line 39)
- Cause: Missing timeout wrapper around the image download.
- Improvement path: Wrap with `tokio::time::timeout(Duration::from_secs(10), reqwest::get(url))` or use a `reqwest::Client` with a configured timeout instead of the convenience `reqwest::get()`.

**DynamicImage cloning in artwork cache:**
- Problem: `ArtworkCache::get()` calls `.cloned()` on `DynamicImage` (line 35 of `src/artwork/cache.rs`), which deep-copies the entire image buffer. For 600x600 RGBA images, this is ~1.4MB per clone.
- Files: `src/artwork/cache.rs` (line 35), `src/artwork/mod.rs` (line 56)
- Cause: `LruCache::get()` returns a reference, but the image must outlive the lock.
- Improvement path: Wrap cached images in `Arc<DynamicImage>` so `get()` returns a cheap `Arc::clone()` instead of a full data copy.

**`std::sync::Mutex` in async context:**
- Problem: `AppleMusicController` and `LyricsManager` use `std::sync::Mutex` for their LRU caches. While the locks are not held across `.await` points (which would cause deadlocks), `std::sync::Mutex` blocks the Tokio runtime thread while waiting to acquire the lock.
- Files: `src/player/apple_music.rs` (line 36), `src/lyrics/mod.rs` (line 53), `src/artwork/cache.rs` (line 10)
- Cause: `std::sync::Mutex` was chosen (likely for simplicity or because `LruCache` is not `Send`).
- Improvement path: Since these locks are held only for brief in-memory operations (cache get/put), the current usage is acceptable for a single-user TUI. If contention ever becomes an issue, migrate to `tokio::sync::Mutex`. Document the design decision.

## Fragile Areas

**Lyrics caching caches `None` permanently:**
- Files: `src/lyrics/mod.rs` (lines 116-118)
- Why fragile: When all lyrics providers fail (network timeout, API error), `None` is cached permanently for that track key. If the user's network recovers or a provider comes back online, the lyrics remain missing until the cache evicts the entry or the app is restarted.
- Safe modification: Add a TTL to the `None` cache entries, or use a separate `HashSet<String>` of "recently failed" keys with a timestamp, evicting after 5 minutes.
- Test coverage: No test covers the negative-caching behavior.

**AppleScript output parsing with `:::BOLT_SPLIT:::` delimiter:**
- Files: `src/player/apple_music.rs` (lines 165-214)
- Why fragile: The combined status call uses `:::BOLT_SPLIT:::` as a field delimiter. If any track metadata (name, artist, album) contains this exact string, parsing silently produces garbled data. The parser expects exactly 7 parts (`parts.len() >= 7`) and uses index-based access (`parts[2]` through `parts[6]`).
- Safe modification: Validate field count strictly and use a delimiter that is less likely to appear in metadata (or escape metadata fields in the AppleScript).
- Test coverage: The mock-based test in `src/player/apple_music.rs` does not test `get_player_status()` at all.

**`unwrap_or(0.0)` silently ignores parse failures:**
- Files: `src/player/apple_music.rs` (lines 202-203)
- Why fragile: Duration and position parsing in `get_player_status()` silently default to `0.0` on parse error. This causes the progress bar to jump to 0% and then recover on the next poll, creating a visible glitch.
- Safe modification: Return an error or log a warning when parsing fails, and skip the track update for that cycle.
- Test coverage: No test exercises the parse-error path.

**Mouse event handler is a no-op placeholder:**
- Files: `src/main.rs` (lines 132-135)
- Why fragile: Mouse capture is enabled (`EnableMouseCapture` on line 50) but `Event::Mouse` is silently consumed. This means mouse events generate overhead (raw mode processing) with no benefit, and users may be confused that mouse clicks do nothing.
- Safe modification: Either implement mouse handling or remove `EnableMouseCapture`/`DisableMouseCapture` from the terminal setup.
- Test coverage: None.

**Navigation methods are empty stubs:**
- Files: `src/ui/mod.rs` (lines 277-280)
- Why fragile: `navigate_up()`, `navigate_down()`, `navigate_left()`, `navigate_right()` are bound to keyboard shortcuts (j/k/h/l and arrow keys in `src/main.rs` lines 121-124) but do nothing. Users pressing these keys see no response.
- Safe modification: Either implement navigation (e.g., for a track list or queue) or remove the keybindings and the stub methods.
- Test coverage: None.

## Test Coverage Gaps

**No integration tests against real Apple Music:**
- What's not tested: All `AppleMusicController` tests use `MockCommandRunner`. There are zero tests that execute actual `osascript` commands or verify that the AppleScript syntax is correct for the current macOS version.
- Files: `src/player/apple_music.rs` (tests at lines 297-355)
- Risk: AppleScript syntax changes across macOS versions (e.g., Music app was iTunes until Catalina). A macOS update could break all player commands silently.
- Priority: Medium -- CI already runs on `macos-latest`, so an optional integration test gated behind a feature flag or env var would catch regressions.

**No tests for `get_player_status()`:**
- What's not tested: The optimized combined-status AppleScript (`get_player_status()`) that is called every 500ms in the main loop has no test coverage.
- Files: `src/player/apple_music.rs` (lines 164-214)
- Risk: The `:::BOLT_SPLIT:::` parsing and 7-field extraction could silently break. This is the single hottest code path in the application.
- Priority: High -- add a mock test that verifies correct parsing of the combined output format.

**No tests for lyrics providers (Netease, LRCLIB, Local):**
- What's not tested: None of the three lyrics provider implementations have any unit tests. Only the LRC parser (`src/lyrics/parser.rs`) is tested.
- Files: `src/lyrics/lrclib.rs`, `src/lyrics/netease.rs`, `src/lyrics/local.rs`
- Risk: API response format changes from LRCLIB or Netease would go undetected. Local file pattern matching (`Artist - Track.lrc` vs `Track - Artist.lrc`) is untested.
- Priority: Medium -- at minimum, test the response parsing with fixture JSON/LRC data.

**No tests for `LyricsManager` orchestration:**
- What's not tested: Provider priority ordering, timeout behavior (the `tokio::time::timeout` of 5s on line 92), cache hit/miss paths, and the negative-caching of `None` results.
- Files: `src/lyrics/mod.rs` (lines 70-121)
- Risk: Changes to provider priority or timeout handling could break lyrics fetching without any test failure.
- Priority: Medium.

**No tests for artwork pipeline:**
- What's not tested: `ArtworkManager::get_artwork_themed_v2()`, `ArtworkCache` (memory and disk caching), `ArtworkConverter`, and the duotone/pixelation image processing.
- Files: `src/artwork/mod.rs`, `src/artwork/cache.rs`, `src/artwork/converter.rs`
- Risk: Image processing regressions (wrong colors, corrupted output) would be invisible.
- Priority: Low -- visual output is hard to test, but cache behavior and converter initialization are testable.

**No CI for TUI rendering:**
- What's not tested: CI runs `cargo test` which exercises the `TestBackend` rendering test, but there is no visual regression testing. The one rendering test (`test_ui_rendering` at `src/ui/mod.rs` line 1196) only checks that the buffer contains "TEST", "SONG", and "ARTIST" strings.
- Files: `.github/workflows/ci.yml`, `src/ui/mod.rs` (lines 1195-1211)
- Risk: Layout regressions (overlapping widgets, truncated text, broken theme rendering) go undetected.
- Priority: Low -- consider snapshot testing with `ratatui::backend::TestBackend` buffer assertions for each theme.

**Config loading/saving untested:**
- What's not tested: `Config::load()`, `Config::save()`, TOML serialization round-trip, default config generation, and the `get_config_path()` directory creation logic.
- Files: `src/config/mod.rs` (lines 99-132)
- Risk: A serde attribute change or TOML format issue could make existing user configs unloadable.
- Priority: Medium -- at minimum, test that `Config::default()` round-trips through `toml::to_string_pretty()` and `toml::from_str()`.

## Scaling Limits

**LRU cache sizes are small and not configurable per-cache:**
- Current capacity: Artwork URL cache: 20 entries (`src/player/apple_music.rs` line 44), Lyrics cache: 20 entries (`src/ui/mod.rs` line 178), Artwork image cache: 100 entries (`src/artwork/mod.rs` line 17, configurable via `config.artwork.cache_size`).
- Limit: Users with large playlists on shuffle will see frequent cache misses, triggering repeated network requests.
- Scaling path: Make all cache sizes configurable in `config.toml`, or implement adaptive sizing. The artwork URL cache of 20 is particularly low given that artwork URLs are fetched on every track change.

## Dependencies at Risk

**`reqwest 0.11` is behind current:**
- Risk: `reqwest 0.11` is a major version behind (current is 0.12+). Version 0.12 changed the `rustls` integration and some API surface.
- Impact: Pinning to 0.11 blocks updates to other crates that depend on newer `reqwest` or `hyper` versions.
- Migration plan: Update to `reqwest = { version = "0.12", ... }` and verify that the `rustls-tls` feature still works.

**`lazy_static` vs `std::sync::LazyLock`:**
- Risk: `lazy_static` (used in `src/lyrics/parser.rs`) is effectively superseded by `std::sync::LazyLock` stabilized in Rust 1.80. It is a maintenance-mode crate.
- Impact: No functional risk, but adds an unnecessary dependency.
- Migration plan: Replace `lazy_static!` block with `static TIME_REGEX: LazyLock<Regex> = LazyLock::new(|| ...)` and remove `lazy_static` from `Cargo.toml`.

## Missing Critical Features

**No graceful degradation when Apple Music is not running:**
- Problem: If Apple Music is not running, `osascript` commands trigger macOS to launch it. There is no check for whether the app is running before issuing commands.
- Blocks: Users who accidentally launch `amcli` without Music running get an unexpected Music app launch.
- Files: `src/player/apple_music.rs`

**No shuffle state display or persistence:**
- Problem: The `toggle_shuffle()` method exists (line 283 of `src/ui/mod.rs`) but is marked `#[allow(dead_code)]` and is not bound to any key. The UI does not display current shuffle state.
- Blocks: Users cannot control or see shuffle mode from the TUI.
- Files: `src/ui/mod.rs` (line 282-285)

---

*Concerns audit: 2026-03-24*
