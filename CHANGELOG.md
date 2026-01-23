# Changelog / 更新日志

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Homebrew distribution support
- Automated release workflow for macOS binaries (Intel and Apple Silicon)
- Comprehensive release documentation (RELEASE.md)

## [0.1.0] - TBD

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

### [0.2.0] - Planned (Phase 4-5)

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

[Unreleased]: https://github.com/juntaochi/amcli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/juntaochi/amcli/releases/tag/v0.1.0
