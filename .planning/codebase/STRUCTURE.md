# Codebase Structure

**Analysis Date:** 2026-03-24

## Directory Layout

```
amcli/
├── src/                        # All Rust source code
│   ├── main.rs                 # Binary entry point, event loop, terminal setup
│   ├── ui/                     # UI rendering and application state
│   │   ├── mod.rs              # App struct, draw(), themes, all rendering logic (~1213 lines)
│   │   └── settings.rs         # Settings menu overlay widget (~335 lines)
│   ├── player/                 # Media player abstraction and Apple Music implementation
│   │   ├── mod.rs              # MediaPlayer trait, Track, PlaybackState, RepeatMode, PlayerStatus
│   │   ├── apple_music.rs      # AppleMusicController, CommandRunner trait, AppleScript execution
│   │   └── scripts/            # Empty directory (AppleScript files are inline in Rust code)
│   ├── lyrics/                 # Multi-provider lyrics system
│   │   ├── mod.rs              # Lyrics, LyricLine, LyricsManager (orchestrator with LRU cache)
│   │   ├── provider.rs         # LyricsProvider trait definition
│   │   ├── parser.rs           # LRC format parser (parse_lrc)
│   │   ├── local.rs            # LocalProvider - reads .lrc files from ~/Music/Lyrics/
│   │   ├── lrclib.rs           # LrclibProvider - LRCLIB.net API client
│   │   └── netease.rs          # NeteaseProvider - Netease Cloud Music API client
│   ├── artwork/                # Album artwork pipeline
│   │   ├── mod.rs              # ArtworkManager, image processing (duotone, pixelation)
│   │   ├── cache.rs            # ArtworkCache (LRU memory + disk persistence)
│   │   └── converter.rs        # ArtworkConverter (ratatui-image protocol bridge)
│   └── config/                 # Configuration management
│       └── mod.rs              # Config, ArtworkConfig, UIConfig, GeneralConfig, Language
├── configs/                    # Example configuration files
│   └── config.example.toml     # Annotated example config for users
├── scripts/                    # Build and CI scripts
│   ├── applescript/            # Standalone AppleScript files (reference only)
│   │   ├── get_current_track.scpt
│   │   └── playback_control.scpt
│   ├── verify.sh               # Full verification pipeline (fmt + clippy + test + build)
│   └── pr-triage.sh            # PR triage automation script
├── .github/workflows/          # CI/CD pipeline definitions
│   ├── ci.yml                  # Main CI: check, test, fmt, clippy, build (macOS)
│   ├── release.yml             # Release workflow
│   ├── pr-validate-merge.yml   # PR merge validation
│   └── pr-conflict-scan.yml    # PR conflict scanning
├── tests/                      # Integration test directory (currently empty)
├── examples/                   # Example programs directory (currently empty)
├── assets/                     # Static assets (screenshots, etc.)
├── dist/                       # Distribution artifacts
├── homebrew/                   # Homebrew formula for installation
├── Cargo.toml                  # Rust package manifest and dependencies
├── Cargo.lock                  # Locked dependency versions
├── Makefile                    # Build convenience targets
├── CLAUDE.md                   # AI assistant project context
└── README.md                   # User-facing documentation
```

## Directory Purposes

**`src/`:**
- Purpose: All application source code
- Contains: 5 modules (ui, player, lyrics, artwork, config) plus main.rs entry point
- Key files: `main.rs` (entry), `ui/mod.rs` (largest file, core state + rendering)

**`src/ui/`:**
- Purpose: Application state management and terminal rendering
- Contains: `App` struct (central state), `draw()` function, theme definitions, settings overlay
- Key files: `mod.rs` (~1213 lines, the largest file), `settings.rs` (modal settings widget)

**`src/player/`:**
- Purpose: Media player abstraction layer and Apple Music backend
- Contains: Traits (`MediaPlayer`, `CommandRunner`), data types (`Track`, `PlaybackState`, `RepeatMode`), implementation (`AppleMusicController`)
- Key files: `mod.rs` (trait + types), `apple_music.rs` (implementation + tests)

**`src/lyrics/`:**
- Purpose: Lyrics fetching, caching, and LRC parsing
- Contains: Manager/orchestrator, trait definition, parser, three provider implementations
- Key files: `mod.rs` (LyricsManager), `provider.rs` (trait), `parser.rs` (LRC parser with tests)

**`src/artwork/`:**
- Purpose: Album art download, caching, theme-based image processing, terminal protocol conversion
- Contains: Manager, cache layer, protocol converter, image filters
- Key files: `mod.rs` (ArtworkManager + image processing), `cache.rs` (LRU + disk cache), `converter.rs` (ratatui-image bridge)

**`src/config/`:**
- Purpose: TOML configuration loading, saving, and schema definition
- Contains: Config structs with serde derive, Language enum, default values
- Key files: `mod.rs` (single file)

