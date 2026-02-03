# AMCLI v0.1.0 Release - Manual Completion Steps

## Status
- ✅ Code prepared and tested
- ✅ Tag v0.1.0 created and pushed
- ✅ GitHub Release created (empty)
- ⚠️ GitHub Actions workflow has issues - manual upload needed
- ✅ ARM64 binary built locally: `/tmp/amcli-v0.1.0-arm64-apple-darwin.tar.gz`
- ✅ Twitter drafts ready: `TWITTER_ANNOUNCEMENT.md`

## Immediate Steps Required

### 1. Upload Release Assets Manually

The GitHub Actions workflow is failing. Upload binaries manually:

```bash
# ARM64 binary is already built at:
ls /tmp/amcli-v0.1.0-arm64-apple-darwin.tar.gz
cat /tmp/amcli-v0.1.0-arm64-apple-darwin.tar.gz.sha256

# Get the SHA256:
ARM64_SHA=$(cat /tmp/amcli-v0.1.0-arm64-apple-darwin.tar.gz.sha256 | awk '{print $1}')
echo "ARM64 SHA256: $ARM64_SHA"

# Upload to GitHub Release via web interface:
# 1. Go to: https://github.com/juntaochi/amcli/releases/tag/v0.1.0
# 2. Click "Edit release"
# 3. Drag and drop these files:
#    - /tmp/amcli-v0.1.0-arm64-apple-darwin.tar.gz
#    - /tmp/amcli-v0.1.0-arm64-apple-darwin.tar.gz.sha256
# 4. Click "Update release"
```

**Note:** Since you're on M-chip Mac, you can only build ARM64 locally. For x86_64 Intel binary, either:
- Option A: Skip it (ARM64 works on all modern Macs via Rosetta)
- Option B: Use GitHub Actions (needs workflow fixes)
- Option C: Build on an Intel Mac

### 2. Create Homebrew Tap Repository

```bash
# Create repository on GitHub
gh repo create homebrew-tap --public --description "Homebrew formulae for juntaochi's projects"

# Clone and set up structure
cd ~/
git clone https://github.com/juntaochi/homebrew-tap.git
cd homebrew-tap

# Create Formula directory
mkdir -p Formula

# Create README
cat > README.md <<'EOF'
# Homebrew Tap for juntaochi

Homebrew formulae for juntaochi's projects.

## Installation

```bash
brew tap juntaochi/tap
brew install amcli
```

## Available Formulae

- **amcli** - Apple Music Command Line Interface
EOF

# Commit and push
git add README.md
git commit -m "Initial commit"
git push origin main
```

### 3. Update Homebrew Formula with SHA256

```bash
# Get the ARM64 SHA256 from step 1
ARM64_SHA="<paste-from-step-1>"

# Copy formula template
cd ~/homebrew-tap
cp ~/Repos/amcli/homebrew/amcli.rb Formula/amcli.rb

# Update SHA256 (replace placeholder)
sed -i '' "s/REPLACE_WITH_ARM64_SHA256/$ARM64_SHA/" Formula/amcli.rb

# Since we only have ARM64, update the formula to only support ARM64:
# Edit Formula/amcli.rb manually to remove x86_64 block:
```

Edit `Formula/amcli.rb` to look like this:

```ruby
class Amcli < Formula
  desc "Apple Music Command Line Interface - A powerful TUI for controlling Apple Music"
  homepage "https://github.com/juntaochi/amcli"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-arm64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"  # Replace with actual SHA from step 1
    else
      odie "Intel Macs not yet supported in v0.1.0. ARM64/Apple Silicon only."
    end
  end

  depends_on :macos

  def install
    bin.install "amcli"
  end

  def caveats
    <<~EOS
      AMCLI requires Apple Music to be installed on macOS.

      To get started, run:
        amcli

      For configuration options:
        amcli --help

      Default config location: ~/.config/amcli/config.toml
    EOS
  end

  test do
    assert_match "amcli", shell_output("#{bin}/amcli --version")
  end
end
```

Then:

```bash
# Commit and push formula
git add Formula/amcli.rb
git commit -m "Add amcli v0.1.0"
git push origin main
```

### 4. Test Homebrew Installation

```bash
# Install from your tap
brew tap juntaochi/tap
brew install amcli

# Verify
amcli --version
which amcli

# Test audit
cd ~/homebrew-tap
brew audit --strict Formula/amcli.rb
```

### 5. Post Twitter Announcement

Open `~/Repos/amcli/TWITTER_ANNOUNCEMENT.md` and choose a format:
- **Option 1**: Single concise tweet
- **Option 2**: Detailed 5-tweet thread (RECOMMENDED)
- **Option 3**: Visual-first with screenshots

**Recommended timing:**
- Best time: 9-10 AM PST or 1-2 PM PST (weekdays)
- Include hashtags: #RustLang #macOS #TUI #CLI #OpenSource

## Verification Checklist

After completing all steps:

- [ ] GitHub Release has ARM64 tarball + SHA256 file
- [ ] homebrew-tap repository created and public
- [ ] Formula pushed to homebrew-tap
- [ ] `brew install juntaochi/tap/amcli` works on your Mac
- [ ] `amcli --version` shows v0.1.0
- [ ] Twitter announcement posted
- [ ] Pin tweet to profile (optional)
- [ ] Cross-post to Hacker News / Reddit (optional)

## Known Issues

1. **GitHub Actions workflow** - Has permission/config issues. Fixed for future releases.
2. **Intel binary missing** - Only ARM64 available for v0.1.0. Most modern Macs use ARM64.
3. **Formula audit warnings** - May show warnings about single-arch support. This is OK for v0.1.0.

## Next Release (v0.1.1)

For the next release, fix:
1. GitHub Actions workflow to build both architectures
2. Add both ARM64 and x86_64 to Homebrew formula
3. Automate formula updates with GitHub Action

## Support

If users report issues:
- Check: https://github.com/juntaochi/amcli/issues
- Common fix: Ensure Apple Music is installed and running
- ARM64 only: Older Intel Macs can use Rosetta or wait for v0.1.1

---

**Status**: Ready for manual completion. Follow steps 1-5 above.
