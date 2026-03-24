# Testing Patterns

**Analysis Date:** 2026-03-24

## Test Framework

**Runner:**
- `cargo test` (built-in Rust test framework)
- Async test support via `tokio-test` crate (`Cargo.toml` dev-dependency)
- No separate test config file -- uses Cargo defaults

**Assertion Library:**
- Standard `assert!`, `assert_eq!` macros
- No additional assertion crate

**Mocking:**
- `mockall` 0.12 (`Cargo.toml` dev-dependency) for auto-generating mocks from traits
- `mockall::predicate::eq()` for argument matching

**Run Commands:**
```bash
make test                      # Run all tests (cargo test)
cargo test                     # Run all tests
cargo test <name>              # Run a specific test by name
cargo test -- --nocapture      # Run tests with stdout visible
cargo test --all-features      # Run tests with all features (CI mode)
make verify                    # Full pipeline: fmt + clippy + test + build + docs
```

## Test File Organization

**Location:**
- Co-located with source code using `#[cfg(test)] mod tests { ... }` at the bottom of each file
- No separate `tests/` directory with integration tests (the `tests/` directory exists but contains no `.rs` files)

**Naming:**
- Test functions prefixed with `test_`: `test_play`, `test_get_volume`, `test_parse_simple`
- Test modules always named `tests`

**Files containing tests:**
- `src/player/apple_music.rs` -- 3 async tests for player commands
- `src/lyrics/parser.rs` -- 6 synchronous tests for LRC parsing
- `src/ui/mod.rs` -- 2 async tests for app initialization and UI rendering

## Test Structure

**Suite Organization:**
```rust
// At the bottom of any source file
#[cfg(test)]
mod tests {
    use super::*;
    // Additional test-specific imports

    // Helper functions (not marked #[test])
    fn mock_output(stdout: &str, success: bool) -> std::process::Output { ... }

    #[test]  // or #[tokio::test] for async
    fn test_descriptive_name() {
        // Arrange
        // Act
        // Assert
    }
}
```

**Patterns:**
- Setup: Inline within each test function. No shared setup/teardown hooks.
- No `#[fixture]` or `#[rstest]` usage.
- Async tests use `#[tokio::test]` attribute.
- Synchronous tests use standard `#[test]` attribute.

## Mocking

**Framework:** `mockall` 0.12

**Auto-mock generation pattern (preferred for osascript abstraction):**
```rust
// src/player/apple_music.rs

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn execute(&self, script: &str) -> Result<std::process::Output>;
}
```

This generates `MockCommandRunner` automatically at compile time (test builds only).

**Mock usage pattern:**
```rust
#[tokio::test]
async fn test_play() {
    let mut mock = MockCommandRunner::new();
    mock.expect_execute()
        .with(mockall::predicate::eq(
            r#"tell application "Music" to play"#,
        ))
        .times(1)
        .returning(|_| Ok(mock_output("", true)));

    let controller = AppleMusicController::with_runner(Box::new(mock));
    assert!(controller.play().await.is_ok());
}
```

**Test-only constructor pattern:**
```rust
// Guarded by #[cfg(test)] so it only exists in test builds
#[cfg(test)]
pub fn with_runner(runner: Box<dyn CommandRunner>) -> Self {
    Self {
        runner,
        artwork_cache: Mutex::new(LruCache::new(
            NonZeroUsize::new(20).expect("cache capacity must be non-zero"),
        )),
    }
}
```

**Manual mock pattern (for MediaPlayer trait in UI tests):**
```rust
// src/ui/mod.rs tests -- hand-written mock implementing full MediaPlayer trait
struct MockPlayer {
    volume: u8,
}

#[async_trait]
impl MediaPlayer for MockPlayer {
    async fn play(&self) -> Result<()> { Ok(()) }
    async fn pause(&self) -> Result<()> { Ok(()) }
    async fn toggle(&self) -> Result<()> { Ok(()) }
    // ... all trait methods implemented with stub returns
    async fn get_current_track(&self) -> Result<Option<Track>> {
        Ok(Some(Track {
            name: "Test Song".into(),
            artist: "Test Artist".into(),
            album: "Test Album".into(),
            duration: Duration::from_secs(300),
            position: Duration::from_secs(150),
        }))
    }
    async fn get_player_status(&self) -> Result<crate::player::PlayerStatus> {
        Ok(crate::player::PlayerStatus {
            track: Some(Track { ... }),
            volume: Some(self.volume),
            state: PlaybackState::Playing,
        })
    }
    // ...
}
```

**What to Mock:**
- External process execution (`osascript` commands via `CommandRunner` trait)
- The entire `MediaPlayer` trait when testing UI logic
- Mock data uses realistic but hardcoded values: `"Test Song"`, `"Test Artist"`, volume `70`

**What NOT to Mock:**
- Pure functions (LRC parsing, duration formatting, scroll text)
- In-memory data structures (LRU cache, config deserialization)

## Fixtures and Factories

**Test Data:**
```rust
// Helper function for creating mock process output
// Located in: src/player/apple_music.rs (inside #[cfg(test)] mod tests)
fn mock_output(stdout: &str, success: bool) -> std::process::Output {
    std::process::Output {
        status: ExitStatus::from_raw(if success { 0 } else { 1 }),
        stdout: stdout.as_bytes().to_vec(),
        stderr: if success { vec![] } else { b"error".to_vec() },
    }
}
```

