# AMCLI - Apple Music Command Line Interface

<div align="center">

**A Rust-powered terminal controller for Apple Music on macOS**

[![Rust Version](https://img.shields.io/badge/Rust-1.75+-dea584?style=flat&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Developing-green.svg)](PROJECT_SPEC.md)

English | [中文](README.zh-CN.md)

<img src="assets/amcli-cyan-vfd-lyrics.png" alt="AMCLI cyan VFD theme with album artwork, playback controls, and synchronized lyrics" width="100%" />

</div>

## Overview

**AMCLI** is a Terminal User Interface (TUI) application for controlling Apple Music on macOS. It brings playback controls, album artwork, synchronized lyrics, and terminal-first customization into one focused interface.

- Complete playback controls for Apple Music
- 8-bit style album artwork rendering
- Real-time synchronized lyrics
- Lightweight Rust implementation
- Vim-style keybindings
- Theme and mosaic display customization

## Feature Highlights

### Media Control

- Play, pause, skip, and go back
- Adjust volume and mute
- Seek forward and backward
- Cycle repeat mode
- Track progress with precise position display

### Visual Experience

- ASCII, Unicode, and TrueColor album artwork
- Current-track artwork first, with online lookup as a fallback
- Non-blocking background artwork loading
- Automatic retry after transient artwork failures
- Six built-in themes:
  - `AMBER VFD`
  - `GREEN VFD`
  - `CYAN VFD`
  - `RED ALERT`
  - `MODERN`
  - `CLEAN`
- Optional mosaic mode for pixelated artwork
- Responsive terminal layout

<p align="center">
  <img src="assets/screenshot_2.png" alt="AMCLI amber VFD theme screenshot" width="49%" />
  <img src="assets/screenshot_3.png" alt="AMCLI alternate terminal theme screenshot" width="49%" />
</p>

### Synchronized Lyrics

- Millisecond-precision LRC playback sync
- Concurrent LRCLIB and Netease lookup on first fetch
- Candidate matching by title, artist, album, and duration
- Session-level source preference based on the faster provider
- LRU cache for repeated lyrics queries
- Centered auto-scrolling lyrics view
- LRC parser support for multiple timestamps and offset adjustments

<p align="center">
  <img src="assets/amcli-modern-dark-lyrics.png" alt="AMCLI modern dark theme with synchronized lyrics" width="49%" />
  <img src="assets/amcli-clean-light-lyrics.png" alt="AMCLI clean light theme with synchronized lyrics" width="49%" />
</p>

### Terminal-First Workflow

AMCLI is designed to stay useful inside a real terminal workspace: compact controls, readable metadata, predictable keybindings, and visual feedback that works alongside other command-line tools.

<p align="center">
  <img src="assets/amcli-terminal-dashboard.png" alt="AMCLI running inside a terminal dashboard workflow" width="100%" />
</p>

### Configuration & Customization

- Interface language: English / Japanese
- Settings menu with `s`
- Live theme switching with `t`
- Mosaic mode toggle
- Configuration file at `~/.config/amcli/config.toml`

## Quick Start

> [!TIP]
> Project status: Phases 1-3 are complete, covering the core TUI, album artwork, and LRCLIB + Netease lyrics integration.

### Installation

**Option 1: Build from Source**

```bash
git clone https://github.com/juntaochi/amcli.git
cd amcli
cargo build --release
cargo install --path .
```

**Option 2: Download a Release**

Download a pre-built binary from the [Releases](https://github.com/juntaochi/amcli/releases) page.

**Option 3: Homebrew tap**

If the Homebrew tap has been published for the current release:

```bash
brew tap juntaochi/tap
brew install amcli
```

Maintainers can use the template in [homebrew/amcli.rb](homebrew/amcli.rb) when preparing tap updates.

### Usage

```bash
amcli
amcli --help
amcli --config ~/.config/amcli/config.toml
```

### Configuration

AMCLI creates `~/.config/amcli/config.toml` on first run.

```toml
[general]
language = "en"  # "en" or "jp"

[artwork]
enabled = true
cache_size = 100
mode = "auto"     # auto, ascii, blocks, truecolor
mosaic = true

[ui]
color_theme = "default"
show_help_on_start = true
```

## Keybindings

| Action | Key |
| --- | --- |
| Play / Pause | `Space` |
| Next Track | `]` |
| Previous Track | `[` |
| Volume Up | `=` / `+` |
| Volume Down | `-` / `_` |
| Mute | `m` |
| Seek Forward / Backward | `.` / `,` or `→` / `←` |
| Navigate | `h` / `j` / `k` / `l` or arrow keys |
| Cycle Repeat Mode | `r` |
| Theme Switch | `t` |
| Settings | `s` |
| Help | `?` |
| Quit | `q` |

See [PROJECT_SPEC.md](PROJECT_SPEC.md) for the full design notes and planned keyboard system.

## Documentation

- [PROJECT_SPEC.md](PROJECT_SPEC.md) - Full project specification, architecture, feature design, and roadmap
- [LYRICS.md](LYRICS.md) - Lyrics system internals, provider integration, parsing, and sync behavior
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guide

## Development Roadmap

1. Phase 1: Core foundation, TUI framework, and Apple Music control
2. Phase 2: UI enhancements and album artwork
3. Phase 3: Lyrics integration
4. Phase 4: Playlist and music library features
5. Phase 5: Plugin system and multi-player support
6. Phase 6: Polish and release

## Tech Stack

- **Language:** Rust 1.75+
- **TUI Framework:** [Ratatui](https://github.com/ratatui-org/ratatui)
- **Terminal Backend:** [Crossterm](https://github.com/crossterm-rs/crossterm)
- **Async Runtime:** [Tokio](https://tokio.rs/)
- **macOS Integration:** AppleScript / osascript
- **Configuration:** Serde + TOML + Clap

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and workflow details.

## License

AMCLI is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [go-musicfox](https://github.com/go-musicfox/go-musicfox) for design inspiration
- [Ratatui](https://ratatui.rs/) for the TUI framework

<div align="center">

**Made for music lovers and terminal enthusiasts**

</div>
