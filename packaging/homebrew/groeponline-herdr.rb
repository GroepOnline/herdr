class GroeponlineHerdr < Formula
  desc "Terminal workspace manager for AI coding agents"
  homepage "https://github.com/GroepOnline/herdr"
  version "0.8.8"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.8/herdr-linux-x86_64"
      sha256 "4dbbe467ede16945b415e881133bd30b751b0283cf868c55d19cccd025117b6d"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.8/herdr-linux-aarch64"
      sha256 "85e994e01114e4d3d63733c9a3cda16d72951f3a67bfeee83d66a08a15b41bb0"
    end
  end

  on_macos do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.8/herdr-macos-x86_64"
      sha256 "ca22bfa9f4d805e097579b670432456efe3f8e36b09ee232646ae61002ad8e72"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.8/herdr-macos-aarch64"
      sha256 "ae7d42e3f75ece2af88e0337c759b238bcc4430d8f256530e6e9d0f5b1ba03d1"
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
