# Add Intel Mac Support to v0.1.0 Release

## Context

### Original Request
用户发现 Intel Mac 无法通过 Homebrew 下载 amcli，需要允许 Intel Mac 下载。

### Interview Summary
**Key Discussions**:
- Solution approach: 方案 B - 手动补充 Intel binary（快速修复 v0.1.0）
- Test strategy: 跳过测试，直接发布（信任交叉编译结果）
- Formula update scope: 同时更新 amcli 仓库模板 AND homebrew-tap 仓库

**Research Findings**:
- Cross-compilation from Apple Silicon to Intel is standard practice in Rust ecosystem
- GitHub CLI (v2.85.0) installed and ready for asset upload
- Current release v0.1.0 only has ARM64 assets (2 downloads so far)
- Homebrew-tap formula currently shows error: "Intel Macs not yet supported in v0.1.0"

### Metis Review
**Identified Gaps** (addressed in this plan):
1. **Missing verification step**: Added Rosetta test to confirm binary executes
2. **Tarball structure assumption**: Verified binary should be at tarball root
3. **SHA256 format clarity**: Confirmed BSD format with `shasum -a 256`
4. **Minimal testing on Intel**: Added `arch -x86_64 ./amcli --version` verification
5. **Formula update clarity**: Specified exact replacement in both locations

---

## Work Objectives

### Core Objective
Enable Intel Mac users to install amcli v0.1.0 via Homebrew by adding missing x86_64 binary to GitHub Release and updating formula.

### Concrete Deliverables
1. Cross-compiled x86_64 binary tarball uploaded to v0.1.0 release
2. SHA256 checksum file for x86_64 binary
3. Updated Homebrew formula in `juntaochi/homebrew-tap` with x86_64 support
4. Updated formula template in `amcli/homebrew/amcli.rb`

### Definition of Done
- [ ] `gh release view v0.1.0` shows 4 assets total (2 ARM64 + 2 x86_64)
- [ ] Homebrew formula has valid x86_64 URL and SHA256
- [ ] Intel Mac users can run `brew install juntaochi/tap/amcli` successfully

### Must Have
- x86_64 binary built via cross-compilation
- Binary verified to execute (at minimum `--version` works)
- Tarball naming matches existing pattern: `amcli-v0.1.0-x86_64-apple-darwin.tar.gz`
- SHA256 in correct format (BSD `shasum` output)
- Both files uploaded to v0.1.0 release
- Formula updated in homebrew-tap repository

### Must NOT Have (Guardrails)
- **NO version bump** - this is a patch to v0.1.0, not v0.1.1
- **NO modification to ARM64 assets** - leave existing files untouched
- **NO release metadata changes** - don't edit release notes, title, or tag
- **NO workflow fixes** - release.yml improvements are a separate task
- **NO universal binary creation** - separate arch-specific binaries are correct for Homebrew
- **NO testing beyond basic verification** - user explicitly chose to skip extensive testing

---

## Verification Strategy

### Manual Verification (No Test Infrastructure)

**CRITICAL**: Without automated tests, manual verification MUST be exhaustive.

Each TODO includes detailed verification procedures:

**By Deliverable Type:**

| Type | Verification Tool | Procedure |
|------|------------------|-----------|
| **Binary** | `file` + `arch` commands | Verify architecture and executability |
| **Tarball** | `tar tzf` | Verify structure matches ARM64 version |
| **SHA256** | `shasum -c` | Verify checksum format and correctness |
| **GitHub Upload** | `gh release view` | Verify assets appear in release |
| **Formula** | `brew audit --strict` | Verify syntax and SHA256 validity |

**Evidence Required:**
- Commands run with actual output
- File listings showing correct names and sizes
- Terminal output showing successful executions
- `gh` command outputs confirming uploads

---

## Task Flow

```
Task 0 (Install target)
  ↓
Task 1 (Build x86_64)
  ↓
Task 2 (Verify binary)
  ↓
Task 3 (Create tarball + SHA256)
  ↓
Task 4 (Upload to GitHub)
  ↓
Task 5 (Update homebrew-tap) ← depends on 4
  ↓
Task 6 (Update local template) ← can run parallel with 5
```

## Parallelization

| Group | Tasks | Reason |
|-------|-------|--------|
| Sequential | 0 → 1 → 2 → 3 → 4 | Each depends on previous output |
| Parallel | 5, 6 | Both are formula updates, independent |