**`configs/`:**
- Purpose: Example configuration for users to copy
- Contains: Annotated TOML showing all current and planned config options

**`scripts/`:**
- Purpose: Build automation and CI helper scripts
- Contains: Verification script, PR triage script, reference AppleScript files

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point. Sets up tokio runtime, crossterm terminal, panic hook, runs event loop.

**Configuration:**
- `src/config/mod.rs`: Config schema and load/save logic. Runtime config path: `~/.config/amcli/config.toml`
- `configs/config.example.toml`: User-facing example configuration
- `Cargo.toml`: Rust package manifest, all dependency declarations

**Core Logic:**
- `src/ui/mod.rs`: `App` struct (lines 125-148), `App::new()` (line 151), `App::update()` (line 391), `draw()` (line 617)
- `src/player/mod.rs`: `MediaPlayer` trait (line 39), `Track` struct (line 9), `PlayerStatus` (line 32)
- `src/player/apple_music.rs`: `AppleMusicController` (line 34), `CommandRunner` trait (line 12), optimized `get_player_status()` (line 164)
- `src/lyrics/mod.rs`: `LyricsManager` (line 51), `Lyrics` struct (line 22), `LyricLine` (line 16)
- `src/lyrics/provider.rs`: `LyricsProvider` trait
- `src/lyrics/parser.rs`: `parse_lrc()` function (line 15)
- `src/artwork/mod.rs`: `ArtworkManager` (line 10), `get_artwork_themed_v2()` (line 21)
- `src/artwork/cache.rs`: `ArtworkCache` (line 8)
- `src/artwork/converter.rs`: `ArtworkConverter` (line 6)

**Rendering:**
- `src/ui/mod.rs`: `draw()` (line 617) -- main rendering entry point called every frame
- `src/ui/mod.rs`: `draw_lyrics()` (line 567) -- lyrics panel rendering
- `src/ui/settings.rs`: `SettingsMenu::render()` (line 174) -- modal overlay

**Testing:**
- `src/player/apple_music.rs`: Lines 297-355 (unit tests with `MockCommandRunner`)
- `src/lyrics/parser.rs`: Lines 84-141 (LRC parser unit tests)
- `src/ui/mod.rs`: Lines 1107-1212 (integration tests with `MockPlayer` and `TestBackend`)

## Module Hierarchy and Visibility

```
crate (src/main.rs)
├── mod artwork           (private module)
│   ├── pub mod cache     (pub within crate)
│   └── pub mod converter (pub within crate)
├── mod config            (private module)
│   └── pub types: Config, ArtworkConfig, UIConfig, GeneralConfig, Language
├── mod lyrics            (private module)
│   ├── pub mod local     (pub within crate)
│   ├── pub mod lrclib    (pub within crate)
│   ├── pub mod netease   (pub within crate)
│   ├── pub mod parser    (pub within crate)
│   └── pub mod provider  (pub within crate)
├── mod player            (private module)
│   └── pub mod apple_music (pub within crate)
└── mod ui                (private module)
    └── pub mod settings  (pub within crate)
```

All top-level modules are declared `mod` (private) in `main.rs`. Sub-modules use `pub mod` for intra-crate access. Only `crate::ui::App` is imported in `main.rs` via `use crate::ui::App`.

## Key Types and Their Locations

| Type | Location | Purpose |
|------|----------|---------|
| `App` | `src/ui/mod.rs:125` | Central application state, owns all subsystems |
| `Theme` | `src/ui/mod.rs:36` | Color scheme definition (6 built-in themes) |
| `MetadataCache` | `src/ui/mod.rs:116` | Pre-formatted track metadata for rendering |
| `SettingsMenu` | `src/ui/settings.rs:13` | Modal settings overlay state |
| `SettingsItem` | `src/ui/settings.rs:20` | Enum of configurable settings |
| `Track` | `src/player/mod.rs:9` | Current track metadata (name, artist, album, duration, position) |
| `PlaybackState` | `src/player/mod.rs:18` | Playing / Paused / Stopped enum |
| `RepeatMode` | `src/player/mod.rs:25` | Off / One / All enum |
| `PlayerStatus` | `src/player/mod.rs:32` | Combined track + volume + state snapshot |
| `MediaPlayer` | `src/player/mod.rs:39` | Async trait for player backends |
| `AppleMusicController` | `src/player/apple_music.rs:34` | Apple Music implementation via osascript |
| `CommandRunner` | `src/player/apple_music.rs:12` | Trait for subprocess execution (mockable) |
| `Lyrics` | `src/lyrics/mod.rs:22` | Parsed lyrics with lines, metadata, offset |
| `LyricLine` | `src/lyrics/mod.rs:16` | Single lyric line with text and timestamp |
| `LyricsManager` | `src/lyrics/mod.rs:51` | Orchestrator: cache check, provider iteration |
| `LyricsProvider` | `src/lyrics/provider.rs:8` | Async trait for lyrics data sources |
| `LocalProvider` | `src/lyrics/local.rs:9` | Reads .lrc files from local filesystem |
| `LrclibProvider` | `src/lyrics/lrclib.rs:13` | LRCLIB.net API client |
| `NeteaseProvider` | `src/lyrics/netease.rs:10` | Netease Cloud Music API client |
| `ArtworkManager` | `src/artwork/mod.rs:10` | Downloads and processes album artwork |
| `ArtworkCache` | `src/artwork/cache.rs:8` | LRU memory cache + disk persistence |
| `ArtworkConverter` | `src/artwork/converter.rs:6` | Converts DynamicImage to terminal protocol |
| `Config` | `src/config/mod.rs:31` | Top-level config (artwork, ui, general) |
| `Language` | `src/config/mod.rs:6` | English / Japanese enum |

