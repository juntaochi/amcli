# AMCLI Development Setup Guide (Rust)

## Prerequisites

### 1. Install Rust

**macOS (recommended):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Or using Homebrew:**
```bash
brew install rust
```

**Verify installation:**
```bash
rustc --version  # Should show Rust 1.75 or higher
cargo --version  # Cargo is Rust's package manager
```

### 2. Initialize Project

From the project root:
```bash
cd /Users/jac/Repos/amcli

# Cargo.toml is already created, just build to fetch dependencies
cargo build
```

### 3. Development Tools

```bash
# Install Rust formatter
rustup component add rustfmt

# Install Rust linter
rustup component add clippy

# Install cargo-watch for auto-reload during development
cargo install cargo-watch

# Install cargo-edit for managing dependencies
cargo install cargo-edit
```

## Quick Start

### Build the Project

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (optimized)
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Run the Application

```bash
# Debug mode
cargo run

# Release mode (optimized)
cargo run --release
```

### Development Mode with Auto-Reload

```bash
# Automatically recompile and run on file changes
cargo watch -x run
```

## Development Workflow

1. **Create a new branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Write code**
   - Follow Rust best practices
   - Write tests for new features
   - Format code: `cargo fmt`
   - Lint code: `cargo clippy`

3. **Test your changes**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt -- --check
   ```

4. **Build and run**
   ```bash
   cargo run
   ```

5. **Commit and push**
   ```bash
   git add .
   git commit -m "feat: your feature description"
   git push origin feature/your-feature-name
   ```

## Project Structure

```
amcli/
├── src/
│   ├── main.rs              # Application entry point
│   ├── player/
│   │   ├── mod.rs           # Player trait and types
│   │   └── apple_music.rs   # Apple Music implementation
│   ├── ui/
│   │   └── mod.rs           # TUI using Ratatui
│   ├── lyrics/
│   │   └── mod.rs           # Lyrics providers
│   ├── artwork/
│   │   └── mod.rs           # Album art processing
│   └── config/
│       └── mod.rs           # Configuration management
├── scripts/
│   └──applescript/         # AppleScript helpers
├── configs/
│   └── config.example.toml  # Example configuration
├── Cargo.toml              # Rust project manifest
├── Cargo.lock              # Dependency lock file
└── target/                 # Build output (ignored by git)
```

## Configuration

Create a config file at `~/.config/amcli/config.toml`:

```bash
mkdir -p ~/.config/amcli
cp configs/config.example.toml ~/.config/amcli/config.toml
```

Edit the config file as needed.

## macOS Permissions

AMCLI needs certain permissions to function properly:

1. **Accessibility Access** (for AppleScript)
   - System Settings → Privacy & Security → Accessibility
   - Add Terminal or your terminal emulator
   - Allow the app to control your computer

2. **Full Disk Access** (optional, for better integration)
   - System Settings → Privacy & Security → Full Disk Access
   - Add Terminal

## Cargo Commands Reference

### Building
```bash
cargo build                 # Debug build
cargo build --release       # Release build (optimized)
cargo clean                 # Clean build artifacts
```

### Running
```bash
cargo run                   # Run in debug mode
cargo run --release         # Run in release mode
cargo run -- --help         # Pass arguments to the app
```

### Testing
```bash
cargo test                  # Run all tests
cargo test --release        # Run tests in release mode
cargo test test_name        # Run specific test
cargo test -- --nocapture   # Run tests with println! output
```

### Code Quality
```bash
cargo fmt                   # Format code
cargo clippy                # Run linter
cargo clippy -- -D warnings # Fail on warnings
```

### Dependencies
```bash
cargo add <crate>           # Add a dependency
cargo rm <crate>            # Remove a dependency
cargo update                # Update dependencies
```

### Documentation
```bash
cargo doc --open            # Generate and open docs
```

## Troubleshooting

### Rust not found after installation
```bash
# Add Rust to your PATH
source $HOME/.cargo/env

# Or add to shell profile
echo 'source $HOME/.cargo/env' >> ~/.zshrc
source ~/.zshrc
```

### Compilation errors
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build
```

### AppleScript permission denied
- Go to System Settings → Privacy & Security → Accessibility
- Add and enable your terminal application

## Next Steps

1. Review [PROJECT_SPEC.md](PROJECT_SPEC.md) for detailed specifications
2. Check [TODO.md](TODO.md) for implementation checklist
3. Start with Phase 1 implementation (Core Foundation)
4. Join discussions in GitHub Issues

## Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Official Rust programming book
- [Ratatui Tutorial](https://ratatui.rs/) - TUI framework documentation
- [Tokio Guide](https://tokio.rs/tokio/tutorial) - Async runtime tutorial
- [AppleScript Language Guide](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn Rust with examples

## Performance Notes

**Debug vs Release:**
- **Debug builds** (`cargo build`): Faster compilation, slower execution, includes debug symbols
- **Release builds** (`cargo build --release`): Slower compilation, optimized execution (60%+ faster)  

For development, use debug builds. For testing performance or distribution, use release builds.

## IDE Setup (VS Code)

Recommended extensions:
```bash
# rust-analyzer (essential)
code --install-extension rust-lang.rust-analyzer

# Even Better TOML
code --install-extension tamasfe.even-better-toml

# CodeLLDB (debugger)
code --install-extension vadimcn.vscode-lldb
```

Settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```
