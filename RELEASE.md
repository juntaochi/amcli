# Release Checklist / 发布检查清单

This document provides a step-by-step guide for releasing AMCLI.

## Pre-Release Preparation / 发布前准备

### 1. Code Quality / 代码质量

- [ ] All tests pass: `cargo test --all-features`
- [ ] Code is properly formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy --all-features -- -D warnings`
- [ ] Documentation builds: `cargo doc --no-deps --all-features`
- [ ] Run verification script: `./scripts/verify.sh`

### 2. Version Management / 版本管理

- [ ] Update version in `Cargo.toml`
- [ ] Update version in `homebrew/amcli.rb` template
- [ ] Create/update `CHANGELOG.md` with release notes
- [ ] Ensure all documentation references correct version

### 3. Documentation / 文档

- [ ] README.md is up to date
- [ ] CHANGELOG.md includes all changes since last release
- [ ] All example code and screenshots are current
- [ ] Installation instructions are accurate

### 4. Git Repository / Git 仓库

- [ ] All changes are committed
- [ ] Working directory is clean: `git status`
- [ ] On the correct branch (usually `main`)
- [ ] Branch is up to date with remote: `git pull`

## Creating a Release / 创建发布

### Step 1: Create and Push Git Tag / 创建并推送标签

```bash
# Set the version (without 'v' prefix)
VERSION="0.1.0"

# Create annotated tag
git tag -a "v${VERSION}" -m "Release version ${VERSION}"

