# AI Agent 准备清单 / AI Agent Preparation Checklist

> 在让 AI 通过 ralphloop 自动执行 TODO.md 之前的准备工作

## ✅ 已完成 / Completed

- [x] ✅ Rust 项目结构已创建
- [x] ✅ Cargo.toml 已配置所有依赖
- [x] ✅ 基础代码框架已搭建（main.rs, player, ui）
- [x] ✅ PROJECT_SPEC.md 已更新为 Rust
- [x] ✅ TODO.md 已更新为 Rust 任务
- [x] ✅ SETUP.md 开发环境指南已就绪

## 🔧 需要立即完成 / Immediate ToDo

### 1. 确保项目能编译 🚨 优先级最高

```bash
cd /Users/jac/Repos/amcli
cargo check       # 检查代码（不生成可执行文件）
cargo build       # 完整构建（会暴露编译错误）
```

**预期问题：**
- 模块声明可能有问题（placeholder modules）
- 某些 trait 实现可能不完整

**修复建议：**
- 修正 `src/artwork/mod.rs` 等模块声明
- 添加 `#[allow(dead_code)]` 到未完成的部分

### 2. 创建 CONTRIBUTING.md

AI 需要知道：
- 代码规范（rustfmt, clippy）
- 提交信息格式
- 测试要求
- PR 流程

### 3. 设置 .github/workflows/ci.yml

让 AI 的代码自动验证：
```yaml
- cargo fmt --check
- cargo clippy -- -D warnings  
- cargo test
- cargo build --release
```

### 4. 创建测试模板

```rust
// tests/integration_test.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_player_interface() {
        // AI can follow this pattern
    }
}
```

### 5. 添加任务上下文文件

创建 `AI_CONTEXT.md` 提供：
- 项目当前状态
- 下一步应该做什么
- 常见陷阱和注意事项

## 📋 推荐准备工作 / Recommended Preparations

### A. 修复当前编译错误

```bash
# 检查问题
cargo check 2>&1 | tee build_errors.txt

# 让 AI 知道从哪里开始
```

### B. 创建示例测试

```rust
// tests/apple_music_test.rs (示例)
#[tokio::test]
async fn test_play_pause() {
    let controller = AppleMusicController::new();
    // Mock test or integration test
}
```

### C. 设置 pre-commit hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo fmt
cargo clippy -- -D warnings
```

### D. 文档完善

- [ ] **ARCHITECTURE.md** - 架构概述
- [ ] **DEVELOPMENT.md** - 开发指南
- [ ] **TESTING.md** - 测试策略

### E. Issue Templates

创建 `.github/ISSUE_TEMPLATE/`:
- `feature.md` - 新功能模板
- `bug.md` - Bug 报告模板

## 🤖 AI Agent 配置建议

### 1. 给 AI 的指令模板

```markdown
你是一个 Rust 开发专家，正在开发 AMCLI 项目。

**当前任务：** TODO.md 中的 Phase 1, Task X

**要求：**
- 使用 Rust 最佳实践
- 遵循项目代码风格（cargo fmt）
- 添加单元测试
- 更新文档
- 通过 `cargo clippy` 检查

**参考：**
- PROJECT_SPEC.md - 项目规格
- SETUP.md - 开发环境
- 现有代码风格

**完成标准：**
- [ ] 代码编译通过
- [ ] 测试通过
- [ ] Clippy 无警告
- [ ] 文档已更新
```

### 2. 任务执行循环

```python
# ralphloop 伪代码
while TODO.md 有未完成任务:
    1. 读取下一个任务
    2. 理解需求（通过 PROJECT_SPEC.md）
    3. 编写代码
    4. 运行测试 (cargo test)
    5. 检查质量 (cargo clippy)
    6. 提交更改
    7. 标记任务完成 ✓
```

### 3. 验证检查点

每个任务完成后验证：
```bash
#!/bin/bash
# verify.sh

set -e

echo "🔍 Formatting..."
cargo fmt --check

echo "🔍 Linting..."
cargo clippy -- -D warnings

echo "🧪 Testing..."
cargo test

echo "🏗️ Building..."
cargo build

echo "✅ All checks passed!"
```

## ⚠️ 常见陷阱 / Common Pitfalls

### 1. AppleScript 权限
- AI 无法测试真实的 Apple Music 交互
- 建议：创建 mock 实现用于测试

### 2. 异步代码复杂性
- Tokio runtime 配置
- 生命周期问题
- 建议：从简单的同步代码开始

### 3. Ratatui 渲染
- 终端尺寸处理
- 事件循环管理
- 建议：先实现基础布局

## 📦 推荐的项目结构增强

```
amcli/
├── .github/
│   ├── workflows/
│   │   └── ci.yml           # CI/CD
│   └── ISSUE_TEMPLATE/
├── docs/
│   ├── ARCHITECTURE.md      # 架构文档
│   ├── DEVELOPMENT.md       # 开发指南
│   └── AI_CONTEXT.md        # AI 上下文
├── tests/
│   ├── integration/         # 集成测试
│   └── unit/                # 单元测试
├── examples/                # 示例代码
├── benches/                 # 性能测试
└── scripts/
    ├── verify.sh            # 验证脚本
    └── setup-hooks.sh       # Git hooks
```

## 🎯 立即行动项 / Immediate Actions

**在启动 AI Agent 之前，请执行：**

1. **修复编译错误**
   ```bash
   cargo build 2>&1 | tee build_errors.txt
   # 修复所有错误
   ```

2. **创建 CONTRIBUTING.md**
   - 代码规范
   - 提交规范
   - 测试要求

3. **设置 CI/CD**
   - GitHub Actions workflow
   - 自动化测试和检查

4. **创建 AI_CONTEXT.md**
   - 项目当前状态
   - 下一步目标
   - 注意事项

5. **准备验证脚本**
   ```bash
   ./scripts/verify.sh
   ```

## 📝 AI Agent 启动检查清单

启动前确认：
- [ ] `cargo build` 成功
- [ ] `cargo test` 通过（即使只有示例测试）
- [ ] `cargo clippy` 无警告
- [ ] README.md 清晰
- [ ] TODO.md 任务明确
- [ ] CONTRIBUTING.md 存在
- [ ] CI/CD 配置完成

## 🚀 启动命令示例

```bash
# 1. 确保环境就绪
cargo build

# 2. 启动 AI Agent（根据你的 ralphloop 工具）
# ralphloop --project amcli --task-file TODO.md --mode auto

# 3. 监控进度
# watch -n 5 'git log --oneline -10'
```

---

**准备好后，AI 就可以开始自动化开发了！** 🤖🦀
