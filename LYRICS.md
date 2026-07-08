# AMCLI 歌词系统技术文档

> **Phase 3+ 当前实现** - LRCLIB + 网易云在线双源歌词系统 + 实时同步显示

---

## 目录

1. [架构概览](#架构概览)
2. [LRC 格式解析](#lrc-格式解析)
3. [在线歌词获取](#在线歌词获取)
4. [同步与显示](#同步与显示)
5. [缓存系统](#缓存系统)
6. [扩展指南](#扩展指南)

---

## 架构概览

AMCLI 的歌词系统采用 **Provider 模式** + **在线源竞速/校准** + **LRU 缓存** 的三层架构：

```
┌─────────────────────────────────────────────────────┐
│                  LyricsManager                      │
│  - 在线源竞速与会话 primary 校准                      │
│  - LRU 缓存 (Arc<Mutex<LruCache>>)                  │
│  - 统一错误处理                                      │
└────────────┬────────────────────────────────────────┘
             │
    ┌────────────┴────────────┐
    │                         │
┌───▼────────┐        ┌───────▼──────┐
│ LRCLIB(5)  │        │ Netease(10)  │
│ 全球歌词    │        │ 中文/CJK 歌词 │
└────────────┘        └──────────────┘
```

### 核心组件

| 模块 | 文件 | 职责 |
|-----|------|-----|
| **LyricsManager** | `src/lyrics/mod.rs` | 协调多个 Provider，管理缓存 |
| **LRC Parser** | `src/lyrics/parser.rs` | 解析 LRC 格式，提取时间轴 |
| **LyricsProvider Trait** | `src/lyrics/provider.rs` | 统一的歌词源接口 |
| **LrclibProvider** | `src/lyrics/lrclib.rs` | LRCLIB API - 全球歌词 (优先级 5) |
| **NeteaseProvider** | `src/lyrics/netease.rs` | 网易云音乐 API - 中文歌曲 (优先级 10) |

---

## LRC 格式解析

### LRC 标准格式

LRC (Lyric) 是一种基于文本的歌词格式，每行包含时间戳和歌词文本：

```
[ar:艺术家名]
[ti:歌曲名]
[al:专辑名]
[offset:500]
[00:12.34]第一行歌词
[00:15.67]第二行歌词
[00:20.00][00:40.00]重复的歌词
```

**⚠️ 网易云特有问题**：部分歌词文件包含无时间戳的元数据行：
```
作词 : 周杰伦
作曲 : 周杰伦
[00:12.34]真正的歌词
```

### 实现细节

#### 0. 过滤无时间戳行（重要）

```rust
// 跳过没有时间戳的行（如"作词："、"作曲："等）
if !TIME_REGEX.is_match(line) {
    continue;
}
```

**效果**：自动过滤掉网易云的"作词：作曲："等中文元数据。

#### 1. 正则表达式解析

```rust
lazy_static! {
    // 时间戳: [mm:ss.xx] 或 [mm:ss.xxx]
    static ref TIME_REGEX: Regex = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\]").unwrap();

    // 元数据: [key:value]
    static ref META_REGEX: Regex = Regex::new(r"\[([a-z]+):(.*)\]").unwrap();
}
```

#### 2. 时间轴计算

```rust
let min: u64 = caps[1].parse()?;
let sec: u64 = caps[2].parse()?;
let ms_str = &caps[3];

// 兼容 2 位和 3 位毫秒
let ms: u64 = if ms_str.len() == 2 {
    ms_str.parse::<u64>()? * 10  // 百分秒转毫秒
} else {
    ms_str.parse::<u64>()?
};

let total_ms = (min * 60 + sec) * 1000 + ms;
let timestamp = Duration::from_millis(total_ms);
```

#### 3. 偏移量处理

LRC 文件可能包含全局偏移量（用于同步调整）：

```rust
if key == "offset" {
    lyrics.offset = value.parse().unwrap_or(0);
}

// 应用偏移量
if lyrics.offset != 0 {
    let offset_dur = Duration::from_millis(lyrics.offset.abs() as u64);
    for line in lyrics.lines.iter_mut() {
        if lyrics.offset > 0 {
            line.timestamp += offset_dur;  // 向后延迟
        } else {
            line.timestamp = line.timestamp.saturating_sub(offset_dur);  // 向前提前
        }
    }
}
```

#### 4. 多时间戳支持

一行歌词可能重复出现在不同时间点：

```
[00:12.00][00:25.00]这是副歌
```

解析器会为同一文本创建两个独立的 `LyricLine` 对象，并最终按时间排序。

---

## 在线歌词获取

### Provider 优先级机制

每个 Provider 返回一个优先级数字（**越小越优先**）：

```rust
impl LyricsProvider for LrclibProvider {
    fn priority(&self) -> u8 { 5 }  // 全球歌曲优先
}

impl LyricsProvider for NeteaseProvider {
    fn priority(&self) -> u8 { 10 }  // 中文歌曲备选
}
```

**执行流程**：

1. `LyricsManager` 先按静态优先级排序 Provider（LRCLIB 优先级 5，Netease 优先级 10）
2. 未校准时并发竞速查询在线源，取最快命中的歌词，并在明确时记录本会话 primary provider
3. 已校准后优先查询 primary provider，再按优先级回退到其它 provider
4. 如果 provider 可达但无匹配，显示 "NO LYRICS AVAILABLE"；如果 provider 全部不可达，显示/记录 "NO SIGNAL" 类错误

### LRCLIB API 实现 (推荐)

**适用场景**：全球歌曲（英文、欧美流行、日韩、多语言）

**优势**：
- ✅ 免费且无需 API Key
- ✅ 原生 LRC 格式支持
- ✅ 社区驱动，国际化覆盖优秀
- ✅ 无"作词作曲"等中文元数据污染

#### API 端点

**搜索歌词**：`https://lrclib.net/api/get?artist_name={artist}&track_name={track}`

**请求方式**：GET

**必需 Headers**：
```rust
headers.insert("Lrclib-Client", "AMCLI v1.0.0");
```

**返回 JSON**：
```json
{
  "id": 123456,
  "trackName": "Song Name",
  "artistName": "Artist",
  "albumName": "Album",
  "duration": 240,
  "syncedLyrics": "[00:12.34]Line 1\n[00:15.67]Line 2\n...",
  "plainLyrics": "Line 1\nLine 2\n..."
}
```

#### 实现代码

```rust
let url = format!(
    "https://lrclib.net/api/get?artist_name={}&track_name={}",
    urlencoding::encode(&track.artist),
    urlencoding::encode(&track.name)
);

let resp: Value = self.client.get(url)
    .headers(self.headers())
    .send().await?
    .json().await?;

if let Some(synced_lyrics) = resp["syncedLyrics"].as_str() {
    return Ok(Some(parse_lrc(synced_lyrics)?));
}
```

### 网易云音乐 API 实现 (备选)

**适用场景**：中文歌曲（华语流行、国语老歌等）

#### 阶段一：搜索曲目 ID

**API 端点**：`https://music.163.com/api/cloudsearch/pc`

**请求参数**：
```rust
let params = [
    ("s", "歌名 歌手"),       // 搜索关键词
    ("type", "1"),           // 1=单曲, 10=专辑
    ("limit", "1"),          // 只需要第一个结果
    ("offset", "0"),         // 分页偏移
];
```

**HTTP Headers**（伪装浏览器请求）：
```rust
headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; ...) Chrome/120.0.0.0");
headers.insert(REFERER, "https://music.163.com");
headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded");
```

**返回 JSON**：
```json
{
  "result": {
    "songs": [
      { "id": 123456789, "name": "歌曲名", "artists": [...] }
    ]
  }
}
```

#### 阶段二：获取 LRC 歌词

**API 端点**：`https://music.163.com/api/song/lyric?id={song_id}&lv=-1&kv=-1&tv=-1`

**参数说明**：
- `lv`: Lyric Version（-1 = 最新版本）
- `kv`: Karaoke Version（-1 = 不需要逐字时间轴）
- `tv`: Translation Version（-1 = 同时获取翻译）

**返回 JSON**：
```json
{
  "lrc": {
    "lyric": "[00:12.34]歌词第一行\n[00:15.67]歌词第二行\n..."
  },
  "tlyric": {
    "lyric": "[00:12.34]Translation line 1\n..."  // 可选
  }
}
```

#### 错误处理

| 场景 | 处理方式 |
|-----|---------|
| 搜索无结果 | `return Ok(None)` → 尝试下一个 Provider |
| 网络超时/不可达 | surface provider error；不把临时故障缓存成永久 miss |
| 纯音乐曲目 | 检测空字符串 → `return Ok(None)` |
| API 限流 | 自动重试（未实现）或缓存未命中时失败 |

---

## 同步与显示

### 二分查找定位当前行

歌词数组按时间排序，使用 **二分查找** 在 `O(log n)` 时间内定位当前播放位置：

```rust
impl Lyrics {
    pub fn find_index(&self, position: Duration) -> usize {
        self.lines
            .partition_point(|line| line.timestamp <= position)
            .saturating_sub(1)
    }
}
```

**原理**：`partition_point` 找到第一个 `timestamp > position` 的索引，减 1 即为当前行。

### 滚动视图实现

```rust
let current_index = lyrics.find_index(track.position);
let h = area.height as usize;
let mid = h / 2;

// 计算滚动偏移，使当前行居中
let scroll = current_index.saturating_sub(mid) as u16;

let p = Paragraph::new(lines)
    .alignment(Alignment::Center)
    .scroll((scroll, 0));
```

**效果**：当前播放的歌词始终保持在终端窗口的正中间。

### 视觉反馈

```rust
for (i, line) in lyrics.lines.iter().enumerate() {
    let distance = (i as isize - current_index as isize).unsigned_abs();
    let style = if distance == 0 {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else if distance <= 2 {
        Style::default().fg(theme.primary)
    } else {
        Style::default().fg(theme.dim)
    };
    lines.push(Line::from(Span::styled(&line.text, style)));
}
```

---

## 缓存系统

### LRU 缓存设计

使用 `lru` crate 实现最近最少使用缓存，避免重复查询：

```rust
pub struct LyricsManager {
    providers: Vec<Arc<dyn LyricsProvider>>,
    cache: Arc<Mutex<LruCache<String, Lyrics>>>,
    primary: Arc<Mutex<Option<usize>>>,
}
```

**缓存键**：版本感知的 track cache key（基于歌曲元数据，避免不同版本互相污染）

**缓存大小**：20 条（可配置）

### 线程安全

- `Arc<Mutex<...>>`：允许在异步 UI 更新时安全访问
- 读操作：先获取锁 → 查询 → 释放锁
- 写操作：Provider 查询成功后 → 获取锁 → 写入 → 释放锁

### 缓存策略

```rust
pub async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
    let cache_key = format!("{}-{}", track.artist, track.name);

    // 1. 检查缓存
    {
        let mut cache = self.cache.lock()?;
        if let Some(lyrics) = cache.get(&cache_key) {
            return Ok(Some(lyrics.clone()));  // 命中，直接返回
        }
    }

    // 2. 查询 Providers：未校准时竞速；已校准时 primary-first + fallback
    let result = match self.calibrated_primary() {
        Some(primary) => self.fetch_sequential(track, primary).await,
        None => self.fetch_race(track).await,
    }?;

    // 3. 只缓存成功命中的歌词；临时失败或无匹配不写成永久 miss
    if let Some(lyrics) = &result {
        let mut cache = self.cache.lock()?;
        cache.put(cache_key, lyrics.clone());
    }
    Ok(result)
}
```

**优化效果**：
- 首次查询：200-500ms（网络请求）
- 重复查询：< 1μs（内存命中）

---

## 扩展指南

### 添加新的歌词源

#### 1. 实现 `LyricsProvider` Trait

```rust
// src/lyrics/musixmatch.rs
use crate::lyrics::provider::LyricsProvider;
use async_trait::async_trait;

pub struct MusixmatchProvider {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
impl LyricsProvider for MusixmatchProvider {
    async fn get_lyrics(&self, track: &Track) -> Result<Option<Lyrics>> {
        // 1. 调用 Musixmatch API 搜索
        // 2. 获取歌词文本
        // 3. 解析并返回
        todo!()
    }

    fn priority(&self) -> u8 {
        20  // 优先级低于网易云
    }

    fn name(&self) -> &'static str {
        "musixmatch"
    }
}
```

#### 2. 注册到 `LyricsManager`

```rust
// src/ui/mod.rs
let mut lyrics_manager = LyricsManager::new(20);
lyrics_manager.add_provider(Box::new(LrclibProvider::new()));
lyrics_manager.add_provider(Box::new(NeteaseProvider::new()));
lyrics_manager.add_provider(Box::new(MusixmatchProvider::new(api_key)));  // 可选
```

### 性能优化建议

| 优化项 | 实现方式 |
|-------|---------|
| **并发查询** | 当前已使用 provider racing 查询在线源，命中后按会话校准 primary provider |
| **持久化缓存** | 将 LRU 缓存序列化到磁盘（`~/.cache/amcli/lyrics.db`） |
| **预加载** | 播放列表模式下预先加载下一首歌词 |
| **请求限流** | 使用 `governor` crate 避免 API 限流 |

---

## 技术指标

| 指标 | 数值 | 备注 |
|-----|------|-----|
| **解析速度** | < 1ms | 单个 LRC 文件（~100 行） |
| **同步精度** | ±50ms | 依赖 AppleScript 查询频率 |
| **缓存命中率** | > 95% | 重复播放场景 |
| **在线查询延迟** | 200-500ms | 网易云 API 两次请求 |
| **内存占用** | < 500KB | 20 条缓存歌词 |

---

## 常见问题

### Q: 为什么有些歌词不同步？

A: 可能原因：
1. LRC 文件的 `[offset:]` 标签不正确 → 手动调整
2. Apple Music 的播放位置查询延迟 → 系统限制
3. 网易云返回的歌词版本与 Apple Music 不同 → 优先依赖 LRCLIB/Netease 的标题、歌手、专辑、时长匹配评分；必要时改进匹配规则

### Q: 如何禁用某个在线歌词源？

A: 修改 `src/ui/mod.rs` 的 `default_lyrics_manager()`，只注册需要的在线 Provider：

```rust
let mut lyrics_manager = LyricsManager::new(20);
lyrics_manager.add_provider(Box::new(LrclibProvider::new()));
// lyrics_manager.add_provider(Box::new(NeteaseProvider::new()));  // 注释掉
```

### Q: 还支持本地 LRC 文件吗？

A: 当前 v0.3.0 代码已移除 local file provider，默认只使用 LRCLIB 与 Netease 在线源。若要恢复本地 LRC，需要重新实现 `LyricsProvider` 并在 `default_lyrics_manager()` 注册。

---

## 相关文档

- **[README.md](README.md)** - 项目总览和快速开始
- **[PROJECT_SPEC.md](PROJECT_SPEC.md)** - 完整技术规格
- **[TODO.md](TODO.md)** - Phase 3 开发清单
- **[AGENTS.md](AGENTS.md)** - AI 开发协作指南

---

**最后更新**: 2026-07-08
**维护者**: AMCLI Development Team