| Task | Depends On | Reason |
|------|------------|--------|
| 1 | 0 | Need x86_64 target installed before building |
| 2 | 1 | Need binary to test |
| 3 | 2 | Need verified binary to package |
| 4 | 3 | Need tarball + SHA256 to upload |
| 5 | 4 | Need uploaded assets and SHA256 to update formula |
| 6 | 4 | Need SHA256 from step 3 |

---

## TODOs

- [ ] 0. Install x86_64-apple-darwin target

  **What to do**:
  - Run `rustup target add x86_64-apple-darwin`
  - Verify installation successful

  **Must NOT do**:
  - Do NOT install any other targets
  - Do NOT modify Rust toolchain version

  **Parallelizable**: NO (first task, nothing depends on it initially)

  **References**:
  
  **Pattern References**:
  - GitHub search results show standard practice across Rust projects
  - Examples: formatjs, tun2proxy, sshx all use identical command

  **Documentation References**:
  - Rustup docs: Cross-compilation targets for macOS
  
  **Acceptance Criteria**:

  **Manual Execution Verification**:
  - [ ] Using terminal:
    - Command: `rustup target add x86_64-apple-darwin`
    - Expected output contains: `info: downloading component 'rust-std' for 'x86_64-apple-darwin'`
    - Expected output contains: `info: installing component 'rust-std' for 'x86_64-apple-darwin'`
    - Exit code: 0

  - [ ] Verification command:
    - Command: `rustup target list --installed | grep x86_64-apple-darwin`
    - Expected output: `x86_64-apple-darwin`

  **Evidence Required**:
  - [ ] Command output captured showing successful installation

  **Commit**: NO (no code changes, only local toolchain)

---

- [ ] 1. Build x86_64 binary via cross-compilation

  **What to do**:
  - Run `cargo build --release --target x86_64-apple-darwin`
  - Build will output to `target/x86_64-apple-darwin/release/amcli`
  - Wait for build completion (may take 2-5 minutes)

  **Must NOT do**:
  - Do NOT build with debug profile
  - Do NOT modify Cargo.toml or build flags
  - Do NOT build for other targets

  **Parallelizable**: NO (depends on Task 0)

  **References**:
  
  **Pattern References**:
  - `.github/workflows/release.yml:88` - GitHub Actions uses identical build command
  - Real-world examples from formatjs, tun2proxy show same pattern

  **Build Configuration**:
  - `Cargo.toml:64-68` - Release profile with `opt-level = 3`, `lto = true`, `strip = true`

  **Acceptance Criteria**:

  **Manual Execution Verification**:
  - [ ] Using terminal:
    - Command: `cargo build --release --target x86_64-apple-darwin`
    - Expected: Compilation completes without errors
    - Expected output contains: `Finished release [optimized] target(s)`
    - Exit code: 0

  - [ ] Binary exists check:
    - Command: `ls -lh target/x86_64-apple-darwin/release/amcli`
    - Expected: File exists, size approximately 3-4 MB (similar to ARM64)

  - [ ] Architecture verification:
    - Command: `file target/x86_64-apple-darwin/release/amcli`
    - Expected output contains: `Mach-O 64-bit executable x86_64`
    - Must NOT contain: `arm64` or `aarch64`

  **Evidence Required**:
  - [ ] Build log showing success
  - [ ] `file` command output confirming x86_64 architecture
  - [ ] File size comparison with ARM64 version

  **Commit**: NO (binary is build artifact, not source)

---

- [ ] 2. Verify x86_64 binary executes under Rosetta

  **What to do**:
  - Test binary runs on Apple Silicon using Rosetta 2 emulation
  - Run `arch -x86_64 target/x86_64-apple-darwin/release/amcli --version`
  - Verify version output is correct

  **Must NOT do**:
  - Do NOT run without `arch -x86_64` prefix (will fail on ARM Mac)
  - Do NOT skip this verification (risk uploading broken binary)

  **Parallelizable**: NO (depends on Task 1)

  **References**:
  
  **Pattern References**:
  - Metis recommendation: Minimal verification to prevent uploading broken binary
  
  **Project Context**:
  - Current version: `0.1.0` (from `Cargo.toml:3`)
  - Expected output: `amcli 0.1.0` or similar

  **Acceptance Criteria**:

  **Manual Execution Verification**:
  - [ ] Using terminal with Rosetta:
    - Command: `arch -x86_64 target/x86_64-apple-darwin/release/amcli --version`
    - Expected output contains: `amcli` and `0.1.0`
    - Exit code: 0
    - Execution time: < 2 seconds (proves binary loads and runs)

  - [ ] Verify running under correct architecture:
    - During execution, binary should be running as x86_64 process
    - If binary shows version successfully, architecture emulation works

  **Evidence Required**:
  - [ ] Terminal screenshot or output showing successful `--version` execution
  - [ ] Output confirms version matches expected `0.1.0`

  **Commit**: NO (verification step only)

