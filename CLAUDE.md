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
cargo test <name>   # Run a specific test
cargo test -- --nocapture  # Run tests with stdout visible
```

CI runs on macOS and checks: `cargo check`, `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo build --release` (all with `--all-features`).

## Architecture

**Entry point** (`src/main.rs`): Sets up Tokio runtime, Crossterm terminal (raw mode, alternate screen), and runs a 50ms poll event loop with 500ms state refresh.

**Key modules:**

- **`src/ui/mod.rs`** — `App` struct is the central state hub. Contains all rendering logic (Ratatui), keyboard handling, and application state. Has a modal settings menu (`src/ui/settings.rs`). Supports 6 color themes.
- **`src/player/`** — `MediaPlayer` trait defines the player abstraction. `AppleMusicController` implements it by executing AppleScript via `osascript` CLI. Uses `CommandRunner` trait for testability (mocked with `mockall`).
- **`src/lyrics/`** — Multi-provider lyrics system with priority: local files → Netease → LRCLIB. Parses LRC format with timestamp regex. `LyricsManager` orchestrates providers via the `LyricsProvider` trait.
- **`src/artwork/`** — Album art with LRU caching. Converts images to terminal protocols (Sixel, Kitty, halfblocks) via `ratatui-image`.
- **`src/config/`** — TOML-based config with serde. Supports language (en/ja), theme selection, artwork mode, mosaic effects.

## Conventions

- **AppleScript strings**: Use raw string literals (`r#"..."#`) for inline AppleScript.
- **Async traits**: Use `#[async_trait]` for traits with async methods.
- **Error handling**: `anyhow::Result` for application logic, `thiserror` for defining error types in modules.
- **Never block the UI draw loop** with I/O (network, AppleScript calls).
- **Commit messages**: `<type>(<scope>): <subject>` format (e.g., `feat(player): add volume control`, `fix(ui): resolve layout overflow`).
- **macOS only**: Requires Apple Music app installed. Album art display depends on terminal protocol support.
