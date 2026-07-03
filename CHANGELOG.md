# Changelog / 更新日志

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-07-03

### Added
- **Current-line lyrics marquee** — The active lyric line scrolls horizontally when it exceeds the pane width, measured in display cells so CJK lyrics scroll and fit correctly.
- **Scored LRCLIB search** — Rank LRCLIB search results with release-type-aware matching instead of taking the first hit.
- **Provider racing** — Race lyrics providers to calibrate query priority; the local file provider has been removed.

### Fixed
- **Netease match selection** — Pick the best match across all search queries instead of the first query that returns anything.
- **Chinese duration fallback guardrails** — The duration-only fallback for Chinese lyrics now only applies to trusted (artist-bearing) searches and can never outrank a match that agrees on title, artist, or album, preventing unrelated Chinese lyrics from being cached for English tracks. LRCLIB matching behavior is unchanged.
- **Provider errors surfaced** — Unreachable lyrics providers are reported instead of being masked as "no lyrics found", so temporary network failures no longer get cached as permanent misses.

### Changed
- **Marquee speed** — Scrolling text advances one character per frame (twice the previous speed).
- **README** — Screenshots interspersed throughout the feature documentation.

## [0.2.2] - 2026-06-30

### Added
- **Terminal title now playing** — Update the terminal window/tab title to `AMCLI: <track name>` as the active song changes.
- **Release asset backfill workflow** — Add a manual workflow for uploading missing macOS release artifacts to existing tags.

### Fixed
- **Netease lyrics matching** — Search with title, album, and artist context, preserve Netease result rank, and weight close duration matches so localized catalog results are selected without hardcoded alias tables.
- **Lyrics retry recovery** — Cache only successful lyric lookups so a temporary empty result does not prevent later attempts from loading lyrics for the same track.
- **Artwork retry recovery** — Clear failed artwork URLs so transient image load failures can retry on later updates.

### Changed
- **Footer affordance** — Show the theme-switch shortcut in the footer controls.
- **Lyrics and artwork README coverage** — Document LRCLIB/Netease matching, version-aware lyric caching, current-track artwork preference, and artwork retry behavior.

## [0.2.1] - 2026-06-29

### Fixed
- **Artwork overlay repaint** — Force a full terminal repaint after closing settings overlays so terminal graphics protocol artwork does not retain stale menu content.

### Changed
- **Mosaic rendering** — Restore the selected fixed 8px alpha-weighted averaged mosaic style without sharpening, contrast, saturation, or resampling filters.

## [0.2.0] - 2026-06-29

### Performance
- **Batched AppleScript IPC** — Combine player state, volume, and track metadata into a single `osascript` call, halving process overhead during the 500ms UI update loop
- **LRU artwork cache** — Replace single-entry artwork cache with 20-entry LRU cache, eliminating redundant iTunes API calls when switching between recently played tracks
- **Zero-allocation draw loop** — Pre-compute uppercase metadata and formatted duration strings in a `MetadataCache`, avoiding `format!`/`to_uppercase()` allocations on every ~50ms draw tick
- **Cow-based scroll_text** — Return `Cow::Borrowed` when text fits within column width, avoiding heap allocation for non-scrolling text

### Fixed
- **Metadata rendering dependency** — Render UI metadata directly from track state so display updates do not depend on stale optimization cache state.
- **Artwork display** — Restored synchronous artwork fetch that was broken by an async fire-and-forget pattern which always returned `None` on first call
- **Lyrics rendering** — Decoupled lyrics display from metadata cache check so lyrics render independently of the optimization cache
- **Artwork protocol converter** — Fixed panic in artwork protocol conversion
- **Terminal resilience** — Harden mutex poison handling, terminal panic recovery, and error logging paths.
- **Compact layout polish** — Tighten artwork/info spacing, metadata height, and lyrics gaps across constrained viewports.

### Added
- **Real artwork fetching pipeline** — Download and cache Apple Music artwork instead of relying only on placeholder or previously cached data.
- **Expanded lyrics fetching** — Improve local, LRCLIB, and Netease provider integration so synced lyrics are fetched and surfaced more reliably.
- **Responsive playback layout** — Collapse controls and then the progress bar on short terminal heights to keep the now-playing view usable.
- **Graduated lyrics dimming** — Add three-tier dimming around the current lyric line for better scanability during playback.
- **PR validation workflow** (`pr-validate-merge.yml`) — Tests the merged result against latest `main` (not just the PR branch), with Cargo caching, concurrency groups, and auto-merge support via `auto-merge` label
- **PR conflict scanner** (`pr-conflict-scan.yml`) — Daily scan of all open PRs for merge conflicts and file overlaps, reports to a tracking GitHub Issue
- **Local PR triage script** (`scripts/pr-triage.sh`) — Analyzes all open PRs, finds safe sequential merge order, supports `--auto-merge` and `--close-conflicts`
- **CLAUDE.md** — Project conventions and commands for AI-assisted development

