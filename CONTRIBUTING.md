# Contributing to AMCLI

感谢你对 AMCLI 的贡献！

## 开发流程 / Development Workflow

### 1. 设置开发环境

```bash
# 克隆项目
git clone https://github.com/yourusername/amcli.git
cd amcli

# 安装 Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装开发工具
rustup component add rustfmt clippy

# 构建项目
cargo build
```

### 2. 创建分支

```bash
git checkout -b feature/your-feature-name
# 或
git checkout -b fix/bug-description
```

### 3. 编写代码

**代码规范：**
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust 命名规范
- 添加适当的注释和文档

**提交前检查：**
```bash
# 格式化
cargo fmt

# 检查
cargo clippy -- -D warnings

# 测试
cargo test

# 构建
cargo build
```

### 4. 编写测试

每个新功能都应该有测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_function() {
        // 测试代码
    }

    #[tokio::test]
    async fn test_async_function() {
        // 异步测试
    }
}
```

### 5. 提交代码

**Commit Message 格式：**

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type:**
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例：**
```bash
git commit -m "feat(player): add AppleScript volume control"
git commit -m "fix(ui): resolve layout overflow issue"
git commit -m "docs: update setup guide for Rust"
```

### 6. 推送并创建 PR

```bash
git push origin feature/your-feature-name
```

然后在 GitHub 上创建 Pull Request。

## 代码规范 / Code Standards

### Rust 风格

- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 使用 `cargo fmt` 默认配置
- 通过 `cargo clippy` 所有检查

### 错误处理

使用 `anyhow` 和 `thiserror`：

```rust
use anyhow::{Result, Context};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("AppleScript execution failed: {0}")]
    ScriptError(String),
}

pub async fn do_something() -> Result<()> {
    some_operation()
        .context("Failed to do something")?;
    Ok(())
}
```

### 异步代码

使用 Tokio runtime：

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ...
}

#[async_trait]
pub trait MediaPlayer {
    async fn play(&self) -> Result<()>;
}
```

### 文档注释

```rust
/// 播放当前曲目
///
/// # Errors
///
/// 如果 AppleScript 执行失败则返回错误
///
/// # Examples
///
/// ```no_run
/// use amcli::player::AppleMusicController;
///
/// let player = AppleMusicController::new();
/// player.play().await?;
/// ```
pub async fn play(&self) -> Result<()> {
    // ...
}
```

## 测试策略 / Testing Strategy

### 单元测试

```rust
// src/player/apple_music.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_formatting() {
        // 测试逻辑
    }
}
```

### 集成测试

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_player_integration() {
    // 集成测试
}
```

### 运行测试

```bash
# 所有测试
cargo test

# 特定测试
cargo test test_name

# 显示输出
cargo test -- --nocapture

# 集成测试
cargo test --test integration_test
```

## PR 检查清单 / PR Checklist

提交 PR 前确认：

- [ ] 代码通过 `cargo fmt --check`
- [ ] 代码通过 `cargo clippy -- -D warnings`
- [ ] 所有测试通过 `cargo test`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] Commit message 符合规范
- [ ] 没有合并冲突

## 优先级任务 / Priority Tasks

查看 [TODO.md](TODO.md) 了解当前任务优先级。

## 需要帮助？ / Need Help?

- 📖 查看 [PROJECT_SPEC.md](PROJECT_SPEC.md)
- 🚀 阅读 [SETUP.md](SETUP.md)
- 💬 在 Issues 中提问
- 📧 联系维护者

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
