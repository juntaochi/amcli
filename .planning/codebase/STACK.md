# Technology Stack

**Analysis Date:** 2026-03-24

## Languages

**Primary:**
- Rust (Edition 2021) - All application code

**Secondary:**
- AppleScript - Inline scripts executed via `osascript` for Apple Music IPC (`src/player/apple_music.rs`)
- Bash - Build verification script (`scripts/verify.sh`)
- YAML - GitHub Actions CI/CD (`.github/workflows/`)

## Runtime

**Environment:**
- macOS only (requires Apple Music app installed)
- Tokio async runtime (`tokio 1.35` with `features = ["full"]`)
- Terminal raw mode via Crossterm (alternate screen, mouse capture)

**Package Manager:**
- Cargo (Rust standard)
- Lockfile: `Cargo.lock` present and committed

## Frameworks

**Core:**
- `ratatui 0.30` - Terminal UI framework (rendering, layout, widgets)
- `crossterm 0.28` - Terminal backend (raw mode, events, alternate screen)
- `tokio 1.35` - Async runtime with full features (process spawning, timers, fs, task)

**Testing:**
- `mockall 0.12` - Mock generation for trait-based testing (dev-dependency)
- `tokio-test 0.4` - Async test utilities (dev-dependency)
- Built-in `cargo test` runner (no custom test framework)

**Build/Dev:**
- Cargo with Makefile wrapper (`Makefile`)
- Clippy for linting (`cargo clippy -- -D warnings`)
- Rustfmt for formatting (`cargo fmt`)
- Full verification script: `scripts/verify.sh` (fmt check, clippy, test, build, doc)

## Key Dependencies

**Critical (core functionality):**
- `ratatui 0.30` - All UI rendering (`src/ui/mod.rs`, `src/ui/settings.rs`)
- `crossterm 0.28` - Terminal I/O, keyboard/mouse events (`src/main.rs`)
- `tokio 1.35` - Async runtime, process spawning for osascript, file I/O, timers
- `reqwest 0.11` (with `json`, `rustls-tls`, no default features) - HTTP client for lyrics APIs and artwork URLs (`src/lyrics/lrclib.rs`, `src/lyrics/netease.rs`, `src/artwork/mod.rs`)
- `image 0.25` - Album artwork image loading, processing, duotone/pixelation effects (`src/artwork/mod.rs`)
- `ratatui-image 10.0` (no default features) - Terminal image protocol rendering: Sixel, Kitty, halfblocks (`src/artwork/converter.rs`)

**Infrastructure:**
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

**Utilities:**
- `regex 1.10` - LRC timestamp parsing (`src/lyrics/parser.rs`)
- `lazy_static 1.4` - Compiled regex singletons (`src/lyrics/parser.rs`)
- `sha2 0.10` - URL hashing for disk cache keys (`src/artwork/cache.rs`)
- `urlencoding 2.1` - URL parameter encoding for API calls
- `chrono 0.4` - Date/time utilities
- `unicode-width 0.1` - CJK character width for UI layout
- `rgb 0.8` - RGB color utilities
- `futures 0.3` - Future combinators
- `config 0.13` - Configuration framework (declared but primarily using custom TOML loading)

**UI Widgets (Ratatui ecosystem):**
- `throbber-widgets-tui 0.10` - Loading spinner widgets
- `tui-big-text 0.8` - Large text rendering

## Configuration

**User Config:**
- TOML format at `~/.config/amcli/config.toml` (via `dirs::config_dir()`)
- Auto-created with defaults on first run
- Sections: `artwork` (enabled, cache_size, mode, album, mosaic), `ui` (color_theme, show_help_on_start), `general` (language)
- Schema defined in `src/config/mod.rs` via serde derive

**Build Config:**
- `Cargo.toml` - Package manifest, dependencies, release profile
- No `rust-toolchain.toml` - uses stable toolchain via CI (`dtolnay/rust-toolchain@stable`)

**Release Profile (`Cargo.toml`):**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

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

**CI Pipeline (`.github/workflows/ci.yml`):**
- Runs on: `macos-latest`
- Triggers: push/PR to `main` and `develop`
- Jobs (parallel): `cargo check`, `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build --release` (all with `--all-features`)
- Toolchain: `dtolnay/rust-toolchain@stable`

**Release Pipeline (`.github/workflows/release.yml`):**
- Triggers: git tag `v*.*.*` or manual dispatch
- Builds for both `x86_64-apple-darwin` (macos-13) and `aarch64-apple-darwin` (macos-14)
- Creates GitHub Release with tar.gz archives and SHA256 checksums
- Dispatches Homebrew formula update (`brew tap juntaochi/tap`)

**PR Workflows (`.github/workflows/pr-validate-merge.yml`, `.github/workflows/pr-conflict-scan.yml`):**
- PR validation and conflict scanning automation

## Platform Requirements

**Development:**
- macOS (Apple Music app must be installed for integration testing)
- Rust stable toolchain
- Terminal with optional Sixel/Kitty support for album art

**Production:**
- macOS only (AppleScript IPC with Music.app)
- Binary targets: `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Distribution: Homebrew tap (`juntaochi/tap`) or direct binary download
- No external runtime dependencies (statically linked with LTO)

---

*Stack analysis: 2026-03-24*