---

- [ ] 3. Create tarball and SHA256 checksum

  **What to do**:
  - Navigate to build output directory
  - Create gzip tarball with binary at root (matches ARM64 structure)
  - Generate SHA256 checksum in BSD format
  - Verify checksum file format matches existing ARM64 .sha256 file

  **Must NOT do**:
  - Do NOT include subdirectories in tarball (binary must be at root)
  - Do NOT use `sha256sum` (GNU format) - use `shasum -a 256` (BSD format)
  - Do NOT modify file naming pattern

  **Parallelizable**: NO (depends on Task 2)

  **References**:
  
  **Pattern References**:
  - `.github/workflows/release.yml:93-94` - Exact commands from CI workflow
  - Existing ARM64 tarball structure (binary at root, no subdirs)

  **File Naming Pattern**:
  - Format: `amcli-v{VERSION}-{ARCH}-apple-darwin.tar.gz`
  - Example: `amcli-v0.1.0-x86_64-apple-darwin.tar.gz`

  **SHA256 Format** (from explore agent):
  - BSD format: `{hash}  {filename}` (two spaces between)
  - Generated by: `shasum -a 256 {file} > {file}.sha256`

  **Acceptance Criteria**:

  **Manual Execution Verification**:
  - [ ] Navigate to build directory:
    - Command: `cd target/x86_64-apple-darwin/release`
    - Verify: Current directory contains `amcli` binary

  - [ ] Create tarball:
    - Command: `tar czf amcli-v0.1.0-x86_64-apple-darwin.tar.gz amcli`
    - Expected: File created successfully
    - Verify structure: `tar tzf amcli-v0.1.0-x86_64-apple-darwin.tar.gz`
    - Expected output: `amcli` (binary at root, not in subdirectory)

  - [ ] Generate SHA256:
    - Command: `shasum -a 256 amcli-v0.1.0-x86_64-apple-darwin.tar.gz > amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256`
    - Verify file created: `ls -l amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256`
    - Check content: `cat amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256`
    - Expected format: `{64-char-hex}  amcli-v0.1.0-x86_64-apple-darwin.tar.gz`

  - [ ] Verify checksum works:
    - Command: `shasum -a 256 -c amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256`
    - Expected output: `amcli-v0.1.0-x86_64-apple-darwin.tar.gz: OK`

  - [ ] Size comparison:
    - Command: `ls -lh amcli-v0.1.0-x86_64-apple-darwin.tar.gz`
    - Expected: Size approximately 3-4 MB (similar to ARM64 version: 3.6MB)

  **Evidence Required**:
  - [ ] `tar tzf` output showing binary at root
  - [ ] SHA256 file content showing correct format
  - [ ] Verification output showing checksum OK
  - [ ] File sizes confirming reasonable tarball size

  **Commit**: NO (build artifacts)

---