### Changed
- **UI layout system** — Introduce shared spacing constants and modernize the metadata/lyrics split for more consistent terminal layouts.
- **Artwork presentation** — Center artwork within its region, constrain it by both width and height, and preserve visible margins across terminal sizes.
- **Control spacing** — Distribute playback controls evenly with flexible layout constraints.
- **Renderer structure** — Split the main draw path into smaller section renderers for easier maintenance without changing the top-level TUI behavior.

## [0.1.0] - 2026-01-24

### Added

#### Phase 1: Core Foundation (Week 1-2)
- **TUI Framework**
  - Ratatui-based terminal interface with responsive layout
  - Multiple view support (Now Playing, Queue, Library, Help)
  - Vim-style keyboard navigation (hjkl, Space, etc.)
  - Status bar with playback information

- **Apple Music Integration**
  - AppleScript-based macOS integration
  - Complete playback control (play, pause, next, previous)
  - Volume control and mute functionality
  - Playback position and seeking
  - Repeat and shuffle mode control
  - Track metadata display (title, artist, album, duration)

- **Testing & Infrastructure**
  - Comprehensive unit tests with mockall
  - Integration tests for UI components
  - CI/CD pipeline with GitHub Actions
  - Code formatting and linting automation

#### Phase 2: UI Enhancement (Week 3-4)
- **Album Artwork Display**
  - Multi-format support (ASCII, Unicode, TrueColor)
  - Non-blocking background loading and caching
  - Automatic image processing and resizing
  - LRU cache for improved performance
  - Graceful fallback for missing artwork

- **UI Improvements**
  - Enhanced color scheme with theme support
  - Progress bar with visual feedback
  - Smooth animations and transitions
  - Responsive layout adaptation
  - Loading indicators

#### Phase 3: Lyrics Integration (Week 5-6)
- **LRC Lyrics Support**
  - Full LRC format parser with millisecond precision
  - Multi-timestamp support for chorus/repeated sections
  - Offset adjustment for sync correction
  - Real-time synchronized display

- **Multi-source Lyrics Fetching**
  - Local file priority (`~/Music/Lyrics/*.lrc`)
  - Netease Cloud Music API integration
  - Automatic song matching with fuzzy search
  - LRU cache for repeated queries
  - Graceful error handling

- **Lyrics Display**
  - Auto-scrolling synchronized view
  - Current line highlighting
  - Multi-line display with context
  - Fallback to plain text lyrics

### Technical Stack
- **Language**: Rust 1.75+
- **TUI**: Ratatui 0.30 with Crossterm 0.28
- **Async Runtime**: Tokio 1.35
- **Image Processing**: image 0.25, ratatui-image 10.0
- **Configuration**: Serde + TOML + Clap
- **HTTP Client**: reqwest 0.11 (rustls)
- **Error Handling**: anyhow + thiserror
- **Logging**: tracing + tracing-subscriber

### Known Limitations
- macOS only (requires Apple Music)
- Apple Music must be running
- Some features require macOS 10.15+
- Album artwork requires terminal with image support for best quality

### Documentation
- Comprehensive README (English + Chinese)
- Detailed PROJECT_SPEC.md (69KB technical documentation)
- LYRICS.md (lyrics system architecture)
- CONTRIBUTING.md (contribution guidelines)
- RELEASE.md (release process documentation)

---

## Future Releases

### [0.3.0] - Planned (Phase 4-5)

#### Phase 4: Advanced Features
- Playlist management and editing
- Music library browsing (albums, artists, songs)
- Advanced search functionality
- Favorites and rating system
- Recently played tracking

#### Phase 5: Plugin System
- Plugin architecture design
- Spotify integration
- VLC media player support
- Last.fm scrobbling
- Custom theme engine

### [1.0.0] - Planned (Phase 6)

#### Phase 6: Polish and Release
- Performance optimization
- Comprehensive documentation
- User onboarding improvements
- Crash reporting and analytics
- Homebrew core submission
- Stable API

---

## Release Types

- **Major** (x.0.0): Breaking changes, major new features
- **Minor** (0.x.0): New features, backwards compatible
- **Patch** (0.0.x): Bug fixes, minor improvements

---

[Unreleased]: https://github.com/juntaochi/amcli/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/juntaochi/amcli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/juntaochi/amcli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/juntaochi/amcli/releases/tag/v0.1.0
