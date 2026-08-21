class GroeponlineHerdr < Formula
  desc "Terminal workspace manager for AI coding agents"
  homepage "https://github.com/GroepOnline/herdr"
  version "0.8.6"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.6/herdr-linux-x86_64"
      sha256 "f8163eb1e4883cc0650aec9a59d8e9f99824f25b210daa3fbe48221ee1f578f7"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.6/herdr-linux-aarch64"
      sha256 "086eb5cdce8739c611bda34d5768891ea9629805fc7cd0587fcfb68ceba55f75"
    end
  end

  on_macos do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.6/herdr-macos-x86_64"
      sha256 "073ccb2dbb58c7519668e7aeca1ac5cb9a1c1f00497f32ad4ffe69cb15d6c9fd"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.6/herdr-macos-aarch64"
      sha256 "daa61cacaee3857b940e67c5c2f409bdbea3b9dcd2d8643a0d1bf142cce9bb10"
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
