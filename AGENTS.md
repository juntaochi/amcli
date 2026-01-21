# AGENTS.md - Development Guide for AI Coding Agents

**Project**: AMCLI - Apple Music Command Line Interface  
**Language**: Rust 2021  
**Platform**: macOS only (requires AppleScript)  
**Status**: Active development (Phase 1)

---

## Quick Reference Commands

### Build & Run
```bash
# Build project
cargo build                 # Debug build
cargo build --release       # Release build (optimized)

# Run application
cargo run                   # Run in debug mode
make run                    # Same as cargo run

# Install locally
make install                # Installs to ~/.cargo/bin
```

### Testing
```bash
# Run all tests
cargo test                  # Run all tests
make test                   # Same as cargo test

# Run specific test
cargo test test_name        # Run test containing "test_name"
cargo test --test integration_test  # Run specific integration test

# Run with output visible
cargo test -- --nocapture   # Show println! output

# Run async tests (using tokio::test)
cargo test -- --test-threads=1  # Run tests sequentially if needed
```

### Code Quality
```bash
# Format code (REQUIRED before commit)
cargo fmt                   # Format all code
cargo fmt --check           # Check formatting without changing files
make fmt                    # Same as cargo fmt

# Lint (REQUIRED before commit)
cargo clippy -- -D warnings # Run clippy, fail on warnings
make lint                   # Same as cargo clippy

# Check without building
cargo check                 # Fast compilation check
cargo check --all-features  # Check with all features enabled

# Full verification pipeline
make verify                 # Runs ./scripts/verify.sh (fmt, clippy, test, build)
```

---

## Code Style Guidelines

### 1. Import Organization

**Order**: External crates → Standard library → Local modules → Crate imports

```rust
// External crates first
use anyhow::{anyhow, Result, Context};
use async_trait::async_trait;

// Multi-item imports grouped with braces
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

// Standard library
use std::io;
use std::process::Command;
use std::time::Duration;

// Module declarations
mod player;
mod ui;

// Crate-internal imports
use crate::player::{MediaPlayer, Track};
```

### 2. Type Definitions

**Enums** - Always derive Debug, Clone, Copy (if possible), PartialEq:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Playing,    // PascalCase variants
    Paused,
    Stopped,
}
```

**Structs** - Derive Debug, Clone at minimum:
```rust
#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub artist: String,
    pub duration: Duration,
}
```

**Traits** - Use async_trait for async methods, require Send + Sync:
```rust
#[async_trait]
pub trait MediaPlayer: Send + Sync {
    async fn play(&self) -> Result<()>;
    async fn get_volume(&self) -> Result<u8>;
}
```

### 3. Error Handling

**Always use `anyhow::Result` for application code**:
```rust
use anyhow::{anyhow, Result, Context};

// Function signatures
async fn do_something() -> Result<()> {
    operation().context("Failed to do something")?;
    Ok(())
}

// Creating errors
return Err(anyhow!("Invalid state: {}", state));

// Propagation (preferred)
let output = Command::new("osascript").output()?;

// Safe fallbacks
let volume = player.get_volume().await.unwrap_or(50);
```

**Use `thiserror` for library error types** (see CONTRIBUTING.md):
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("AppleScript execution failed: {0}")]
    ScriptError(String),
}
```

### 4. Async Patterns

**Main function**:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    // ... application logic
    Ok(())
}
```

**Trait implementations**:
```rust
#[async_trait]
impl MediaPlayer for AppleMusicController {
    async fn play(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to play"#)?;
        Ok(())
    }
}
```

**Async tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_playback() {
        let player = AppleMusicController::new();
        assert!(player.play().await.is_ok());
    }
}
```

### 5. Naming Conventions