## Naming Conventions

**Files:**
- Module roots: `mod.rs` in each directory
- Implementations named after the concept: `apple_music.rs`, `settings.rs`, `cache.rs`, `converter.rs`, `parser.rs`, `provider.rs`
- Provider implementations: named after the service (`local.rs`, `netease.rs`, `lrclib.rs`)
- All lowercase with underscores (snake_case)

**Directories:**
- Singular nouns: `player/`, `config/`, `ui/`
- Plural for collections: `lyrics/`, (exception: `artwork/` is singular)

## Where to Add New Code

**New Player Backend (e.g., Spotify):**
- Create `src/player/spotify.rs`
- Implement `MediaPlayer` trait from `src/player/mod.rs`
- Register in `App::new()` in `src/ui/mod.rs` (replace or wrap `AppleMusicController`)

**New Lyrics Provider:**
- Create `src/lyrics/<provider_name>.rs`
- Implement `LyricsProvider` trait from `src/lyrics/provider.rs`
- Add `pub mod <provider_name>;` to `src/lyrics/mod.rs`
- Register with `lyrics_manager.add_provider()` in `App::with_player_and_config()` at `src/ui/mod.rs` line 178-181

**New UI Component / Widget:**
- Add to `src/ui/mod.rs` if it is a core display element rendered by `draw()`
- Create a new file `src/ui/<component>.rs` if it is a self-contained overlay (like `settings.rs`)
- Add `pub mod <component>;` to `src/ui/mod.rs`

**New Configuration Section:**
- Add struct to `src/config/mod.rs` with `#[derive(Debug, Serialize, Deserialize, Clone)]`
- Add field to `Config` struct with `#[serde(default)]`
- Update `Config::default()` implementation
- Update `configs/config.example.toml` with documentation

**New Theme:**
- Add `pub const THEME_<NAME>: Theme = Theme { ... };` to `src/ui/mod.rs`
- Add it to the `THEMES` array at `src/ui/mod.rs` line 106

**Utility Functions:**
- Image processing helpers: `src/artwork/mod.rs`
- LRC parsing utilities: `src/lyrics/parser.rs`
- Duration/text formatting: `src/ui/mod.rs` (bottom of file, lines 1073-1105)

**Tests:**
- Unit tests: add `#[cfg(test)] mod tests` block at the bottom of the relevant source file
- Integration tests: place in `tests/` directory (currently empty)
- Use `MockCommandRunner` (via `mockall`) for player tests
- Use manual `MockPlayer` struct implementing `MediaPlayer` for UI tests (see `src/ui/mod.rs` line 1115)

## Config File Format and Location

**Runtime Path:** `~/.config/amcli/config.toml` (created automatically on first run with defaults)

**Example Path:** `configs/config.example.toml`

**Format (TOML):**
```toml
[general]
language = "en"           # "en" or "jp"

[artwork]
enabled = true
cache_size = 100          # LRU capacity for processed images
mode = "auto"             # "auto", "halfblocks", "sixel"
album = true              # Show album artwork panel
mosaic = true             # Apply pixelation effect

[ui]
color_theme = "default"   # Theme name (lowercase): "default", "green_vfd", "cyan_vfd", "red_alert", "modern", "clean"
show_help_on_start = true
```

**Load/Save:** `Config::load()` and `Config::save()` in `src/config/mod.rs` (async, uses `tokio::fs`)

## Special Directories

**`target/`:**
- Purpose: Rust build artifacts
- Generated: Yes (by cargo)
- Committed: No (in .gitignore)

**`.planning/`:**
- Purpose: GSD planning and codebase analysis documents
- Generated: Yes (by tooling)
- Committed: Yes

**`dist/`:**
- Purpose: Distribution artifacts (release binaries)
- Generated: Yes (by build/release process)
- Committed: Partially (may contain release assets)

**`homebrew/`:**
- Purpose: Homebrew tap formula for macOS installation
- Generated: No (manually maintained)
- Committed: Yes

**`scripts/applescript/`:**
- Purpose: Reference AppleScript files (not used at runtime -- scripts are inline in Rust)
- Generated: No
- Committed: Yes

---

*Structure analysis: 2026-03-24*