```rust
// Inline LRC content for parser tests
// Located in: src/lyrics/parser.rs (inside #[cfg(test)] mod tests)
let lrc = "[00:12.34]Hello world";
let lrc = "[ti:Title]\n[ar:Artist]\n[00:01.00]Lyrics";
let lrc = "[offset:500]\n[00:01.00]Lyrics";
let lrc = "作词 : 周杰伦\n作曲 : 周杰伦\n[00:12.34]真正的歌词\n纯文本行\n[00:15.00]第二行";
```

**Location:**
- No dedicated fixtures directory
- All test data is inline within test functions or helper functions in the `#[cfg(test)]` module

## Coverage

**Requirements:** None enforced. No coverage thresholds or CI gates.

**View Coverage:**
```bash
# Install cargo-tarpaulin first: cargo install cargo-tarpaulin
cargo tarpaulin --all-features    # Generate coverage report
```

Note: No coverage tooling is configured in CI or in the project.

## Test Types

**Unit Tests:**
- Co-located `#[cfg(test)] mod tests` blocks
- Test individual functions and methods in isolation
- 3 test files with 11 total test functions

**Integration Tests:**
- No integration tests exist. The `tests/` directory is empty.
- The `test_ui_rendering` test in `src/ui/mod.rs` is the closest to an integration test -- it initializes `App` with a mock player, calls `update()`, and renders to a `TestBackend`

**E2E Tests:**
- Not used. The application requires macOS with Apple Music running, making E2E testing impractical in CI.

## Common Patterns

**Async Testing:**
```rust
// src/player/apple_music.rs
#[tokio::test]
async fn test_get_volume() {
    let mut mock = MockCommandRunner::new();
    mock.expect_execute()
        .with(mockall::predicate::eq(
            r#"tell application "Music" to return sound volume"#,
        ))
        .times(1)
        .returning(|_| Ok(mock_output("75", true)));

    let controller = AppleMusicController::with_runner(Box::new(mock));
    let volume = controller.get_volume().await.unwrap();
    assert_eq!(volume, 75);
}
```

**Synchronous Pure Function Testing:**
```rust
// src/lyrics/parser.rs
#[test]
fn test_parse_simple() {
    let lrc = "[00:12.34]Hello world";
    let lyrics = parse_lrc(lrc).unwrap();
    assert_eq!(lyrics.lines.len(), 1);
    assert_eq!(lyrics.lines[0].timestamp, Duration::from_millis(12340));
    assert_eq!(lyrics.lines[0].text, "Hello world");
}
```

**UI Rendering Test:**
```rust
// src/ui/mod.rs
#[tokio::test]
async fn test_ui_rendering() {
    let player = Box::new(MockPlayer { volume: 70 });
    let mut app = App::with_player(player).await.unwrap();
    app.update().await.unwrap();

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();
    let content = format!("{:?}", buffer).to_uppercase();
    assert!(content.contains("TEST"));
    assert!(content.contains("SONG"));
    assert!(content.contains("ARTIST"));
}
```

## Test Coverage Gaps

**Untested modules (no `#[cfg(test)]` block):**

| Module | File | Risk | Priority |
|--------|------|------|----------|
| Config loading/saving | `src/config/mod.rs` | Config corruption, wrong defaults | Medium |
| Lyrics manager orchestration | `src/lyrics/mod.rs` | Provider fallback chain, cache behavior | High |
| Lyrics providers (Netease) | `src/lyrics/netease.rs` | API parsing failures | Medium |
| Lyrics providers (LRCLIB) | `src/lyrics/lrclib.rs` | Two-stage lookup logic, response parsing | Medium |
| Lyrics providers (Local) | `src/lyrics/local.rs` | File pattern matching | Low |
| Artwork manager | `src/artwork/mod.rs` | Duotone algorithm, pixelation, cache integration | Medium |
| Artwork cache | `src/artwork/cache.rs` | Disk I/O, cache eviction, hash collisions | Medium |
| Artwork converter | `src/artwork/converter.rs` | Protocol selection logic | Low |
| Settings menu | `src/ui/settings.rs` | Navigation logic, item skip behavior | Low |
| Main event loop | `src/main.rs` | Terminal setup/teardown, key dispatch | Low |

**Specific gaps in tested modules:**

- `src/player/apple_music.rs`: Only tests `play()`, `get_volume()`, `get_current_track()`. Missing tests for:
  - `pause()`, `toggle()`, `next()`, `previous()`, `stop()`
  - `get_playback_state()` with all three states
  - `get_player_status()` (the optimized combined query)
  - `set_volume()`, `seek()`, `set_shuffle()`, `set_repeat()`
  - `get_artwork_url()` (includes HTTP call and LRU cache)
  - Error paths (failed AppleScript execution, invalid output format)

- `src/ui/mod.rs`: Only tests initialization and basic rendering. Missing tests for:
  - `volume_up()`, `volume_down()`, `toggle_mute()` state transitions
  - `cycle_repeat()` state cycling
  - `settings_select()` configuration persistence
  - `update()` with track changes, lyrics loading, artwork loading
  - `scroll_text()` helper function (has performance comment but no tests)

- `src/lyrics/parser.rs`: Good coverage of LRC parsing. Missing edge cases:
  - Empty input
  - Malformed timestamps
  - Lines with timestamps but empty text (already filtered, but not explicitly tested)

**Recommendations for new test additions:**
1. Add `#[cfg_attr(test, automock)]` to `LyricsProvider` trait and test `LyricsManager` provider chain
2. Test `scroll_text()` with edge cases (empty string, exact width, wider than width)
3. Test `Config::default()` produces valid TOML roundtrip
4. Test error paths in `AppleMusicController` (failed scripts, malformed output)
5. Test `Lyrics::find_index()` with various position values

---

*Testing analysis: 2026-03-24*
