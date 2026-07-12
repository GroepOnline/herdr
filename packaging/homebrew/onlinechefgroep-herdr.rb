# Homebrew formula for the OnlineChefGroep herdr fork.
# Copy to OnlineChefGroep/homebrew-tap/Formula/onlinechefgroep-herdr.rb
# Install: brew tap OnlineChefGroep/tap && brew install onlinechefgroep-herdr
class OnlinechefgroepHerdr < Formula
  desc "Herdr fork with OnlineChefGroep agent manifests"
  homepage "https://github.com/OnlineChefGroep/herdr"
  license "MIT"
  version "0.7.3"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-macos-aarch64"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    else
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-macos-x86_64"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-linux-aarch64"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    else
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-linux-x86_64"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    end
  end

  def install
    binary = Hardware::CPU.arm? ? (OS.mac? ? "herdr-macos-aarch64" : "herdr-linux-aarch64") : (OS.mac? ? "herdr-macos-x86_64" : "herdr-linux-x86_64")
    bin.install binary => "herdr"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/herdr --version")
  end
end
