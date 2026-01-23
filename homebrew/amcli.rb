# AMCLI Homebrew Formula
# This is a template - actual formula will be in homebrew-tap repository
class Amcli < Formula
  desc "Apple Music Command Line Interface - A powerful TUI for controlling Apple Music"
  homepage "https://github.com/juntaochi/amcli"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-arm64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    else
      url "https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X86_64_SHA256"
    end
  end

  depends_on :macos

  def install
    bin.install "amcli"

    # Generate shell completions (if available)
    # output = Utils.safe_popen_read("#{bin}/amcli", "completions", "bash")
    # (bash_completion/"amcli").write output

    # output = Utils.safe_popen_read("#{bin}/amcli", "completions", "zsh")
    # (zsh_completion/"_amcli").write output

    # output = Utils.safe_popen_read("#{bin}/amcli", "completions", "fish")
    # (fish_completion/"amcli.fish").write output
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
