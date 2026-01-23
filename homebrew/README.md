# Homebrew Formula for AMCLI

This directory contains the Homebrew formula template for AMCLI.

## Setting Up Homebrew Distribution

### Option 1: Create a Homebrew Tap (Recommended for early releases)

1. **Create a new repository** named `homebrew-tap` or `homebrew-amcli`:
   ```bash
   # Create the repository on GitHub: juntaochi/homebrew-tap
   ```

2. **Add the formula** to the tap repository:
   ```bash
   git clone https://github.com/juntaochi/homebrew-tap
   cd homebrew-tap
   mkdir -p Formula
   cp ../amcli/homebrew/amcli.rb Formula/
   ```

3. **Update SHA256 hashes** after creating a release:
   ```bash
   # Download the release tarballs
   curl -LO https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-arm64-apple-darwin.tar.gz
   curl -LO https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-x86_64-apple-darwin.tar.gz

   # Calculate SHA256
   shasum -a 256 amcli-v0.1.0-arm64-apple-darwin.tar.gz
   shasum -a 256 amcli-v0.1.0-x86_64-apple-darwin.tar.gz

   # Update the formula with the actual SHA256 values
   ```

4. **Commit and push**:
   ```bash
   git add Formula/amcli.rb
   git commit -m "Add amcli v0.1.0"
   git push
   ```

5. **Users can then install with**:
   ```bash
   brew tap juntaochi/tap
   brew install amcli
   ```

### Option 2: Submit to homebrew-core (For stable releases)

Once your project is stable and has:
- Multiple releases
- Active maintenance
- Significant user base

You can submit to the official Homebrew repository:

1. Fork `homebrew/homebrew-core`
2. Add your formula to `Formula/amcli.rb`
3. Create a pull request

See: https://docs.brew.sh/Adding-Software-to-Homebrew

## Automated Updates

The release workflow (`.github/workflows/release.yml`) automatically:
1. Builds binaries for both Intel and Apple Silicon Macs
2. Creates GitHub releases
3. Generates SHA256 checksums
4. Uploads release artifacts

To update the formula automatically, you can:
1. Set up a GitHub Action in your tap repository
2. Use `brew bump-formula-pr` command locally

## Testing the Formula Locally

Before publishing:

```bash
# Install from local formula
brew install --build-from-source ./homebrew/amcli.rb

# Or test with a tap
brew tap juntaochi/tap
brew install --HEAD amcli

# Audit the formula
brew audit --strict amcli
brew test amcli
```

## Version Updates

When releasing a new version:

1. Update `version` in the formula
2. Update URLs with new version number
3. Download new release tarballs and calculate SHA256
4. Update SHA256 values in the formula
5. Commit and push to tap repository

This can be automated with a script or GitHub Action.