- [ ] 4. Upload assets to GitHub Release v0.1.0

  **What to do**:
  - Upload both tarball and SHA256 file to existing v0.1.0 release
  - Use `gh release upload` with `--clobber` flag
  - Verify both files appear in release assets

  **Must NOT do**:
  - Do NOT create new release or tag
  - Do NOT modify existing ARM64 assets
  - Do NOT edit release notes or metadata
  - Do NOT use `--clobber` without verifying file names are correct

  **Parallelizable**: NO (depends on Task 3)

  **References**:
  
  **Pattern References**:
  - GitHub search examples: risingwave, kata-containers, bun all use `gh release upload {tag} {files} --clobber`

  **Current Release Info**:
  - Tag: `v0.1.0`
  - Existing assets: 2 (ARM64 tarball + SHA256)
  - After upload: Should have 4 assets total

  **Acceptance Criteria**:

  **Manual Execution Verification**:
  - [ ] Upload command:
    - Command: `gh release upload v0.1.0 target/x86_64-apple-darwin/release/amcli-v0.1.0-x86_64-apple-darwin.tar.gz target/x86_64-apple-darwin/release/amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256 --clobber`
    - Expected output contains: `Successfully uploaded`
    - Exit code: 0

  - [ ] Verify upload via GitHub CLI:
    - Command: `gh release view v0.1.0 --json assets --jq '.assets[] | {name: .name, size: .size}'`
    - Expected: 4 assets total
    - Expected assets:
      - `amcli-v0.1.0-arm64-apple-darwin.tar.gz` (existing)
      - `amcli-v0.1.0-arm64-apple-darwin.tar.gz.sha256` (existing)
      - `amcli-v0.1.0-x86_64-apple-darwin.tar.gz` (NEW)
      - `amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256` (NEW)

  - [ ] Verify via web UI:
    - Open: `https://github.com/juntaochi/amcli/releases/tag/v0.1.0`
    - Verify: All 4 assets appear in release downloads section
    - Verify: x86_64 tarball size is reasonable (3-4 MB)

  **Evidence Required**:
  - [ ] `gh release view` output showing all 4 assets
  - [ ] Screenshot of GitHub release page showing new assets
  - [ ] Download count for new assets shows 0 initially

  **Commit**: NO (GitHub release assets are not in repo)

---

- [ ] 5. Update Homebrew formula in homebrew-tap repository

  **What to do**:
  - Clone or pull `juntaochi/homebrew-tap` repository
  - Update `Formula/amcli.rb` to replace `odie` error with proper x86_64 support
  - Replace `REPLACE_WITH_X86_64_SHA256` with actual SHA256 from step 3
  - Verify formula syntax with `brew audit --strict`
  - Commit and push changes

  **Must NOT do**:
  - Do NOT modify ARM64 section (already has correct SHA256)
  - Do NOT change formula structure or dependencies
  - Do NOT modify version number (stays at 0.1.0)
  - Do NOT push to homebrew-core (only the tap)

  **Parallelizable**: YES (with Task 6, both are formula updates)

  **References**:
  
  **Repository Info**:
  - Repo: `https://github.com/juntaochi/homebrew-tap`
  - Branch: `main`
  - File: `Formula/amcli.rb`

  **Current Formula** (from web fetch):
  - Lines 9-16 currently have:
    ```ruby
    if Hardware::CPU.arm?
      url "https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-arm64-apple-darwin.tar.gz"
      sha256 "d63737ba3669d9b73baf95d7b2378f8d6d493c4e42995cd0d87abf2dc86b618e"
    else
      odie "Intel Macs not yet supported in v0.1.0. ARM64/Apple Silicon only."
    end
    ```

  **Template Formula** (from local repo):
  - `homebrew/amcli.rb:14-15` shows correct structure with placeholders

  **Acceptance Criteria**:

  **Manual Execution Verification**:
  - [ ] Clone or update tap repository:
    - Command: `git clone https://github.com/juntaochi/homebrew-tap.git /tmp/homebrew-tap` (or `git pull` if already cloned)
    - Verify: Repository cloned successfully
    - Navigate: `cd /tmp/homebrew-tap`

  - [ ] Get SHA256 from step 3:
    - Command: `cat ~/Repos/amcli/target/x86_64-apple-darwin/release/amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256 | awk '{print $1}'`
    - Save output for next step (e.g., `X86_SHA="..."`)

  - [ ] Update formula file:
    - Edit: `Formula/amcli.rb`
    - Replace line with `odie "Intel Macs not yet supported..."` with:
      ```ruby
      url "https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "{ACTUAL_SHA256_FROM_STEP_3}"
      ```
    - Verify ARM64 section remains unchanged

  - [ ] Verify formula syntax:
    - Command: `brew audit --strict Formula/amcli.rb`
    - Expected output: No errors or warnings
    - If warnings appear about single-arch support, ignore (we now have both)

  - [ ] Commit and push:
    - Command: `git add Formula/amcli.rb`
    - Command: `git commit -m "Add x86_64 (Intel Mac) support to v0.1.0"`
    - Command: `git push origin main`
    - Expected: Push successful

  - [ ] Verify via GitHub:
    - Open: `https://github.com/juntaochi/homebrew-tap/blob/main/Formula/amcli.rb`
    - Verify: Changes appear in GitHub web UI
    - Verify: Both ARM64 and x86_64 sections present with valid SHA256

  **Evidence Required**:
  - [ ] `brew audit` output showing no errors
  - [ ] `git diff` showing the exact changes made
  - [ ] GitHub commit showing successful push
  - [ ] Web UI showing updated formula

  **Commit**: YES (to homebrew-tap repo)
  - Message: `Add x86_64 (Intel Mac) support to v0.1.0`
  - Files: `Formula/amcli.rb`
  - Pre-commit: `brew audit --strict Formula/amcli.rb` → passes

