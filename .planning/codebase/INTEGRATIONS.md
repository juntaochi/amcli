# External Integrations

**Analysis Date:** 2026-03-24

## APIs & External Services

### Apple Music via AppleScript/osascript

**Purpose:** Primary player control and metadata retrieval. All interaction with the Music.app happens through AppleScript commands executed via the `osascript` CLI.

**Implementation:** `src/player/apple_music.rs`

**Mechanism:**
- `OsascriptRunner` spawns `tokio::process::Command::new("osascript")` with `-e` flag and inline AppleScript
- Each command is an async process spawn (~100-200ms overhead per call)
- The `AppleMusicController` struct implements the `MediaPlayer` trait

**Key AppleScript Commands:**
| Operation | Script |
|-----------|--------|
| Play/Pause/Stop | `tell application "Music" to play/pause/stop` |
| Toggle | `tell application "Music" to playpause` |
| Next/Previous | `tell application "Music" to next track / previous track` |
| Get volume | `tell application "Music" to return sound volume` |
| Set volume | `tell application "Music" to set sound volume to {n}` |
| Seek | `tell application "Music" to set player position to (player position + {n})` |
| Shuffle | `tell application "Music" to set shuffle enabled to {bool}` |
| Repeat | `tell application "Music" to set song repeat to {off/one/all}` |
| Get track info | Multi-field query returning `name\|artist\|album\|duration\|position` |

**Optimized Status Call:**
- `get_player_status()` combines playback state, volume, and track metadata into a single AppleScript execution
- Uses `:::BOLT_SPLIT:::` delimiter to pack multiple values into one response
- Reduces process spawns from 3 to 1 per update cycle (called every 500ms)

**Testability:**
- `CommandRunner` trait abstracts osascript execution
- `MockCommandRunner` (via `mockall`) allows unit testing without Apple Music
- Test constructor: `AppleMusicController::with_runner(Box::new(mock))`

**Error Handling:**
- Non-zero exit status returns `anyhow!("AppleScript failed: {stderr}")`
- Empty result from track query returns `Ok(None)` (stopped state)

### iTunes Search API (Artwork URLs)

**Purpose:** Fetch album artwork URLs for the currently playing track.

**Implementation:** `src/player/apple_music.rs` method `get_artwork_url()`

**Endpoint:**
```
GET https://itunes.apple.com/search?term={artist}+{track_name}&entity=song&limit=1
```

**Details:**
- HTTP client: `reqwest::get()` (simple one-shot, no persistent client)
- Timeout: 3 seconds (via `tokio::time::timeout`)
- Response: JSON parsed with `serde_json::Value`
- Extracts `results[0].artworkUrl100` and replaces `100x100bb` with `600x600bb` for high-res
- No authentication required

**Caching:**
- In-memory LRU cache (capacity 20) keyed by `"{artist}|{track_name}"`
- Stored on `AppleMusicController` behind `Mutex<LruCache<String, Option<String>>>`
- Mutex uses `unwrap_or_else(|e| e.into_inner())` to recover from poison

### LRCLIB API (Lyrics)

**Purpose:** Fetch synchronized (timestamped) lyrics in LRC format.

**Implementation:** `src/lyrics/lrclib.rs`

**Provider Priority:** 5 (higher than Netease; local file provider removed in v0.3.0)

**Endpoints:**
```
# Stage 1: Precise match (preferred)
GET https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}&duration={}

# Stage 2: Loose match (fallback)
GET https://lrclib.net/api/get?artist_name={}&track_name={}
```

**Details:**
- HTTP client: Persistent `reqwest::Client` with 5-second timeout
- Custom headers: `User-Agent: AMCLI v1.0.0 (https://github.com/juntaochi/amcli)` and `Lrclib-Client: AMCLI v1.0.0`
- Two-stage lookup: first with album name + duration, then loose match
- Prefers `syncedLyrics` field, falls back to `plainLyrics`
- No authentication required
- Response parsed as `serde_json::Value`, then LRC content parsed via `parse_lrc()` in `src/lyrics/parser.rs`

### Netease Cloud Music API (Lyrics)

**Purpose:** Fetch synchronized lyrics, particularly strong for CJK (Chinese/Japanese/Korean) music.

**Implementation:** `src/lyrics/netease.rs`

**Provider Priority:** 10 (fallback/lower priority than LRCLIB)

**Endpoints:**
```
# Step 1: Search for song ID
GET https://music.163.com/api/cloudsearch/pc?s={query}&type=1&limit=1

# Step 2: Fetch lyrics by song ID
GET https://music.163.com/api/song/lyric?id={song_id}&lv=-1&kv=-1&tv=-1
```

**Details:**
- HTTP client: Persistent `reqwest::Client` with 5-second timeout
- User-Agent mimics Chrome browser: `Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)...`
- Two-step process: search for song ID, then fetch lyrics by ID
- Extracts `lrc.lyric` field from response JSON
- No authentication required
- LRC content parsed via shared `parse_lrc()` function

### Lyrics Provider System

**Architecture:** `src/lyrics/provider.rs` defines the `LyricsProvider` trait.

