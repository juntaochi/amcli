# Architecture

**Analysis Date:** 2026-03-24

## Pattern Overview

**Overall:** Modular monolith with async event loop

**Key Characteristics:**
- Single-binary TUI application with a synchronous 50ms poll event loop driving async operations
- Trait-based abstraction layer for player backends and lyrics providers, enabling testability via mocking
- Background `tokio::spawn` tasks for network I/O (lyrics fetch, artwork download) to avoid blocking the UI draw loop
- All application state centralized in a single `App` struct in `src/ui/mod.rs`
- AppleScript IPC via `osascript` CLI subprocess for all Apple Music communication

## Layers

**Entry Point / Event Loop:**
- Purpose: Initialize terminal, configure panic hook, run the main poll-draw-update loop
- Location: `src/main.rs`
- Contains: `main()`, `run_app()`, `restore_terminal()`, CLI arg parsing via `clap`
- Depends on: `ui` (App, draw), `crossterm`, `ratatui`, `tokio`
- Used by: Nothing (top-level binary entry)

**UI / Application State (`ui`):**
- Purpose: Central state hub, rendering logic, keyboard input delegation, async task management
- Location: `src/ui/mod.rs`, `src/ui/settings.rs`
- Contains: `App` struct (all mutable state), `draw()` function (all rendering), `Theme` definitions, `MetadataCache`, `SettingsMenu`
- Depends on: `player` (MediaPlayer trait, Track, RepeatMode), `lyrics` (LyricsManager, Lyrics, all providers), `artwork` (ArtworkManager, ArtworkConverter), `config` (Config, Language)
- Used by: `main.rs` (creates App, calls draw, dispatches key events)

**Player Abstraction (`player`):**
- Purpose: Define the media player interface and implement Apple Music control via AppleScript
- Location: `src/player/mod.rs`, `src/player/apple_music.rs`
- Contains: `MediaPlayer` trait, `Track`, `PlaybackState`, `RepeatMode`, `PlayerStatus`, `AppleMusicController`, `CommandRunner` trait
- Depends on: `tokio::process::Command`, `reqwest` (for iTunes artwork API), `lru` (artwork URL cache)
- Used by: `ui` module (via `Box<dyn MediaPlayer>`)

**Lyrics System (`lyrics`):**
- Purpose: Multi-provider lyrics fetching with LRC parsing and LRU caching
- Location: `src/lyrics/mod.rs`, `src/lyrics/provider.rs`, `src/lyrics/parser.rs`, `src/lyrics/netease.rs`, `src/lyrics/lrclib.rs`
- Contains: `LyricsManager`, `LyricsProvider` trait, `Lyrics`, `LyricLine`, `parse_lrc()`, and two online provider implementations
- Depends on: `player` (Track struct), `reqwest` (HTTP for Netease/LRCLIB)
- Used by: `ui` module (via `Arc<LyricsManager>`)

**Artwork System (`artwork`):**
- Purpose: Download, cache, theme, and convert album artwork images for terminal rendering
- Location: `src/artwork/mod.rs`, `src/artwork/cache.rs`, `src/artwork/converter.rs`
- Contains: `ArtworkManager`, `ArtworkCache` (LRU memory + disk), `ArtworkConverter` (ratatui-image protocol adapter), image processing functions (duotone, pixelation)
- Depends on: `image`, `ratatui-image`, `reqwest`, `sha2`, `lru`
- Used by: `ui` module (via `ArtworkManager` and `ArtworkConverter`)

**Configuration (`config`):**
- Purpose: Load/save TOML config, define configuration schema
- Location: `src/config/mod.rs`
- Contains: `Config`, `ArtworkConfig`, `UIConfig`, `GeneralConfig`, `Language` enum
- Depends on: `serde`, `toml`, `dirs`, `tokio::fs`
- Used by: `ui` module (reads config at startup, saves on settings changes)

## Data Flow

**Main Event Loop (in `src/main.rs` `run_app()`):**