---

- [ ] 6. Update local formula template in amcli repository

  **What to do**:
  - Update `homebrew/amcli.rb` template in amcli repository
  - Replace `REPLACE_WITH_X86_64_SHA256` placeholder with actual SHA256
  - Keep this in sync with homebrew-tap for future releases
  - Commit as documentation update

  **Must NOT do**:
  - Do NOT modify ARM64 placeholder (keep for reference)
  - Do NOT change formula structure
  - Do NOT update version number yet

  **Parallelizable**: YES (with Task 5, independent formula updates)

  **References**:
  
  **File Location**:
  - `homebrew/amcli.rb` in amcli repository
  - Current line 15: `sha256 "REPLACE_WITH_X86_64_SHA256"`

  **Purpose**:
  - This is a template/reference for future releases
  - Keeping it updated prevents confusion
  - Documents the working formula structure

  **Acceptance Criteria**:

  **Manual Execution Verification**:
  - [ ] Navigate to amcli repo:
    - Command: `cd ~/Repos/amcli`
    - Verify: In correct repository

  - [ ] Get SHA256 (same as Task 5):
    - Command: `cat target/x86_64-apple-darwin/release/amcli-v0.1.0-x86_64-apple-darwin.tar.gz.sha256 | awk '{print $1}'`
    - Save value for replacement

  - [ ] Update template file:
    - Edit: `homebrew/amcli.rb`
    - Find line 15: `sha256 "REPLACE_WITH_X86_64_SHA256"`
    - Replace with: `sha256 "{ACTUAL_SHA256}"`
    - Verify line 12 (ARM64) remains as placeholder or actual value

  - [ ] Verify changes:
    - Command: `git diff homebrew/amcli.rb`
    - Expected: Only line 15 modified
    - Expected: Valid 64-character hex SHA256 inserted

  - [ ] Commit:
    - Command: `git add homebrew/amcli.rb`
    - Command: `git commit -m "docs(homebrew): update x86_64 SHA256 for v0.1.0"`
    - Command: `git push origin main`
    - Expected: Commit pushed successfully

  **Evidence Required**:
  - [ ] `git diff` showing one-line change
  - [ ] Commit hash from successful push
  - [ ] Updated file visible on GitHub

  **Commit**: YES (to amcli repo)
  - Message: `docs(homebrew): update x86_64 SHA256 for v0.1.0`
  - Files: `homebrew/amcli.rb`
  - Pre-commit: None (documentation change)

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 5 | `Add x86_64 (Intel Mac) support to v0.1.0` | Formula/amcli.rb (in homebrew-tap) | brew audit --strict |
| 6 | `docs(homebrew): update x86_64 SHA256 for v0.1.0` | homebrew/amcli.rb (in amcli) | git diff |

---

## Success Criteria

### Verification Commands
```bash
# Verify GitHub release has all assets
gh release view v0.1.0 --json assets --jq '.assets[].name'
# Expected: 4 assets (2 ARM64 + 2 x86_64)

# Verify Homebrew formula (in tap repo)
cd /tmp/homebrew-tap && brew audit --strict Formula/amcli.rb
# Expected: No errors

# Verify formula has both architectures
curl -s https://raw.githubusercontent.com/juntaochi/homebrew-tap/main/Formula/amcli.rb | grep -A 2 "Hardware::CPU"
# Expected: Both ARM64 and x86_64 sections present
```

### Final Checklist
- [ ] All 4 assets present in v0.1.0 release
- [ ] x86_64 tarball is approximately same size as ARM64 (3-4 MB)
- [ ] SHA256 checksums are valid 64-character hex strings
- [ ] Homebrew formula passes `brew audit --strict`
- [ ] Formula has both ARM64 and x86_64 support
- [ ] Both homebrew-tap and amcli template formulas updated
- [ ] No errors in GitHub release upload
- [ ] Intel Mac users can run `brew install juntaochi/tap/amcli`
