# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AMCLI is a Rust TUI application for controlling Apple Music on macOS. It uses AppleScript via `osascript` to communicate with the Apple Music app, Ratatui for terminal rendering, and Tokio for async operations.

## Common Commands

```bash
make build          # cargo build --release
make test           # cargo test
make fmt            # cargo fmt
make lint           # cargo clippy -- -D warnings
make verify         # Full pipeline: fmt check + clippy + test + build
make run            # cargo run
make install        # cargo install --path . (installs to ~/.cargo/bin)
make clean          # Remove build artifacts, bin/, dist/
cargo test <name>   # Run a specific test
cargo test -- --nocapture  # Run tests with stdout visible
```

CI runs on macOS with `--all-features` on all steps. Use `make verify` (runs `scripts/verify.sh`) to replicate the full CI pipeline locally. Pass `--all-features` when running individual cargo commands.

## Planning Artifacts

`.planning/` contains project-level context: `PROJECT.md` (goals, milestones) and `codebase/` (architecture, conventions, concerns, testing analysis). These are generated docs — read them for context but prefer the source code as the authoritative reference.

## Architecture

**Entry point** (`src/main.rs`): Sets up Tokio runtime, Crossterm terminal (raw mode, alternate screen), and runs a 50ms poll event loop with 500ms state refresh.

**Key modules:**

- **`src/ui/mod.rs`** — `App` struct is the central state hub. Contains all rendering logic (Ratatui), keyboard handling, and application state. Has a modal settings menu (`src/ui/settings.rs`). Supports 6 color themes.
- **`src/player/`** — `MediaPlayer` trait defines the player abstraction. `AppleMusicController` implements it by executing AppleScript via `osascript` CLI. Uses `CommandRunner` trait for testability (mocked with `mockall`).
- **`src/lyrics/`** — Multi-provider lyrics system with priority: local files → Netease → LRCLIB. Parses LRC format with timestamp regex. `LyricsManager` orchestrates providers via the `LyricsProvider` trait.
- **`src/artwork/`** — Album art with LRU caching (`cache.rs`). Protocol conversion (Sixel, Kitty, halfblocks) via `ratatui-image` in `converter.rs`.
- **`src/config/`** — TOML-based config with serde. Supports language (en/ja), theme selection, artwork mode, mosaic effects.

## Conventions

- **AppleScript strings**: Use raw string literals (`r#"..."#`) for inline AppleScript.
- **Async traits**: Use `#[async_trait]` for traits with async methods.
- **Error handling**: `anyhow::Result` for application logic, `thiserror` for defining error types in modules.
- **Never block the UI draw loop** with I/O (network, AppleScript calls).
- **Commit messages**: `<type>(<scope>): <subject>` format (e.g., `feat(player): add volume control`, `fix(ui): resolve layout overflow`).
- **macOS only**: Requires Apple Music app installed. Album art display depends on terminal protocol support.

<!-- GSD:project-start source:PROJECT.md -->
## Project

**AMCLI**

A macOS terminal UI application for controlling Apple Music. Renders album artwork, synchronized lyrics, playback controls, and track metadata in a Ratatui-powered TUI. Communicates with Apple Music via AppleScript/osascript.

**Core Value:** The TUI looks polished and adapts gracefully to any terminal size — artwork, lyrics, controls, and metadata all use available space well without breaking layout.

### Constraints