| Type | Convention | Examples |
|------|-----------|----------|
| Files | `snake_case.rs` | `apple_music.rs`, `mod.rs` |
| Modules | `snake_case` | `mod player`, `mod ui` |
| Structs/Enums | `PascalCase` | `Track`, `PlaybackState` |
| Traits | `PascalCase` | `MediaPlayer` |
| Functions/Methods | `snake_case` | `toggle_playback`, `get_volume` |
| Variables | `snake_case` | `current_track`, `is_muted` |
| Constants | `SCREAMING_SNAKE` | `MAX_VOLUME`, `DEFAULT_TIMEOUT` |
| Booleans | Prefix `is_`, `has_`, `should_` | `is_muted`, `has_track` |

### 6. Code Organization

**Struct implementation order**:
```rust
impl MyStruct {
    // 1. Constructor(s)
    pub fn new() -> Self { /* ... */ }
    
    // 2. Public methods
    pub async fn command(&mut self) -> Result<()> { /* ... */ }
    pub fn query(&self) -> u8 { /* ... */ }
    
    // 3. Private helper methods
    fn helper(&self) -> Result<String> { /* ... */ }
}

// 4. Trait implementations (separate block)
#[async_trait]
impl SomeTrait for MyStruct {
    async fn trait_method(&self) -> Result<()> { /* ... */ }
}
```

### 7. Pattern Matching

**Match expressions**:
```rust
match key.code {
    // Single action per line
    KeyCode::Char('q') => return Ok(()),
    
    // Multi-pattern with |
    KeyCode::Char('=') | KeyCode::Char('+') => volume_up().await?,
    
    // Grouped by category (use comments)
    // Playback control
    KeyCode::Char(' ') => toggle().await?,
    KeyCode::Char('[') => previous().await?,
    KeyCode::Char(']') => next().await?,
    
    // Default case at end
    _ => {}
}

// String matching
match state.as_str() {
    "playing" => Ok(PlaybackState::Playing),
    "paused" => Ok(PlaybackState::Paused),
    _ => Err(anyhow!("Unknown state: {}", state)),
}
```

### 8. String Handling

**Use raw strings for AppleScript**:
```rust
// Simple scripts
self.execute_script(r#"tell application "Music" to play"#)?;

// Multi-line scripts
let script = r#"
    tell application "Music"
        if player state is not stopped then
            return name of current track
        end if
    end tell
"#;

// With variables
let script = format!(
    r#"tell application "Music" to set sound volume to {}"#,
    volume
);
```

### 9. Type Conversions & Math

**Safe numeric operations**:
```rust
// Saturating subtraction
self.volume = self.volume.saturating_sub(5);

// Clamping with min/max
self.volume = (self.volume + 5).min(100);
value.max(0).min(100)

// Duration conversions
Duration::from_secs_f64(seconds_float)
duration.as_secs()        // u64
duration.as_secs_f64()    // f64

// Parsing with error propagation
let volume: u8 = volume_str.parse()?;
```

### 10. Attributes

**Allow dead code during development**:
```rust
#[allow(dead_code)]
pub enum FeatureInDevelopment {
    NotYetUsed,
}

#[allow(dead_code)]
async fn planned_feature(&self) -> Result<()> {
    // Will be used in future phase
}
```

**Derive macros (standard combinations)**:
```rust
// Data structures
#[derive(Debug, Clone)]
pub struct DataStruct { /* ... */ }

// Simple enums (copyable)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimpleEnum { /* ... */ }

// Configuration types
#[derive(Debug, Clone, Deserialize)]
pub struct Config { /* ... */ }
```

### 11. Comments & Documentation

**File headers** - Simple path comment:
```rust
// src/player/apple_music.rs
```

**Inline comments** - High-level steps only:
```rust
// Setup terminal
enable_raw_mode()?;

// Create app and run it
let app = App::new().await?;

// Restore terminal
disable_raw_mode()?;
```

