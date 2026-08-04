class GroeponlineHerdr < Formula
  desc "Terminal workspace manager for AI coding agents"
  homepage "https://github.com/GroepOnline/herdr"
  version "0.7.6"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.7.6/herdr-linux-x86_64"
      sha256 "8f0785c5e9e471e03e7611d6b987b60bf1f9a7db0f25bec95c11f54e156a561a"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.7.6/herdr-linux-aarch64"
      sha256 "ee943dfdf577fe48d5430f6bd4631bbad67019719f6af57422bd345d015eb671"
    end
  end

  on_macos do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.7.6/herdr-macos-x86_64"
      sha256 "6347f7bf567d48a7617ff6c6b9f8d5ca13fe9b492ccd30ca4fda4c5a34a41dff"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.7.6/herdr-macos-aarch64"
      sha256 "4abe05e0858758b166e1e08bd57305363e151dfa65b1bd3cd9e1f99e5ca1dfcd"
    end
  end

  def install
    asset = if OS.mac?
      Hardware::CPU.arm? ? "herdr-macos-aarch64" : "herdr-macos-x86_64"
    else
      Hardware::CPU.arm? ? "herdr-linux-aarch64" : "herdr-linux-x86_64"
    end
    bin.install asset => "herdr"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/herdr --version")
  end
end