1. `terminal.draw(|f| ui::draw(f, &mut app))` renders the current frame (synchronous, ~16ms target)
2. `event::poll(50ms)` waits up to 50ms for keyboard/mouse input
3. If input arrives, dispatch to `app` methods (playback control, navigation, settings)
4. Every 500ms (`update_interval`), call `app.update().await` to refresh state from Apple Music

**State Refresh (`App::update()` in `src/ui/mod.rs` lines 391-564):**

1. Call `player.get_player_status().await` -- single optimized AppleScript call returning state + volume + track
2. Call `player.get_artwork_url(track).await` -- hits iTunes Search API (with LRU cache in `AppleMusicController`)
3. If track changed: abort existing lyrics task, spawn new `tokio::spawn` for `lyrics_manager.get_lyrics()`
4. If artwork URL changed: abort existing artwork task, spawn new `tokio::spawn` for `artwork_manager.get_artwork_themed_v2()`
5. Poll `JoinHandle`s: if lyrics/artwork tasks finished, `.await` them and store results in `App` state
6. Update `MetadataCache` with pre-formatted strings for rendering

**AppleScript Communication (`src/player/apple_music.rs`):**

1. `AppleMusicController` holds a `Box<dyn CommandRunner>` (production: `OsascriptRunner`, test: `MockCommandRunner`)
2. `OsascriptRunner::execute()` spawns `tokio::process::Command::new("osascript").arg("-e").arg(script)`
3. `get_player_status()` is the hot-path optimization: single AppleScript fetching state + volume + track data, using `":::BOLT_SPLIT:::"` delimiter
4. Artwork URL fetched via iTunes Search API (`https://itunes.apple.com/search`) with 3-second timeout, cached in `Mutex<LruCache>` keyed by `"artist|track"`

**Lyrics Fetching (`src/lyrics/mod.rs`):**

1. `LyricsManager::get_lyrics()` checks `Arc<Mutex<LruCache>>` first
2. Before calibration, races LRCLIB and Netease concurrently and returns the first provider that yields lyrics
3. When a clear winner is observed, stores it as the session primary and tries it first on later tracks, with fallback to the other provider
4. Successful hits are cached; transient provider failures and misses are not cached as permanent misses

**Artwork Pipeline (`src/artwork/mod.rs`, `src/artwork/cache.rs`):**

1. `ArtworkManager::get_artwork_themed_v2()` checks memory cache (keyed by theme+url+mosaic+retro)
2. Downloads image via `reqwest::get(url)`
3. Applies duotone theme filter (for retro themes) or passes through (modern themes)
4. Optionally applies pixelation/mosaic effect
5. Caches processed `DynamicImage` in memory LRU
6. Back in `App::update()`, the `DynamicImage` is converted to `StatefulProtocol` via `ArtworkConverter::create_protocol()`

**State Management:**
- All mutable state lives in `App` struct fields (no external state store, no message passing)
- `App` is owned by `run_app()` and passed as `&mut` to both event handlers and the draw function
- Background tasks communicate results via `JoinHandle` polling (not channels)
- Caches use `Arc<Mutex<LruCache>>` for thread-safe shared access between spawned tasks and main loop

## Key Abstractions

**MediaPlayer Trait:**
- Purpose: Abstract over different media player backends (currently only Apple Music)
- Definition: `src/player/mod.rs` lines 39-76
- Implementation: `src/player/apple_music.rs` `AppleMusicController`
- Pattern: `#[async_trait]` with `Box<dyn MediaPlayer>` stored in `App`
- Key methods: `get_player_status()` (hot path), `toggle()`, `next()`, `previous()`, `set_volume()`, `seek()`, `get_artwork_url()`

**CommandRunner Trait:**
- Purpose: Abstract the `osascript` subprocess execution for testability
- Definition: `src/player/apple_music.rs` lines 10-14
- Implementations: `OsascriptRunner` (production), `MockCommandRunner` (test, via `mockall`)
- Pattern: `#[cfg_attr(test, automock)]` generates mock at compile time