**Doc comments** - For public APIs (see CONTRIBUTING.md):
```rust
/// Plays the current track
///
/// # Errors
///
/// Returns error if AppleScript execution fails
///
/// # Examples
///
/// ```no_run
/// let player = AppleMusicController::new();
/// player.play().await?;
/// ```
pub async fn play(&self) -> Result<()> {
    // ...
}
```

---

## Module Structure

```
src/
├── main.rs              # Entry point, minimal logic
├── player/
│   ├── mod.rs          # Trait definitions (MediaPlayer, Track, etc.)
│   └── apple_music.rs  # AppleScript implementation
├── ui/
│   └── mod.rs          # App state + Ratatui rendering
├── config/
│   └── mod.rs          # Configuration management (TOML)
├── lyrics/
│   └── mod.rs          # Lyrics fetching (Netease, Musixmatch, etc.)
└── artwork/
    └── mod.rs          # Album art processing (ASCII/8-bit)
```

**Pattern**: `mod.rs` defines public interfaces, other files implement them.

---

## Testing Strategy

### Unit Tests (in same file as code)
```rust
// src/player/apple_music.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_formatting() {
        // Test non-async logic
    }
    
    #[tokio::test]
    async fn test_playback_control() {
        // Test async methods
    }
}
```

### Integration Tests (tests/ directory)
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_end_to_end_playback() {
    // Integration test across modules
}
```

**Run commands**:
```bash
cargo test                      # All tests
cargo test test_playback        # Specific test name
cargo test --test integration   # Specific integration test file
cargo test -- --nocapture       # Show output
```

---

## Dependencies

**Core frameworks**:
- `ratatui` (0.26) - TUI framework
- `crossterm` (0.27) - Terminal backend
- `tokio` (1.35, full) - Async runtime
- `async-trait` (0.1) - Async trait support

**Error handling**:
- `anyhow` (1.0) - Application errors
- `thiserror` (1.0) - Library error types

**Serialization**:
- `serde` (1.0, derive) - Serialization framework
- `toml` (0.8) - Config file format
- `config` (0.13) - Config management

**Utilities**:
- `tracing` / `tracing-subscriber` - Logging
- `clap` (4.4, derive) - CLI argument parsing
- `reqwest` (0.11, rustls-tls) - HTTP client for lyrics

**Image processing**:
- `image` (0.24) - Image manipulation
- `rgb` (0.8) - Color handling

---

## Git Workflow

### Commit Message Format
```
<type>(<scope>): <subject>

Examples:
feat(player): add AppleScript volume control
fix(ui): resolve progress bar overflow
docs: update AGENTS.md with testing guide
refactor(lyrics): extract Netease client to separate module
test(player): add unit tests for seek functionality
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Pre-commit Checklist
```bash
# REQUIRED before every commit:
cargo fmt                   # Format code
cargo clippy -- -D warnings # Lint without warnings
cargo test                  # All tests must pass
cargo build                 # Build must succeed
```

---

## Common Patterns

### AppleScript Execution
```rust
fn execute_script(&self, script: &str) -> Result<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!(
            "AppleScript failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
```

### State Management in UI
```rust
pub struct App {
    // Dependencies
    player: Box<dyn MediaPlayer>,
    
    // State
    current_track: Option<Track>,
    volume: u8,
    
    // Flags
    is_muted: bool,
    show_help: bool,
}

impl App {
    // Periodic state update
    pub async fn update(&mut self) -> Result<()> {
        self.current_track = self.player.get_current_track().await?;
        self.volume = self.player.get_volume().await.unwrap_or(self.volume);
        Ok(())
    }
}
```

### Event Loop Pattern
```rust
loop {
    terminal.draw(|f| ui::draw(f, &app))?;
    
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                _ => { /* handle keys */ }
            }
        }
    }
    
    app.update().await?;
}
```

---

## Known Constraints

1. **macOS only** - AppleScript requires macOS
2. **Apple Music required** - No mock implementations in production
3. **Single-threaded UI** - Ratatui rendering is synchronous
4. **Manual testing** - AppleScript interactions can't be fully unit tested

---

## Resources

- **PROJECT_SPEC.md** - Complete technical specification
- **CONTRIBUTING.md** - Extended code standards and workflow
- **TODO.md** - Development task list with priorities
- **AI_READY.md** - AI agent preparation checklist

---

**Last Updated**: 2026-01-21  
**Maintained By**: Development team + AI agents
