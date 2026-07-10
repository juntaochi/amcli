# typed: strict
# frozen_string_literal: true

# AMCLI Homebrew Formula
# This is a template - actual formula will be in homebrew-tap repository
class Amcli < Formula
  desc "Apple Music Command-line Interface - A powerful TUI for controlling Apple Music"
  homepage "https://github.com/juntaochi/amcli"
  version "0.3.1"
  license "MIT"

  depends_on :macos

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/juntaochi/amcli/releases/download/v0.3.1/amcli-v0.3.1-arm64-apple-darwin.tar.gz"
      sha256 "92a8c3ca4d7d445a86827cded0a8d46c8b39d565ac5f24aea065a4b218abaea2"
    else
      url "https://github.com/juntaochi/amcli/releases/download/v0.3.1/amcli-v0.3.1-x86_64-apple-darwin.tar.gz"
      sha256 "e7ef1f1a5543a619aebed93b8669b4b194397651df4d91a4ee48787439ae3f38"
    end
  end

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
