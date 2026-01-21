# AMCLI - Apple Music Command Line Interface

<div align="center">

🎵 **一个用 Rust 编写的 Apple Music 终端控制器**

[![Rust Version](https://img.shields.io/badge/Rust-1.75+-dea584?style=flat&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Developing-green.svg)](PROJECT_SPEC.md)

[English](#english) | [中文](#中文)

</div>

---

## 中文

### 📖 项目简介

**AMCLI** 是一个功能强大的终端用户界面（TUI）应用程序，用于在 macOS 上控制 Apple Music 和其他媒体播放器。它提供了：

- 🎮 完整的媒体播放控制
- 🎨 8-bit 风格的专辑封面显示
- 📝 实时同步歌词
- ⚡ 用 Rust 编写，轻量级、高性能
- 🎯 Vim 风格的快捷键
- 🔌 插件系统支持多个播放器

### ✨ 核心特性

#### 🎵 媒体控制
- 播放/暂停/下一曲/上一曲
- 音量调节和静音
- 播放模式切换（随机、循环）
- 精确的播放进度控制

#### 🎨 视觉体验
- ASCII/Unicode/真彩色专辑封面
- 可自定义的颜色主题
- 响应式布局
- 流畅的动画效果

#### 📝 歌词功能
- 实时同步 LRC 歌词
- 多种显示模式（滚动/卡拉OK/全文）
- 多个歌词源（Netease, Musixmatch, Genius）
- 本地歌词文件支持

#### 🔧 高级功能
- 播放列表管理
- 音乐库浏览（专辑/艺术家/歌曲）
- 搜索功能
- macOS 系统集成（通知、Now Playing、媒体键）
- 插件支持（Spotify, VLC, Last.fm）

### 🚀 快速开始

> [!WARNING]
> **项目状态：** 目前处于规划阶段。完整实现预计需要 10-12 周。

#### 安装（计划中）

```bash
# 使用 Homebrew
brew install amcli

# 或从源码编译
git clone https://github.com/yourusername/amcli.git
cd amcli
make install
```

#### 使用

```bash
# 启动 AMCLI
amcli

# 显示帮助
amcli --help

# 使用配置文件
amcli --config ~/.config/amcli/config.toml
```

### ⌨️ 快捷键

| 功能 | 快捷键 |
|------|--------|
| 播放/暂停 | `Space` |
| 下一曲 | `]` |
| 上一曲 | `[` |
| 音量+ | `=` / `+` |
| 音量- | `-` / `_` |
| 向上/下导航 | `k` / `j` 或 `↑` / `↓` |
| 搜索 | `/` |
| 帮助 | `?` |
| 退出 | `q` |

完整快捷键列表请查看 [PROJECT_SPEC.md](PROJECT_SPEC.md#键盘快捷键系统--keyboard-shortcuts)

### 📋 项目文档

- **[PROJECT_SPEC.md](PROJECT_SPEC.md)** - 完整的项目规格说明（69KB，包含详细的技术架构、功能设计、实现路线图）
- **[ROADMAP.md](ROADMAP.md)** - 开发路线图和里程碑（即将推出）
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - 贡献指南（即将推出）

### 🏗️ 开发路线图

项目分为 6 个主要阶段：

1. **阶段 1** (Week 1-2): 核心基础 - TUI 框架 + Apple Music 控制
2. **阶段 2** (Week 3-4): UI 增强 + 专辑封面
3. **阶段 3** (Week 5-6): 歌词集成
4. **阶段 4** (Week 7-8): 高级功能 (播放列表、库浏览)
5. **阶段 5** (Week 9-10): 插件系统 + 多播放器支持
6. **阶段 6** (Week 11-12): 优化和发布

详细信息请查看 [PROJECT_SPEC.md](PROJECT_SPEC.md#开发路线图--development-roadmap)

### 🛠️ 技术栈

- **语言:** Rust 1.75+
- **TUI 框架:** [Ratatui](https://github.com/ratatui-org/ratatui)  
- **终端后端:** [Crossterm](https://github.com/crossterm-rs/crossterm)
- **异步运行时:** [Tokio](https://tokio.rs/)
- **macOS 集成:** AppleScript / osascript
- **配置:** Serde + TOML + Clap

### 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解如何参与项目。

### 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

### 🙏 致谢

- [go-musicfox](https://github.com/go-musicfox/go-musicfox) - 设计灵感来源
- [Ratatui](https://ratatui.rs/) - 优秀的 TUI 库

---

## English

### 📖 Project Overview

**AMCLI** is a powerful Terminal User Interface (TUI) application for controlling Apple Music and other media players on macOS. It provides:

- 🎮 Complete media playback control
- 🎨 8-bit style album artwork display
- 📝 Real-time synchronized lyrics
- ⚡ Lightweight and high performance
- 🎯 Vim-style keybindings
- 🔌 Plugin system for multiple players

### ✨ Key Features

#### 🎵 Media Control
- Play/Pause/Next/Previous
- Volume adjustment and mute
- Play mode switching (shuffle, repeat)
- Precise playback position control

#### 🎨 Visual Experience
- ASCII/Unicode/TrueColor album artwork
- Customizable color themes
- Responsive layout
- Smooth animations

#### 📝 Lyrics Features
- Real-time LRC lyrics synchronization
- Multiple display modes (scroll/karaoke/full)
- Multiple lyrics sources (Netease, Musixmatch, Genius)
- Local lyrics file support

#### 🔧 Advanced Features
- Playlist management
- Music library browsing (albums/artists/songs)
- Search functionality
- macOS system integration (notifications, Now Playing, media keys)
- Plugin support (Spotify, VLC, Last.fm)

### 🚀 Quick Start

> [!WARNING]
> **Project Status:** Currently in planning phase. Full implementation expected in 10-12 weeks.

#### Installation (Planned)

```bash
# Using Homebrew
brew install amcli

# Or build from source
git clone https://github.com/yourusername/amcli.git
cd amcli
make install
```

#### Usage

```bash
# Launch AMCLI
amcli

# Show help
amcli --help

# Use custom config
amcli --config ~/.config/amcli/config.toml
```

### ⌨️ Keybindings

| Action | Key |
|--------|-----|
| Play/Pause | `Space` |
| Next Track | `]` |
| Previous Track | `[` |
| Volume Up | `=` / `+` |
| Volume Down | `-` / `_` |
| Navigate Up/Down | `k` / `j` or `↑` / `↓` |
| Search | `/` |
| Help | `?` |
| Quit | `q` |

See [PROJECT_SPEC.md](PROJECT_SPEC.md#键盘快捷键系统--keyboard-shortcuts) for complete keybindings.

### 📋 Documentation

- **[PROJECT_SPEC.md](PROJECT_SPEC.md)** - Complete project specification (69KB, includes detailed technical architecture, feature design, implementation roadmap)
- **[ROADMAP.md](ROADMAP.md)** - Development roadmap and milestones (Coming soon)
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines (Coming soon)

### 🏗️ Development Roadmap

The project is divided into 6 major phases:

1. **Phase 1** (Week 1-2): Core Foundation - TUI framework + Apple Music control
2. **Phase 2** (Week 3-4): UI Enhancement + Album artwork
3. **Phase 3** (Week 5-6): Lyrics integration
4. **Phase 4** (Week 7-8): Advanced features (playlists, library browsing)
5. **Phase 5** (Week 9-10): Plugin system + Multi-player support
6. **Phase 6** (Week 11-12): Polish and release

See [PROJECT_SPEC.md](PROJECT_SPEC.md#开发路线图--development-roadmap) for details.

### 🛠️ Tech Stack

- **Language:** Rust 1.75+
- **TUI Framework:** [Ratatui](https://github.com/ratatui-org/ratatui)  
- **Terminal Backend:** [Crossterm](https://github.com/crossterm-rs/crossterm)
- **Async Runtime:** [Tokio](https://tokio.rs/)
- **macOS Integration:** AppleScript / osascript
- **Configuration:** Serde + TOML + Clap

### 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for how to get involved.

### 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### 🙏 Acknowledgments

- [go-musicfox](https://github.com/go-musicfox/go-musicfox) - Design inspiration
- [Ratatui](https://ratatui.rs/) - Excellent TUI library

---

<div align="center">

**Made with ❤️ for music lovers and terminal enthusiasts**

⭐ Star this repo if you find it interesting!

</div>
