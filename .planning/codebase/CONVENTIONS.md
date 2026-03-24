# Coding Conventions

**Analysis Date:** 2026-03-24

## Naming Patterns

**Files:**
- Module directories use `mod.rs` for the root: `src/player/mod.rs`, `src/lyrics/mod.rs`, `src/config/mod.rs`, `src/artwork/mod.rs`
- Sub-module files use `snake_case.rs`: `src/player/apple_music.rs`, `src/lyrics/lrclib.rs`, `src/artwork/converter.rs`
- UI submodules live under `src/ui/`: `src/ui/settings.rs`

**Functions:**
- Use `snake_case` for all functions and methods
- Async methods that perform I/O use `async fn`: `async fn execute_script(&self, script: &str) -> Result<String>`
- Constructor pattern: `fn new() -> Self` or `fn new(param: Type) -> Self`
- Test-only constructors: `fn with_runner(runner: Box<dyn CommandRunner>) -> Self` guarded by `#[cfg(test)]`
- Builder-style factory methods: `fn with_mode(mode: &str) -> Result<Self>` (see `src/artwork/converter.rs:11`)
- Getter methods: `fn get_current_track(&self)`, `fn get_volume(&self)`, `fn is_muted(&self)`, `fn is_settings_open(&self)`
- Toggle methods: `fn toggle_playback()`, `fn toggle_mute()`, `fn toggle_help()`, `fn toggle_settings_menu()`

**Variables:**
- Use `snake_case` for all variables
- Prefix boolean state with `is_`: `is_muted`, `is_loading_artwork`, `is_retro`, `is_jp`
- Prefix saved/cached state with context: `saved_volume`, `current_track`, `current_theme_index`

**Types:**
- Use `PascalCase` for structs, enums, and traits
- Struct names: `AppleMusicController`, `ArtworkManager`, `LyricsManager`, `MetadataCache`
- Enum names: `PlaybackState`, `RepeatMode`, `Language`, `SettingsItem`
- Trait names: `MediaPlayer`, `CommandRunner`, `LyricsProvider`
- Constants: `SCREAMING_SNAKE_CASE` for theme constants (`THEME_AMBER_RETRO`, `COLOR_BG`, `THEMES`)

**Modules:**
- Declared in `src/main.rs` as `mod artwork; mod config; mod lyrics; mod player; mod ui;`
- Re-export key types from `mod.rs` files. Sub-modules declared with `pub mod`: `pub mod apple_music;`, `pub mod cache;`

## Code Style

**Formatting:**
- `cargo fmt` (rustfmt) with default settings
- CI enforces: `cargo fmt --all -- --check`
- No `.rustfmt.toml` override file -- uses Rust defaults

**Linting:**
- `cargo clippy --all-features -- -D warnings`
- Warnings treated as errors in CI
- Liberal use of `#[allow(dead_code)]` on trait methods and struct methods reserved for future use (see `src/player/mod.rs`, `src/ui/mod.rs`, `src/artwork/cache.rs`)
- Use `#[cfg_attr(test, automock)]` for mockall-generated mocks (`src/player/apple_music.rs:10`)

## Import Organization

**Order (observed pattern):**
1. Standard library (`std::*`)
2. External crates (`anyhow`, `async_trait`, `ratatui`, `tokio`, `image`, etc.)
3. Internal crate imports (`crate::*`, `super::*`)

Note: The ordering is not strictly enforced -- some files interleave external and internal imports. The general pattern groups them loosely.

**Example from `src/player/apple_music.rs`:**
```rust
use super::{MediaPlayer, PlaybackState, PlayerStatus, RepeatMode, Track};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::time::Duration;

#[cfg(test)]
use mockall::automock;

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
```

**Example from `src/ui/mod.rs`:**
```rust
use anyhow::Result;
use image::DynamicImage;
use ratatui::{ ... };
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

use crate::artwork::converter::ArtworkConverter;
use crate::artwork::ArtworkManager;
use crate::lyrics::{ ... };
use crate::player::{apple_music::AppleMusicController, MediaPlayer, RepeatMode, Track};
```

**Path Aliases:**
- No path aliases configured. Use full crate paths: `crate::player::Track`, `crate::config::Config`

**Conditional imports:**
- `#[cfg(test)]` guards for test-only imports: `use mockall::automock;` (`src/player/apple_music.rs:8`)

## Error Handling

**Application-level errors:**
- Use `anyhow::Result` as the return type for virtually all fallible functions
- Import pattern: `use anyhow::{anyhow, Result};`
- Create ad-hoc errors with `anyhow!()` macro: `Err(anyhow!("Invalid track info format"))` (`src/player/apple_music.rs:136`)