- **Platform**: macOS only — requires Apple Music app
- **Framework**: Ratatui 0.30 + Crossterm 0.28 — all layout via Ratatui constraint system
- **Terminal size**: Should look good at any reasonable terminal size, no minimum
- **No new dependencies**: Prefer using existing Ratatui layout primitives
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Rust (Edition 2021) - All application code
- AppleScript - Inline scripts executed via `osascript` for Apple Music IPC (`src/player/apple_music.rs`)
- Bash - Build verification script (`scripts/verify.sh`)
- YAML - GitHub Actions CI/CD (`.github/workflows/`)
## Runtime
- macOS only (requires Apple Music app installed)
- Tokio async runtime (`tokio 1.35` with `features = ["full"]`)
- Terminal raw mode via Crossterm (alternate screen, mouse capture)
- Cargo (Rust standard)
- Lockfile: `Cargo.lock` present and committed
## Frameworks
- `ratatui 0.30` - Terminal UI framework (rendering, layout, widgets)
- `crossterm 0.28` - Terminal backend (raw mode, events, alternate screen)
- `tokio 1.35` - Async runtime with full features (process spawning, timers, fs, task)
- `mockall 0.12` - Mock generation for trait-based testing (dev-dependency)
- `tokio-test 0.4` - Async test utilities (dev-dependency)
- Built-in `cargo test` runner (no custom test framework)
- Cargo with Makefile wrapper (`Makefile`)
- Clippy for linting (`cargo clippy -- -D warnings`)
- Rustfmt for formatting (`cargo fmt`)
- Full verification script: `scripts/verify.sh` (fmt check, clippy, test, build, doc)
## Key Dependencies
- `ratatui 0.30` - All UI rendering (`src/ui/mod.rs`, `src/ui/settings.rs`)
- `crossterm 0.28` - Terminal I/O, keyboard/mouse events (`src/main.rs`)
- `tokio 1.35` - Async runtime, process spawning for osascript, file I/O, timers
- `reqwest 0.11` (with `json`, `rustls-tls`, no default features) - HTTP client for lyrics APIs and artwork URLs (`src/lyrics/lrclib.rs`, `src/lyrics/netease.rs`, `src/artwork/mod.rs`)
- `image 0.25` - Album artwork image loading, processing, duotone/pixelation effects (`src/artwork/mod.rs`)
- `ratatui-image 10.0` (no default features) - Terminal image protocol rendering: Sixel, Kitty, halfblocks (`src/artwork/converter.rs`)
- `anyhow 1.0` - Application-level error handling (`Result<T>` throughout)
- `thiserror 1.0` - Typed error definitions in modules
- `async-trait 0.1` - Async methods in traits (`MediaPlayer`, `LyricsProvider`, `CommandRunner`)
- `serde 1.0` (with `derive`) - Serialization for config and API responses
- `toml 0.8` - Config file parsing/writing (`src/config/mod.rs`)
- `serde_json 1.0` - JSON parsing for API responses (iTunes Search, LRCLIB, Netease)
- `clap 4.4` (with `derive`) - CLI argument parsing (`src/main.rs`)
- `tracing 0.1` + `tracing-subscriber 0.3` (with `env-filter`) - Structured logging
- `lru 0.12` - LRU caching for artwork URLs and lyrics (`src/player/apple_music.rs`, `src/lyrics/mod.rs`, `src/artwork/cache.rs`)
- `dirs 5.0` - Platform config directory resolution (`src/config/mod.rs`)
- `regex 1.10` - LRC timestamp parsing (`src/lyrics/parser.rs`)
- `lazy_static 1.4` - Compiled regex singletons (`src/lyrics/parser.rs`)
- `sha2 0.10` - URL hashing for disk cache keys (`src/artwork/cache.rs`)
- `urlencoding 2.1` - URL parameter encoding for API calls
- `chrono 0.4` - Date/time utilities
- `unicode-width 0.1` - CJK character width for UI layout
- `rgb 0.8` - RGB color utilities
- `futures 0.3` - Future combinators
- `config 0.13` - Configuration framework (declared but primarily using custom TOML loading)
- `throbber-widgets-tui 0.10` - Loading spinner widgets
- `tui-big-text 0.8` - Large text rendering
## Configuration
- TOML format at `~/.config/amcli/config.toml` (via `dirs::config_dir()`)
- Auto-created with defaults on first run
- Sections: `artwork` (enabled, cache_size, mode, album, mosaic), `ui` (color_theme, show_help_on_start), `general` (language)
- Schema defined in `src/config/mod.rs` via serde derive
- `Cargo.toml` - Package manifest, dependencies, release profile
- No `rust-toolchain.toml` - uses stable toolchain via CI (`dtolnay/rust-toolchain@stable`)
## Build Commands
| Command | Action |
|---------|--------|
| `make build` | `cargo build --release` |
| `make test` | `cargo test` |
| `make fmt` | `cargo fmt` |
| `make lint` | `cargo clippy -- -D warnings` |
| `make verify` | Full pipeline: fmt check + clippy + test + build + doc (`scripts/verify.sh`) |
| `make run` | `cargo run` |
| `make install` | `cargo install --path .` |
| `make clean` | `cargo clean` + remove `bin/` and `dist/` |
## CI/CD
- Runs on: `macos-latest`
- Triggers: push/PR to `main` and `develop`
- Jobs (parallel): `cargo check`, `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build --release` (all with `--all-features`)
- Toolchain: `dtolnay/rust-toolchain@stable`
- Triggers: git tag `v*.*.*` or manual dispatch
- Builds for both `x86_64-apple-darwin` (macos-13) and `aarch64-apple-darwin` (macos-14)
- Creates GitHub Release with tar.gz archives and SHA256 checksums
- Dispatches Homebrew formula update (`brew tap juntaochi/tap`)
- PR validation and conflict scanning automation
## Platform Requirements
- macOS (Apple Music app must be installed for integration testing)
- Rust stable toolchain
- Terminal with optional Sixel/Kitty support for album art
- macOS only (AppleScript IPC with Music.app)
- Binary targets: `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Distribution: Homebrew tap (`juntaochi/tap`) or direct binary download
- No external runtime dependencies (statically linked with LTO)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- Module directories use `mod.rs` for the root: `src/player/mod.rs`, `src/lyrics/mod.rs`, `src/config/mod.rs`, `src/artwork/mod.rs`
- Sub-module files use `snake_case.rs`: `src/player/apple_music.rs`, `src/lyrics/lrclib.rs`, `src/artwork/converter.rs`
- UI submodules live under `src/ui/`: `src/ui/settings.rs`
- Use `snake_case` for all functions and methods
- Async methods that perform I/O use `async fn`: `async fn execute_script(&self, script: &str) -> Result<String>`
- Constructor pattern: `fn new() -> Self` or `fn new(param: Type) -> Self`
- Test-only constructors: `fn with_runner(runner: Box<dyn CommandRunner>) -> Self` guarded by `#[cfg(test)]`
- Builder-style factory methods: `fn with_mode(mode: &str) -> Result<Self>` (see `src/artwork/converter.rs:11`)
- Getter methods: `fn get_current_track(&self)`, `fn get_volume(&self)`, `fn is_muted(&self)`, `fn is_settings_open(&self)`
- Toggle methods: `fn toggle_playback()`, `fn toggle_mute()`, `fn toggle_help()`, `fn toggle_settings_menu()`
- Use `snake_case` for all variables
- Prefix boolean state with `is_`: `is_muted`, `is_loading_artwork`, `is_retro`, `is_jp`
- Prefix saved/cached state with context: `saved_volume`, `current_track`, `current_theme_index`
- Use `PascalCase` for structs, enums, and traits
- Struct names: `AppleMusicController`, `ArtworkManager`, `LyricsManager`, `MetadataCache`
- Enum names: `PlaybackState`, `RepeatMode`, `Language`, `SettingsItem`
- Trait names: `MediaPlayer`, `CommandRunner`, `LyricsProvider`
- Constants: `SCREAMING_SNAKE_CASE` for theme constants (`THEME_AMBER_RETRO`, `COLOR_BG`, `THEMES`)
- Declared in `src/main.rs` as `mod artwork; mod config; mod lyrics; mod player; mod ui;`
- Re-export key types from `mod.rs` files. Sub-modules declared with `pub mod`: `pub mod apple_music;`, `pub mod cache;`
## Code Style
- `cargo fmt` (rustfmt) with default settings
- CI enforces: `cargo fmt --all -- --check`
- No `.rustfmt.toml` override file -- uses Rust defaults
- `cargo clippy --all-features -- -D warnings`
- Warnings treated as errors in CI
- Liberal use of `#[allow(dead_code)]` on trait methods and struct methods reserved for future use (see `src/player/mod.rs`, `src/ui/mod.rs`, `src/artwork/cache.rs`)
- Use `#[cfg_attr(test, automock)]` for mockall-generated mocks (`src/player/apple_music.rs:10`)
## Import Organization
#[cfg(test)]
- No path aliases configured. Use full crate paths: `crate::player::Track`, `crate::config::Config`
- `#[cfg(test)]` guards for test-only imports: `use mockall::automock;` (`src/player/apple_music.rs:8`)
## Error Handling
- Use `anyhow::Result` as the return type for virtually all fallible functions
- Import pattern: `use anyhow::{anyhow, Result};`
- Create ad-hoc errors with `anyhow!()` macro: `Err(anyhow!("Invalid track info format"))` (`src/player/apple_music.rs:136`)
- `thiserror` is listed as a dependency but no custom error enums are currently defined in the codebase
- All modules use `anyhow::Result` directly
- Use `?` operator for propagation throughout
- Fallible external calls wrapped with `.map_err(|e| anyhow!(e))` when needed (`src/player/apple_music.rs:26`)
- Recover from mutex poison using `.unwrap_or_else(|e| e.into_inner())` -- cache data is not critical
- Pattern used consistently in `src/player/apple_music.rs:268,290`, `src/artwork/cache.rs:34,40,48,61,91`
- Exception: `src/lyrics/mod.rs:74,95,116` uses `if let Ok(mut cache) = self.cache.lock()` -- silently skips on poison
- Failed operations logged via `tracing::debug!` or `tracing::warn!` but do not crash: `src/ui/mod.rs:403-406`, `src/ui/mod.rs:459-460`
- Background task panics caught with `Err(e) => tracing::warn!("...task panicked: {}", e)`: `src/ui/mod.rs:460,557`
- Terminal restore on panic via `std::panic::set_hook` in `src/main.rs:41-45`
- `expect()` used only for infallible operations like `NonZeroUsize::new(20).expect("cache capacity must be non-zero")`
## AppleScript Patterns
- Use `r#"..."#` for all inline AppleScript strings
- Single-line commands: `r#"tell application "Music" to play"#` (`src/player/apple_music.rs:76`)
- Multi-line scripts: Use `r#"` with indented AppleScript blocks (`src/player/apple_music.rs:112-125`, `src/player/apple_music.rs:165-179`)
- Dynamic scripts with format!: `format!(r#"tell application "Music" to set sound volume to {}"#, volume)` (`src/player/apple_music.rs:218`)
- All AppleScript runs through `execute_script(&self, script: &str)` on `AppleMusicController` (`src/player/apple_music.rs:59-71`)
- Uses `tokio::process::Command::new("osascript").arg("-e").arg(script)` (`src/player/apple_music.rs:21-26`)
- Checks `output.status.success()`, returns trimmed stdout or error with stderr
- Simple values: Parse stdout directly (`"75"` -> `u8` for volume)
- Compound data: Use `|` delimiter for pipe-separated fields: `"Song Name|Artist Name|Album Name|180.5|90.0"` (`src/player/apple_music.rs:133`)
- Optimized compound data: Use `":::BOLT_SPLIT:::"` delimiter for batch status queries (`src/player/apple_music.rs:170-178`)
## Async Patterns
- Tokio with `#[tokio::main]` and `features = ["full"]` (`src/main.rs:35`)
- Async test annotation: `#[tokio::test]` (`src/player/apple_music.rs:311`)
- Use `#[async_trait]` from the `async-trait` crate for all async traits
- Pattern: `#[async_trait] pub trait MediaPlayer: Send + Sync { ... }` (`src/player/mod.rs:39-40`)
- Applied to: `MediaPlayer` (`src/player/mod.rs`), `CommandRunner` (`src/player/apple_music.rs`), `LyricsProvider` (`src/lyrics/provider.rs`)
- Use `tokio::spawn` for non-blocking I/O operations (artwork loading, lyrics fetching)
- Store `JoinHandle` on `App` struct: `artwork_task: Option<JoinHandle<Result<DynamicImage>>>`, `lyrics_task: Option<JoinHandle<Result<Option<Lyrics>>>>` (`src/ui/mod.rs:139,145`)
- Poll with `task.is_finished()` in the update loop, then `.await` the handle (`src/ui/mod.rs:453-464,548-562`)
- Abort previous tasks when new ones start: `task.abort()` (`src/ui/mod.rs:441,509`)
- Use `tokio::time::timeout` for external calls: `tokio::time::timeout(timeout_duration, reqwest::get(url)).await??` (`src/player/apple_music.rs:281`)
- Provider-level timeout: 5 seconds for lyrics providers (`src/lyrics/mod.rs:92`)
- HTTP client timeout: 5 seconds configured on `reqwest::Client::builder().timeout()` (`src/lyrics/netease.rs:18`, `src/lyrics/lrclib.rs:21`)
- Use `tokio::task::spawn_blocking` for CPU-bound image operations: `tokio::task::spawn_blocking(move || image::open(path_clone))` (`src/artwork/cache.rs:59`)
## Logging
- `tracing::debug!` for routine operational info (cache hits/misses, provider results, update loop state)
- `tracing::info!` for notable events (lyrics found via specific provider)
- `tracing::warn!` for recoverable errors (task panics, failed status fetches)
- Format: `tracing::debug!("[UPDATE] status OK: track={}, vol={:?}", ...)` -- use bracketed context prefix
- Cache hit/miss events
- Provider query results (success, no result, error, timeout)
- Background task completion (success, error, panic)
- State update diagnostics in the main update loop
## Comments
- Document optimization rationale: the "Bolt Optimization" comment at `src/player/apple_music.rs:159-162` explains why AppleScript calls are batched
- Performance notes: `src/ui/mod.rs:1085-1086` explains benchmark results for `scroll_text`
- Algorithm explanations: `src/artwork/mod.rs:94-100` documents luminance normalization math
- Section separators within long `draw()` function are rare; code is mostly self-documenting
- Use `///` sparingly: only on `src/artwork/cache.rs` methods (`/// Synchronous memory-only cache lookup`, `/// Async cache lookup with disk fallback`)
- No module-level `//!` documentation
- `// Comment` style for brief explanations
- File-level location comment on first line: `// src/main.rs`, `// src/player/mod.rs`, `// src/lyrics/parser.rs` (not consistently applied)
## Function Design
- Most methods are short (5-20 lines), with the notable exception of `draw()` in `src/ui/mod.rs` (~500 lines)
- Helper functions extracted: `format_duration`, `format_duration_seconds`, `scroll_text`, `draw_lyrics`, `apply_pixelation`, `apply_duotone_theme`
- Prefer `&self` for methods on struct instances
- Use `&str` for string parameters, `String` for owned data in structs
- Use references for read-only access: `track: &Track`
- `Result<()>` for commands/side-effects
- `Result<Option<T>>` for queries that may return nothing (tracks, lyrics, artwork URLs)
- `Cow<'a, str>` for optimized string returns that may or may not allocate (`scroll_text` in `src/ui/mod.rs:1087`)
## Module Design
- Each `mod.rs` re-exports key types used by other modules
- `src/player/mod.rs` exports `Track`, `PlaybackState`, `RepeatMode`, `PlayerStatus`, `MediaPlayer` trait
- `src/lyrics/mod.rs` exports `LyricLine`, `Lyrics`, `LyricsManager` and sub-module paths
- `src/ui/mod.rs` exports `App`, `draw`, theme constants, `MetadataCache`
- No barrel re-exports. Consumers use full paths: `crate::lyrics::local::LocalProvider`, `crate::player::apple_music::AppleMusicController`
- `MediaPlayer` trait (`src/player/mod.rs`) abstracts the player backend -- allows mock injection for testing
- `CommandRunner` trait (`src/player/apple_music.rs`) abstracts the osascript execution -- enables unit testing without Apple Music
- `LyricsProvider` trait (`src/lyrics/provider.rs`) with `priority()` method for ordered provider chain
## Commit Message Format
- `feat` -- New features
- `fix` -- Bug fixes
- `release` -- Version bumps
- `docs` -- Documentation changes
- `ci` -- CI/CD changes
- Module names: `player`, `ui`, `artwork`, `lyrics`
- Combined: `artwork,lyrics`, `player,ui`
- Omitted for cross-cutting changes: `fix: address code audit findings`
## CI Checks
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- Single-binary TUI application with a synchronous 50ms poll event loop driving async operations
- Trait-based abstraction layer for player backends and lyrics providers, enabling testability via mocking
- Background `tokio::spawn` tasks for network I/O (lyrics fetch, artwork download) to avoid blocking the UI draw loop
- All application state centralized in a single `App` struct in `src/ui/mod.rs`
- AppleScript IPC via `osascript` CLI subprocess for all Apple Music communication
## Layers
- Purpose: Initialize terminal, configure panic hook, run the main poll-draw-update loop
- Location: `src/main.rs`
- Contains: `main()`, `run_app()`, `restore_terminal()`, CLI arg parsing via `clap`
- Depends on: `ui` (App, draw), `crossterm`, `ratatui`, `tokio`
- Used by: Nothing (top-level binary entry)
- Purpose: Central state hub, rendering logic, keyboard input delegation, async task management
- Location: `src/ui/mod.rs`, `src/ui/settings.rs`
- Contains: `App` struct (all mutable state), `draw()` function (all rendering), `Theme` definitions, `MetadataCache`, `SettingsMenu`
- Depends on: `player` (MediaPlayer trait, Track, RepeatMode), `lyrics` (LyricsManager, Lyrics, all providers), `artwork` (ArtworkManager, ArtworkConverter), `config` (Config, Language)
- Used by: `main.rs` (creates App, calls draw, dispatches key events)
- Purpose: Define the media player interface and implement Apple Music control via AppleScript
- Location: `src/player/mod.rs`, `src/player/apple_music.rs`
- Contains: `MediaPlayer` trait, `Track`, `PlaybackState`, `RepeatMode`, `PlayerStatus`, `AppleMusicController`, `CommandRunner` trait
- Depends on: `tokio::process::Command`, `reqwest` (for iTunes artwork API), `lru` (artwork URL cache)
- Used by: `ui` module (via `Box<dyn MediaPlayer>`)
- Purpose: Multi-provider lyrics fetching with LRC parsing and LRU caching
- Location: `src/lyrics/mod.rs`, `src/lyrics/provider.rs`, `src/lyrics/parser.rs`, `src/lyrics/local.rs`, `src/lyrics/netease.rs`, `src/lyrics/lrclib.rs`
- Contains: `LyricsManager`, `LyricsProvider` trait, `Lyrics`, `LyricLine`, `parse_lrc()`, three provider implementations
- Depends on: `player` (Track struct), `reqwest` (HTTP for Netease/LRCLIB), `tokio::fs` (local file provider)
- Used by: `ui` module (via `Arc<LyricsManager>`)
- Purpose: Download, cache, theme, and convert album artwork images for terminal rendering
- Location: `src/artwork/mod.rs`, `src/artwork/cache.rs`, `src/artwork/converter.rs`
- Contains: `ArtworkManager`, `ArtworkCache` (LRU memory + disk), `ArtworkConverter` (ratatui-image protocol adapter), image processing functions (duotone, pixelation)
- Depends on: `image`, `ratatui-image`, `reqwest`, `sha2`, `lru`
- Used by: `ui` module (via `ArtworkManager` and `ArtworkConverter`)
- Purpose: Load/save TOML config, define configuration schema
- Location: `src/config/mod.rs`
- Contains: `Config`, `ArtworkConfig`, `UIConfig`, `GeneralConfig`, `Language` enum
- Depends on: `serde`, `toml`, `dirs`, `tokio::fs`
- Used by: `ui` module (reads config at startup, saves on settings changes)
## Data Flow
- All mutable state lives in `App` struct fields (no external state store, no message passing)
- `App` is owned by `run_app()` and passed as `&mut` to both event handlers and the draw function
- Background tasks communicate results via `JoinHandle` polling (not channels)
- Caches use `Arc<Mutex<LruCache>>` for thread-safe shared access between spawned tasks and main loop
## Key Abstractions
- Purpose: Abstract over different media player backends (currently only Apple Music)
- Definition: `src/player/mod.rs` lines 39-76
- Implementation: `src/player/apple_music.rs` `AppleMusicController`
- Pattern: `#[async_trait]` with `Box<dyn MediaPlayer>` stored in `App`
- Key methods: `get_player_status()` (hot path), `toggle()`, `next()`, `previous()`, `set_volume()`, `seek()`, `get_artwork_url()`
- Purpose: Abstract the `osascript` subprocess execution for testability
- Definition: `src/player/apple_music.rs` lines 10-14
- Implementations: `OsascriptRunner` (production), `MockCommandRunner` (test, via `mockall`)
- Pattern: `#[cfg_attr(test, automock)]` generates mock at compile time
- Purpose: Abstract over different lyrics data sources
- Definition: `src/lyrics/provider.rs`
- Implementations: `LocalProvider` (`src/lyrics/local.rs`), `LrclibProvider` (`src/lyrics/lrclib.rs`), `NeteaseProvider` (`src/lyrics/netease.rs`)
- Pattern: `#[async_trait]` with `Box<dyn LyricsProvider>` stored in `Vec` inside `LyricsManager`, wrapped in `Arc` for sharing across tasks
- Priority system: `fn priority(&self) -> u8` determines query order (lower first)
- Purpose: Define color schemes for the entire UI
- Definition: `src/ui/mod.rs` lines 35-113
- Pattern: Const `Theme` structs stored in a static `THEMES` array, indexed by `current_theme_index` in `App`
- Affects: All rendering (borders, text, backgrounds), artwork processing (duotone filter colors), and whether retro visual effects apply
## Entry Points
- Location: `src/main.rs` line 36
- Triggers: `cargo run` or `amcli` binary execution
- Responsibilities: Parse CLI args, set up tracing, install panic hook, initialize terminal (raw mode, alternate screen), create `App`, run event loop, restore terminal on exit
- Location: `src/ui/mod.rs` line 151
- Triggers: Called once from `main()`
- Responsibilities: Load config from TOML, create `AppleMusicController`, set up `ArtworkManager` with cache directory, initialize `LyricsManager` with all three providers, create `SettingsMenu`
- Location: `src/ui/mod.rs` line 391
- Triggers: Called from event loop every 500ms
- Responsibilities: Poll Apple Music for current state, detect track changes, manage background tasks for lyrics and artwork, update metadata cache
- Location: `src/ui/mod.rs` line 617
- Triggers: Called every iteration of event loop (~50ms)
- Responsibilities: Layout computation, render all UI components (chassis, screen, artwork, metadata, lyrics, progress bar, controls, settings overlay)
## Error Handling
- `anyhow::Result` used throughout for ergonomic error propagation
- Network failures (lyrics, artwork) are logged via `tracing::debug!`/`tracing::warn!` and silently ignored -- the UI shows "NO LYRICS AVAILABLE" or "NO SIGNAL"
- `App::update()` wraps `get_player_status()` in a match: on `Err`, falls back to `(None, None)` for track and volume
- Background task panics caught via `JoinHandle::await` returning `Err(JoinError)` -- logged and ignored
- `Mutex` poisoning handled with `unwrap_or_else(|e| e.into_inner())` pattern throughout caches
- Terminal restoration guaranteed by panic hook (`restore_terminal()`) installed in `main()`
## Cross-Cutting Concerns
- Artwork URL cache in `AppleMusicController` (`Mutex<LruCache<String, Option<String>>>`, capacity 20) -- `src/player/apple_music.rs`
- Processed image cache in `ArtworkCache` (`Arc<Mutex<LruCache<String, DynamicImage>>>`, capacity 100) -- `src/artwork/cache.rs`
- Lyrics cache in `LyricsManager` (`Arc<Mutex<LruCache<String, Option<Lyrics>>>>`, capacity 20) -- `src/lyrics/mod.rs`
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