**Trait:**
```rust
#[async_trait]
pub trait LyricsProvider: Send + Sync {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>>;
    fn priority(&self) -> u8;   // Lower = higher priority
    fn name(&self) -> &'static str;
}
```

**Provider Order:**
1. `LrclibProvider` (priority 5) - `src/lyrics/lrclib.rs`
2. `NeteaseProvider` (priority 10) - `src/lyrics/netease.rs`

**Orchestration:** `LyricsManager` in `src/lyrics/mod.rs`:
- Before calibration, races online providers concurrently and returns the first lyrics hit
- Stores a clear winner as the session primary, then tries primary-first with fallback
- Uses a 12-second provider timeout (`PROVIDER_TIMEOUT`)
- LRU cache (capacity 20 in default app setup) keyed by version-aware track metadata
- Caches successful hits only; provider errors and misses are not cached as permanent misses
- Distinguishes reachable no-match from all-providers-unreachable so UI can show no lyrics vs no signal

**Local Lyrics Provider:**
- Removed in v0.3.0. Reintroducing local LRC support would require a new provider implementation and explicit registration.

### LRC Parser

**Implementation:** `src/lyrics/parser.rs`

**Capabilities:**
- Parses `[mm:ss.xx]` and `[mm:ss.xxx]` timestamp formats
- Handles multiple timestamps per line (e.g., `[00:12.34][00:15.00]Repeated line`)
- Extracts metadata tags (e.g., `[ti:Title]`, `[ar:Artist]`)
- Applies `[offset:N]` adjustment (positive shifts forward, negative shifts backward)
- Filters out non-timestamped lines
- Sorts output by timestamp

## Terminal Image Protocols

**Purpose:** Render album artwork in the terminal using the best available protocol.

**Implementation:** `src/artwork/converter.rs`

**Protocols (via `ratatui-image 10.0`):**
- **Auto** (default): Queries terminal for best protocol via `Picker::from_query_stdio()`, falls back to halfblocks
- **Sixel**: Queries terminal capability, falls back to halfblocks
- **Halfblocks**: Unicode half-block characters (universally supported)
- **Kitty**: Supported via auto-detection (Kitty graphics protocol)

**Zellij Detection:**
- Checks `ZELLIJ` environment variable
- Forces halfblocks in Zellij (multiplexers break graphics protocols)

**Image Processing Pipeline (`src/artwork/mod.rs`):**
1. Fetch artwork bytes via `reqwest::get(url)`
2. Load into `DynamicImage` via `image::load_from_memory()`
3. Apply duotone theme (for retro themes only): grayscale conversion, luminance-based color mapping
4. Optionally apply pixelation/mosaic effect (8x reduction, nearest-neighbor scaling)
5. Cache processed image in memory LRU (capacity 100)
6. Convert to terminal protocol via `Picker::new_resize_protocol()`

## Data Storage

**Databases:**
- None (no database)

**File Storage:**
- User config: `~/.config/amcli/config.toml` (via `dirs::config_dir()`)
- Artwork disk cache: `{cache_dir}/{sha256_hash}.png` (implemented in `src/artwork/cache.rs` but `insert_async`/`get_async` are currently `#[allow(dead_code)]` -- disk cache not actively used)

**Caching (all in-memory LRU):**
- Artwork URL cache: 20 entries on `AppleMusicController` (`src/player/apple_music.rs`)
- Artwork image cache: 100 entries on `ArtworkCache` (`src/artwork/cache.rs`)
- Lyrics cache: Configurable capacity on `LyricsManager` (`src/lyrics/mod.rs`)

## Authentication & Identity

**Auth Provider:** None
- All external APIs are unauthenticated public endpoints
- Apple Music control via local osascript (no API key)

## Monitoring & Observability

**Logging:**
- Framework: `tracing` + `tracing-subscriber` with `env-filter`
- Initialized in `src/main.rs` via `tracing_subscriber::fmt::init()`
- Usage: `tracing::debug!()` and `tracing::info!()` throughout lyrics and artwork modules
- Controlled via `RUST_LOG` environment variable

**Error Tracking:**
- None (errors logged to tracing, displayed in terminal on exit)

## Webhooks & Callbacks

**Incoming:** None
**Outgoing:** None

## Network Dependencies Summary

| Service | Base URL | Auth | Timeout | Used In |
|---------|----------|------|---------|---------|
| iTunes Search API | `https://itunes.apple.com/search` | None | 3s | `src/player/apple_music.rs` |
| LRCLIB | `https://lrclib.net/api/` | None | 5s | `src/lyrics/lrclib.rs` |
| Netease Cloud Music | `https://music.163.com/api/` | None | 5s | `src/lyrics/netease.rs` |
| Artwork image download | Variable (from iTunes API response) | None | None (default) | `src/artwork/mod.rs` |

## Environment Configuration

**Required env vars:** None (all configuration via TOML config file)

**Optional env vars:**
- `RUST_LOG` - Controls tracing log level
- `ZELLIJ` - Auto-detected for terminal protocol fallback

**System requirements:**
- macOS with Apple Music app installed
- `osascript` binary available in PATH (standard macOS)
- Terminal supporting at least basic Unicode (halfblocks); Sixel/Kitty optional for high-quality artwork

---

*Integration audit: 2026-03-24*