**Module-level error types:**
- `thiserror` is listed as a dependency but no custom error enums are currently defined in the codebase
- All modules use `anyhow::Result` directly

**Error propagation:**
- Use `?` operator for propagation throughout
- Fallible external calls wrapped with `.map_err(|e| anyhow!(e))` when needed (`src/player/apple_music.rs:26`)

**Mutex poisoning:**
- Recover from mutex poison using `.unwrap_or_else(|e| e.into_inner())` -- cache data is not critical
- Pattern used consistently in `src/player/apple_music.rs:268,290`, `src/artwork/cache.rs:34,40,48,61,91`
- Exception: `src/lyrics/mod.rs:74,95,116` uses `if let Ok(mut cache) = self.cache.lock()` -- silently skips on poison

**Graceful degradation in update loop:**
- Failed operations logged via `tracing::debug!` or `tracing::warn!` but do not crash: `src/ui/mod.rs:403-406`, `src/ui/mod.rs:459-460`
- Background task panics caught with `Err(e) => tracing::warn!("...task panicked: {}", e)`: `src/ui/mod.rs:460,557`

**Panic safety:**
- Terminal restore on panic via `std::panic::set_hook` in `src/main.rs:41-45`
- `expect()` used only for infallible operations like `NonZeroUsize::new(20).expect("cache capacity must be non-zero")`

## AppleScript Patterns

**Raw string literals for AppleScript:**
- Use `r#"..."#` for all inline AppleScript strings
- Single-line commands: `r#"tell application "Music" to play"#` (`src/player/apple_music.rs:76`)
- Multi-line scripts: Use `r#"` with indented AppleScript blocks (`src/player/apple_music.rs:112-125`, `src/player/apple_music.rs:165-179`)
- Dynamic scripts with format!: `format!(r#"tell application "Music" to set sound volume to {}"#, volume)` (`src/player/apple_music.rs:218`)

**AppleScript execution:**
- All AppleScript runs through `execute_script(&self, script: &str)` on `AppleMusicController` (`src/player/apple_music.rs:59-71`)
- Uses `tokio::process::Command::new("osascript").arg("-e").arg(script)` (`src/player/apple_music.rs:21-26`)
- Checks `output.status.success()`, returns trimmed stdout or error with stderr

**Data exchange with AppleScript:**
- Simple values: Parse stdout directly (`"75"` -> `u8` for volume)
- Compound data: Use `|` delimiter for pipe-separated fields: `"Song Name|Artist Name|Album Name|180.5|90.0"` (`src/player/apple_music.rs:133`)
- Optimized compound data: Use `":::BOLT_SPLIT:::"` delimiter for batch status queries (`src/player/apple_music.rs:170-178`)

## Async Patterns

**Runtime:**
- Tokio with `#[tokio::main]` and `features = ["full"]` (`src/main.rs:35`)
- Async test annotation: `#[tokio::test]` (`src/player/apple_music.rs:311`)

**Trait definitions:**
- Use `#[async_trait]` from the `async-trait` crate for all async traits
- Pattern: `#[async_trait] pub trait MediaPlayer: Send + Sync { ... }` (`src/player/mod.rs:39-40`)
- Applied to: `MediaPlayer` (`src/player/mod.rs`), `CommandRunner` (`src/player/apple_music.rs`), `LyricsProvider` (`src/lyrics/provider.rs`)

**Background tasks:**
- Use `tokio::spawn` for non-blocking I/O operations (artwork loading, lyrics fetching)
- Store `JoinHandle` on `App` struct: `artwork_task: Option<JoinHandle<Result<DynamicImage>>>`, `lyrics_task: Option<JoinHandle<Result<Option<Lyrics>>>>` (`src/ui/mod.rs:139,145`)
- Poll with `task.is_finished()` in the update loop, then `.await` the handle (`src/ui/mod.rs:453-464,548-562`)
- Abort previous tasks when new ones start: `task.abort()` (`src/ui/mod.rs:441,509`)

**Timeouts:**
- Use `tokio::time::timeout` for external calls: `tokio::time::timeout(timeout_duration, reqwest::get(url)).await??` (`src/player/apple_music.rs:281`)
- Provider-level timeout: 5 seconds for lyrics providers (`src/lyrics/mod.rs:92`)
- HTTP client timeout: 5 seconds configured on `reqwest::Client::builder().timeout()` (`src/lyrics/netease.rs:18`, `src/lyrics/lrclib.rs:21`)

**Blocking operations:**
- Use `tokio::task::spawn_blocking` for CPU-bound image operations: `tokio::task::spawn_blocking(move || image::open(path_clone))` (`src/artwork/cache.rs:59`)

## Logging

**Framework:** `tracing` crate with `tracing-subscriber`

**Initialization:** `tracing_subscriber::fmt::init()` in `src/main.rs:38`