# Push the tag to trigger release workflow
git push origin "v${VERSION}"
```

This will automatically trigger the GitHub Actions release workflow which will:
- Build binaries for macOS (Intel and Apple Silicon)
- Create a GitHub Release
- Upload binary artifacts and SHA256 checksums

### Step 2: Verify GitHub Release / 验证 GitHub 发布

1. Go to https://github.com/juntaochi/amcli/releases
2. Verify the release was created successfully
3. Check that all artifacts are present:
   - `amcli-v${VERSION}-x86_64-apple-darwin.tar.gz`
   - `amcli-v${VERSION}-x86_64-apple-darwin.tar.gz.sha256`
   - `amcli-v${VERSION}-arm64-apple-darwin.tar.gz`
   - `amcli-v${VERSION}-arm64-apple-darwin.tar.gz.sha256`
4. Download and verify checksums match

### Step 3: Set Up Homebrew Tap (First Release Only) / 设置 Homebrew Tap

**Only needed for the first release!**

1. Create a new GitHub repository: `homebrew-tap`
   ```bash
   # On GitHub, create: juntaochi/homebrew-tap
   ```

2. Clone and set up the tap:
   ```bash
   git clone https://github.com/juntaochi/homebrew-tap.git
   cd homebrew-tap
   mkdir -p Formula
   ```

3. Initialize with README:
   ```bash
   cat > README.md <<'EOF'
   # Homebrew Tap for juntaochi

   This is a Homebrew tap for juntaochi's projects.

   ## Installation

   ```bash
   brew tap juntaochi/tap
   brew install amcli
   ```

   ## Available Formulae

   - **amcli** - Apple Music Command Line Interface
   EOF

   git add README.md
   git commit -m "Initial commit"
   git push
   ```

### Step 4: Update Homebrew Formula / 更新 Homebrew 配方

1. Download the release artifacts:
   ```bash
   cd /tmp
   VERSION="0.1.0"

   curl -LO "https://github.com/juntaochi/amcli/releases/download/v${VERSION}/amcli-v${VERSION}-arm64-apple-darwin.tar.gz"
   curl -LO "https://github.com/juntaochi/amcli/releases/download/v${VERSION}/amcli-v${VERSION}-x86_64-apple-darwin.tar.gz"
   ```

2. Calculate SHA256 checksums:
   ```bash
   ARM64_SHA256=$(shasum -a 256 "amcli-v${VERSION}-arm64-apple-darwin.tar.gz" | awk '{print $1}')
   X86_64_SHA256=$(shasum -a 256 "amcli-v${VERSION}-x86_64-apple-darwin.tar.gz" | awk '{print $1}')

   echo "ARM64 SHA256:  $ARM64_SHA256"
   echo "X86_64 SHA256: $X86_64_SHA256"
   ```

3. Update the formula in your tap repository:
   ```bash
   cd /path/to/homebrew-tap

   # Copy template and update
   cp /path/to/amcli/homebrew/amcli.rb Formula/amcli.rb

   # Replace placeholders (use the actual SHA256 values from step 2)
   sed -i '' "s/REPLACE_WITH_ARM64_SHA256/$ARM64_SHA256/" Formula/amcli.rb
   sed -i '' "s/REPLACE_WITH_X86_64_SHA256/$X86_64_SHA256/" Formula/amcli.rb
   ```

4. Test the formula locally:
   ```bash
   brew audit --strict Formula/amcli.rb
   brew install --build-from-source Formula/amcli.rb
   brew test amcli
   amcli --version
   ```

5. Commit and push:
   ```bash
   git add Formula/amcli.rb
   git commit -m "Update amcli to v${VERSION}"
   git push
   ```

### Step 5: Announce the Release / 发布公告

- [ ] Update project README if needed
- [ ] Post on social media / forums
- [ ] Notify users / contributors
- [ ] Update project website (if applicable)

## Post-Release Verification / 发布后验证

### Test Installation / 测试安装

1. **Test Homebrew installation:**
   ```bash
   # Fresh install
   brew uninstall amcli || true
   brew untap juntaochi/tap || true
   brew tap juntaochi/tap
   brew install amcli

   # Verify
   amcli --version
   which amcli
   ```

2. **Test binary download:**
   ```bash
   # Download and extract
   curl -LO "https://github.com/juntaochi/amcli/releases/download/v${VERSION}/amcli-v${VERSION}-arm64-apple-darwin.tar.gz"
   tar xzf "amcli-v${VERSION}-arm64-apple-darwin.tar.gz"
   ./amcli --version
   ```

### Monitor for Issues / 监控问题

- [ ] Check GitHub Issues for installation problems
- [ ] Monitor release download statistics
- [ ] Verify all documentation links work
- [ ] Test on different macOS versions if possible

## Rolling Back a Release / 回滚发布

If critical issues are found:

1. Delete the GitHub release (or mark as pre-release)
2. Delete the git tag:
   ```bash
   git tag -d "v${VERSION}"
   git push origin :refs/tags/"v${VERSION}"
   ```
3. Remove or deprecate the Homebrew formula
4. Fix issues and create a new patch release

## Version Numbering / 版本编号

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: New functionality (backwards compatible)
- **PATCH** version: Bug fixes (backwards compatible)

Examples:
- `0.1.0` → First release
- `0.1.1` → Bug fix
- `0.2.0` → New features
- `1.0.0` → Stable release

## Automation Ideas / 自动化建议

Future improvements to consider:

1. **Automated Homebrew formula updates:**
   - Create a GitHub Action in tap repository
   - Trigger on release webhook
   - Automatically update SHA256 and version

2. **Release notes automation:**
   - Use conventional commits
   - Generate CHANGELOG automatically
   - Parse commit messages for release notes

3. **Multi-platform testing:**
   - Test on multiple macOS versions
   - Automated installation tests
   - Integration test suite

## Troubleshooting / 故障排除

### Release workflow fails
- Check GitHub Actions logs
- Verify Rust toolchain is available
- Ensure all targets are installed

### Homebrew formula fails to install
- Test formula with `brew install --debug`
- Verify SHA256 checksums match
- Check that URLs are accessible
- Audit with `brew audit --strict`

### Binary doesn't run
- Check architecture matches (ARM64 vs x86_64)
- Verify macOS version compatibility
- Check for missing dependencies

## Resources / 资源

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
