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
      sha256 "d63737ba3669d9b73baf95d7b2378f8d6d493c4e42995cd0d87abf2dc86b618e"
    else
      url "https://github.com/juntaochi/amcli/releases/download/v0.1.0/amcli-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "cb7e0f14e002fa976717fadb31483047cc4075de24d1baaacc89343f3c03c574"
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