**Patterns:**
- `tracing::debug!` for routine operational info (cache hits/misses, provider results, update loop state)
- `tracing::info!` for notable events (lyrics found via specific provider)
- `tracing::warn!` for recoverable errors (task panics, failed status fetches)
- Format: `tracing::debug!("[UPDATE] status OK: track={}, vol={:?}", ...)` -- use bracketed context prefix

**When to log:**
- Cache hit/miss events
- Provider query results (success, no result, error, timeout)
- Background task completion (success, error, panic)
- State update diagnostics in the main update loop

## Comments

**When to Comment:**
- Document optimization rationale: the "Bolt Optimization" comment at `src/player/apple_music.rs:159-162` explains why AppleScript calls are batched
- Performance notes: `src/ui/mod.rs:1085-1086` explains benchmark results for `scroll_text`
- Algorithm explanations: `src/artwork/mod.rs:94-100` documents luminance normalization math
- Section separators within long `draw()` function are rare; code is mostly self-documenting

**Doc comments:**
- Use `///` sparingly: only on `src/artwork/cache.rs` methods (`/// Synchronous memory-only cache lookup`, `/// Async cache lookup with disk fallback`)
- No module-level `//!` documentation

**Inline comments:**
- `// Comment` style for brief explanations
- File-level location comment on first line: `// src/main.rs`, `// src/player/mod.rs`, `// src/lyrics/parser.rs` (not consistently applied)

## Function Design

**Size:**
- Most methods are short (5-20 lines), with the notable exception of `draw()` in `src/ui/mod.rs` (~500 lines)
- Helper functions extracted: `format_duration`, `format_duration_seconds`, `scroll_text`, `draw_lyrics`, `apply_pixelation`, `apply_duotone_theme`

**Parameters:**
- Prefer `&self` for methods on struct instances
- Use `&str` for string parameters, `String` for owned data in structs
- Use references for read-only access: `track: &Track`

**Return Values:**
- `Result<()>` for commands/side-effects
- `Result<Option<T>>` for queries that may return nothing (tracks, lyrics, artwork URLs)
- `Cow<'a, str>` for optimized string returns that may or may not allocate (`scroll_text` in `src/ui/mod.rs:1087`)

## Module Design

**Exports:**
- Each `mod.rs` re-exports key types used by other modules
- `src/player/mod.rs` exports `Track`, `PlaybackState`, `RepeatMode`, `PlayerStatus`, `MediaPlayer` trait
- `src/lyrics/mod.rs` exports `LyricLine`, `Lyrics`, `LyricsManager` and sub-module paths
- `src/ui/mod.rs` exports `App`, `draw`, theme constants, `MetadataCache`

**Barrel Files:**
- No barrel re-exports. Consumers use full paths: `crate::lyrics::local::LocalProvider`, `crate::player::apple_music::AppleMusicController`

**Trait-based abstractions:**
- `MediaPlayer` trait (`src/player/mod.rs`) abstracts the player backend -- allows mock injection for testing
- `CommandRunner` trait (`src/player/apple_music.rs`) abstracts the osascript execution -- enables unit testing without Apple Music
- `LyricsProvider` trait (`src/lyrics/provider.rs`) with `priority()` method for ordered provider chain

## Commit Message Format

**Convention:** `<type>(<scope>): <subject>`

**Types observed:**
- `feat` -- New features
- `fix` -- Bug fixes
- `release` -- Version bumps
- `docs` -- Documentation changes
- `ci` -- CI/CD changes

**Scopes observed:**
- Module names: `player`, `ui`, `artwork`, `lyrics`
- Combined: `artwork,lyrics`, `player,ui`
- Omitted for cross-cutting changes: `fix: address code audit findings`

**Examples from git log:**
```
feat(artwork,lyrics): implement actual artwork download and lyrics fetching
fix(ui): revert MetadataCache draw dependency to use track data directly
fix: address code audit findings (mutex poison, terminal panic, error logging)
release: bump version to 0.2.0
docs: add CLAUDE.md with project conventions and commands
ci: add PR validation, conflict scanning, and triage automation
```

## CI Checks

**Pipeline:** GitHub Actions on `macos-latest` (`/.github/workflows/ci.yml`)

**Triggered on:** Push to `main`/`develop`, PRs targeting `main`/`develop`

**Jobs (all run independently, all use `--all-features`):**
1. `cargo check --all-features`
2. `cargo test --all-features`
3. `cargo fmt --all -- --check`
4. `cargo clippy --all-features -- -D warnings`
5. `cargo build --release --all-features`

**Local verification:** `make verify` runs `scripts/verify.sh` which executes fmt check, clippy, test, build, and doc generation in sequence.

---

*Convention analysis: 2026-03-24*