**LyricsProvider Trait:**
- Purpose: Abstract over different lyrics data sources
- Definition: `src/lyrics/provider.rs`
- Implementations: `LrclibProvider` (`src/lyrics/lrclib.rs`), `NeteaseProvider` (`src/lyrics/netease.rs`). The local file provider was removed in v0.3.0.
- Pattern: `#[async_trait]` with `Box<dyn LyricsProvider>` stored in `Vec` inside `LyricsManager`, wrapped in `Arc` for sharing across tasks
- Priority system: `fn priority(&self) -> u8` determines query order (lower first)

**Theme System:**
- Purpose: Define color schemes for the entire UI
- Definition: `src/ui/mod.rs` lines 35-113
- Pattern: Const `Theme` structs stored in a static `THEMES` array, indexed by `current_theme_index` in `App`
- Affects: All rendering (borders, text, backgrounds), artwork processing (duotone filter colors), and whether retro visual effects apply

## Entry Points

**Binary Entry (`src/main.rs`):**
- Location: `src/main.rs` line 36
- Triggers: `cargo run` or `amcli` binary execution
- Responsibilities: Parse CLI args, set up tracing, install panic hook, initialize terminal (raw mode, alternate screen), create `App`, run event loop, restore terminal on exit

**App Construction (`App::new()`):**
- Location: `src/ui/mod.rs` line 151
- Triggers: Called once from `main()`
- Responsibilities: Load config from TOML, create `AppleMusicController`, set up `ArtworkManager` with cache directory, initialize `LyricsManager` with LRCLIB and Netease providers, create `SettingsMenu`

**State Update (`App::update()`):**
- Location: `src/ui/mod.rs` line 391
- Triggers: Called from event loop every 500ms
- Responsibilities: Poll Apple Music for current state, detect track changes, manage background tasks for lyrics and artwork, update metadata cache

**UI Render (`draw()`):**
- Location: `src/ui/mod.rs` line 617
- Triggers: Called every iteration of event loop (~50ms)
- Responsibilities: Layout computation, render all UI components (chassis, screen, artwork, metadata, lyrics, progress bar, controls, settings overlay)

## Error Handling

**Strategy:** Graceful degradation with logging; never crash the UI loop

**Patterns:**
- `anyhow::Result` used throughout for ergonomic error propagation
- Network failures (lyrics, artwork) are logged via `tracing::debug!`/`tracing::warn!` and silently ignored -- the UI shows "NO LYRICS AVAILABLE" or "NO SIGNAL"
- `App::update()` wraps `get_player_status()` in a match: on `Err`, falls back to `(None, None)` for track and volume
- Background task panics caught via `JoinHandle::await` returning `Err(JoinError)` -- logged and ignored
- `Mutex` poisoning handled with `unwrap_or_else(|e| e.into_inner())` pattern throughout caches
- Terminal restoration guaranteed by panic hook (`restore_terminal()`) installed in `main()`

## Cross-Cutting Concerns

**Logging:** `tracing` with `tracing-subscriber` (fmt subscriber, env-filter capable). Initialized in `main()` via `tracing_subscriber::fmt::init()`. Debug-level logs for lyrics/artwork fetch flow, warn-level for task panics.

**Caching:** Three separate LRU caches:
- Artwork URL cache in `AppleMusicController` (`Mutex<LruCache<String, Option<String>>>`, capacity 20) -- `src/player/apple_music.rs`
- Processed image cache in `ArtworkCache` (`Arc<Mutex<LruCache<String, DynamicImage>>>`, capacity 100) -- `src/artwork/cache.rs`
- Lyrics cache in `LyricsManager` (`Arc<Mutex<LruCache<String, Option<Lyrics>>>>`, capacity 20) -- `src/lyrics/mod.rs`

**Configuration:** TOML-based at `~/.config/amcli/config.toml`. Loaded once at startup, saved on settings changes. Schema defined in `src/config/mod.rs`.

**Async Model:** Tokio multi-threaded runtime (`#[tokio::main]`). The main loop is synchronous (crossterm poll), but player commands, lyrics fetch, and artwork download all run as async operations. Long-running network operations are spawned as `tokio::spawn` tasks with `JoinHandle` polling to avoid blocking the draw loop.

---

*Architecture analysis: 2026-03-24*
