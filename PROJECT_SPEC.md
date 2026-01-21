# AMCLI - Apple Music Command Line Interface

> A powerful Terminal User Interface (TUI) for controlling Apple Music and other media players on macOS, written in Rust

## 项目概述 / Project Overview

**AMCLI** 是一个用 Rust 编写的终端用户界面应用程序，旨在为 Apple Music 和其他 macOS 媒体播放器提供完整的命令行控制体验。项目借鉴了 [go-musicfox](https://github.com/go-musicfox/go-musicfox) 的优秀设计，利用 Rust 的性能和安全优势，针对 macOS 生态系统进行了优化。

**AMCLI** is a Terminal User Interface (TUI) application written in Rust that provides comprehensive command-line control for Apple Music and other macOS media players. Inspired by [go-musicfox](https://github.com/go-musicfox/go-musicfox), it leverages Rust's performance and safety advantages while being optimized for the macOS ecosystem.

### 核心目标 / Core Objectives

1. **🎵 全面的媒体控制** - 支持播放、暂停、音量调节、播放模式切换等所有基础功能
2. **🎨 8-bit Album Art** - 使用 ASCII/像素艺术风格显示专辑封面
3. **📝 实时歌词显示** - 同步显示当前播放歌曲的歌词
4. **🎭 精美的 TUI 界面** - 基于 Ratatui 框架构建的美观交互界面
5. **⚡ 高性能** - Rust 零成本抽象，轻量级、响应迅速、资源占用低
6. **🔌 扩展性强** - 基于 trait 的插件系统，可扩展支持其他播放器

---

## 技术架构 / Technical Architecture

### 技术栈 / Technology Stack

#### 核心技术 / Core Technologies

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Programming Language** | Rust 1.75+ | Zero-cost abstractions, memory safety, excellent performance |
| **TUI Framework** | [Ratatui](https://github.com/ratatui-org/ratatui) | Production-ready TUI framework with rich widget library |
| **Terminal Backend** | [Crossterm](https://github.com/crossterm-rs/crossterm) | Cross-platform terminal manipulation library |
| **Async Runtime** | [Tokio](https://github.com/tokio-rs/tokio) | Asynchronous runtime for concurrent operations |
| **macOS Integration** | AppleScript / osascript | Direct access to Apple Music and macOS media controls |
| **Image Processing** | [image](https://github.com/image-rs/image) + custom converter | High-performance image processing and ASCII conversion |

#### 依赖库 / Key Dependencies

```toml
[dependencies]
# TUI Framework
ratatui = "0.26"
crossterm = "0.27"

# Async Runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# Image Processing
image = "0.24"
rgb = "0.8"

# Configuration
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
config = "0.13"
clap = { version = "4.4", features = ["derive"] }

# HTTP Client for Lyrics API
reqwest = { version = "0.11", features = ["json"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Utilities
chrono = "0.4"
unicode-width = "0.1"
lazy_static = "1.4"
regex = "1.10"
dirs = "5.0"
```

---

## 核心功能模块 / Core Features

### 1. 媒体播放控制 / Media Playback Control

#### Trait-based Player Interface

```rust
// src/player/mod.rs
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    pub position: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

#[async_trait]
pub trait MediaPlayer: Send + Sync {
    // Playback Control
    async fn play(&self) -> Result<()>;
    async fn pause(&self) -> Result<()>;
    async fn toggle(&self) -> Result<()>;
    async fn next(&self) -> Result<()>;
    async fn previous(&self) -> Result<()>;

    // Track Information
    async fn get_current_track(&self) -> Result<Option<Track>>;
    async fn get_playback_state(&self) -> Result<PlaybackState>;

    // Advanced Control
    async fn set_volume(&self, volume: u8) -> Result<()>;
    async fn get_volume(&self) -> Result<u8>;
    async fn seek(&self, seconds: i32) -> Result<()>;
    async fn set_shuffle(&self, enabled: bool) -> Result<()>;
    async fn set_repeat(&self, mode: RepeatMode) -> Result<()>;
}
```

#### Apple Music Implementation

```rust
// src/player/apple_music.rs
use super::{MediaPlayer, PlaybackState, Track};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::process::Command;
use std::time::Duration;

pub struct AppleMusicController;

impl AppleMusicController {
    pub fn new() -> Self {
        Self
    }

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
}

#[async_trait]
impl MediaPlayer for AppleMusicController {
    async fn play(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to play"#)?;
        Ok(())
    }

    async fn toggle(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to playpause"#)?;
        Ok(())
    }

    async fn get_current_track(&self) -> Result<Option<Track>> {
        let script = r#"
            tell application "Music"
                if player state is not stopped then
                    set output to name of current track & "|" & ¬
                                  artist of current track & "|" & ¬
                                  album of current track & "|" & ¬
                                  duration of current track & "|" & ¬
                                  player position
                    return output
                end if
            end tell
        "#;

        let result = self.execute_script(script)?;
        if result.is_empty() {
            return Ok(None);
        }

        let parts: Vec<&str> = result.split('|').collect();
        Ok(Some(Track {
            name: parts[0].to_string(),
            artist: parts[1].to_string(),
            album: parts[2].to_string(),
            duration: Duration::from_secs_f64(parts[3].parse()?),
            position: Duration::from_secs_f64(parts[4].parse()?),
        }))
    }
}
```

**支持的操作：**
- ▶️ 播放/暂停 (Play/Pause/Toggle)
- ⏭️ 下一曲/上一曲 (Next/Previous)
- 🔀 随机播放 (Shuffle)
- 🔁 循环模式 (Repeat: Off/One/All)
- 🔊 音量控制 (Volume: 0-100)
- ⏩ 快进/快退 (Seek: +/-seconds)

### 2. Ratatui TUI 界面 / Ratatui UI Interface

```rust
// src/ui/mod.rs
use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::player::{MediaPlayer, apple_music::AppleMusicController};

pub struct App {
    player: Box<dyn MediaPlayer>,
    current_track: Option<Track>,
    // Add more state as needed
}

impl App {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            player: Box::new(AppleMusicController::new()),
            current_track: None,
        })
    }

    pub async fn update(&mut self) -> Result<()> {
        self.current_track = self.player.get_current_track().await?;
        Ok(())
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("AMCLI - Apple Music Controller")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content
    let content = if let Some(track) = &app.current_track {
        format!("Now Playing:\n{} - {}", track.name, track.artist)
    } else {
        "No track playing".to_string()
    };
    
    let main_block = Paragraph::new(content)
        .block(Block::default().title("Now Playing").borders(Borders::ALL));
    f.render_widget(main_block, chunks[1]);

    // Status bar
    let status = Paragraph::new("[Space] Play/Pause | [[] Prev | []] Next | [q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}
```

#### 主界面布局示例

```
┌─────────────────────────────────────────────┐
│        AMCLI - Apple Music Controller       │
├─────────────────────────────────────────────┤
│                                             │
│  ♫ Now Playing                              │
│  Track:  Shake It Off                       │
│  Artist: Taylor Swift                       │
│  Album:  1989 (Taylor's Version)            │
│                                             │
│  ▶️  ━━━━━━━━●────────  02:15 / 03:42       │
│  🔊 ████████░░ 80%                          │
│                                             │
├─────────────────────────────────────────────┤
│ [Space] Play/Pause  [[] Prev  []] Next     │
│ [q] Quit  [?] Help                          │
└─────────────────────────────────────────────┘
```

### 3. 专辑封面 8-bit 显示 / Album Art Display

#### 实现策略

- **ASCII Art** - 字符密度映射
- **Unicode Blocks** - 半字符精度
- **True Color** - 终端真彩色支持（推荐）

```rust
// src/artwork/converter.rs (planned)
use image::{DynamicImage, GenericImageView};
use ratatui::style::Color;

pub struct AlbumArtConverter {
    width: u32,
    height: u32,
}

impl AlbumArtConverter {
    pub fn to_ascii(&self, img: &DynamicImage) -> String {
        // Resize and convert to ASCII
        let chars = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
        // Implementation...
        String::new()
    }

    pub fn to_truecolor_blocks(&self, img: &DynamicImage) -> Vec<Vec<Color>> {
        // Convert to colored blocks for Ratatui
        vec![]
    }
}
```

### 4. 异步歌词同步 / Async Lyrics Synchronization

```rust
// src/lyrics/mod.rs (planned)
use chrono::Duration;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LyricLine {
    pub timestamp: Duration,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Lyrics {
    pub title: String,
    pub artist: String,
    pub lines: Vec<LyricLine>,
}

impl Lyrics {
    pub fn parse_lrc(content: &str) -> anyhow::Result<Self> {
        // Parse LRC format
        // [00:12.00]Line 1
        // [00:17.20]Line 2
        todo!()
    }

    pub fn get_current_line(&self, position: Duration) -> Option<&LyricLine> {
        self.lines
            .iter()
            .rev()
            .find(|line| line.timestamp <= position)
    }
}
```

---

## 开发路线图 / Development Roadmap

### Phase 1: Core Foundation (Week 1-2) ✅ 进行中

- [x] 项目初始化（Cargo.toml, src/ 结构）
- [x] MediaPlayer trait 定义
- [x] Apple Music AppleScript 桥接
- [ ] Ratatui 基础 UI
- [ ] 键盘事件处理
- [ ] 基础播放控制测试

### Phase 2: Enhanced UI & Album Art (Week 3-4)

- [ ] 专辑封面下载和缓存
- [ ] ASCII/True Color 转换器
- [ ] 美化 UI 布局
- [ ] 配置系统（Serde + TOML）

### Phase 3: Lyrics Integration (Week 5-6)

- [ ] LRC 解析器
- [ ] 歌词 API 集成（Netease, Musixmatch）
- [ ] 实时同步显示

### Phase 4-6: Advanced Features & Release

- [ ] 播放列表管理
- [ ] 插件系统（trait objects）
- [ ] 性能优化
- [ ] 打包发布（Homebrew, cargo-dist）

---

## 项目结构 / Project Structure

```
amcli/
├── Cargo.toml              # Rust 项目清单
├── src/
│   ├── main.rs            # 入口，Tokio runtime
│   ├── player/
│   │   ├── mod.rs         # MediaPlayer trait
│   │   └── apple_music.rs # AppleScript 实现
│   ├── ui/
│   │   └── mod.rs         # Ratatui UI
│   ├── lyrics/
│   │   ├── mod.rs         # 歌词管理
│   │   └── parser.rs      # LRC 解析
│   ├── artwork/
│   │   ├── mod.rs         # 专辑封面
│   │   └── converter.rs   # 图像转换
│   └── config/
│       └── mod.rs         # 配置管理
├── configs/
│   └── config.example.toml
└── scripts/
    └── applescript/       # AppleScript 辅助
```

---

## Rust 优势 / Why Rust

### 性能 / Performance
- 零成本抽象，无垃圾回收
- 更快的启动时间和更低的内存占用
- 编译时优化

### 安全性 / Safety
- 编译时内存安全
- 无数据竞争
- 强类型系统

### 并发 / Concurrency
- Tokio 异步运行时
- 安全的并发原语
- async/await 语法

### 生态系统 / Ecosystem
- Ratatui - 成熟的 TUI 框架
- Serde - 强大的序列化
- 丰富的 crates.io 生态

---

## 快速开始 / Quick Start

### 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 构建项目

```bash
cd /Users/jac/Repos/amcli
cargo build
cargo run
```

### 开发命令

```bash
cargo fmt              # 格式化代码
cargo clippy           # 代码检查
cargo test             # 运行测试
cargo watch -x run     # 自动重载
```

---

## 配置示例 / Configuration

```toml
# ~/.config/amcli/config.toml
[app]
language = "zh-CN"
theme = "dark"

[player]
default_player = "apple_music"
default_volume = 50

[ui]
show_album_art = true
album_art_mode = "truecolor"
show_lyrics = true

[lyrics]
auto_download = true
providers = ["local", "netease", "musixmatch"]
```

---

## 贡献 / Contributing

欢迎贡献！请查看 [TODO.md](TODO.md) 了解待办任务。

## 许可证 / License

MIT License - 详见 [LICENSE](LICENSE) 文件

---

## 资源 / Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui 文档](https://ratatui.rs/)
- [Tokio 教程](https://tokio.rs/tokio/tutorial)
- [go-musicfox](https://github.com/go-musicfox/go-musicfox) - 设计灵感

---

**Last Updated:** 2026-01-21  
**Project Status:** 🚧 Phase 1 - Core Foundation  
**Language:** Rust 🦀
