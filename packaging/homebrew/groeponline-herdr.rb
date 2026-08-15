class GroeponlineHerdr < Formula
  desc "Terminal workspace manager for AI coding agents"
  homepage "https://github.com/GroepOnline/herdr"
  version "0.8.1"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.1/herdr-linux-x86_64"
      sha256 "85087e66440fca2ee1f4ce61249044c888e7bedf0776e6ed09657cf80ad6b5e4"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.1/herdr-linux-aarch64"
      sha256 "0e558767f5f640d788c977b8a2938ea99b85f305abc911e2c9006523a4ffa197"
    end
  end

  on_macos do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.1/herdr-macos-x86_64"
      sha256 "c3e04bc4f69ea5392ef3b9f40b98ffed25060e8c80b63c334cf5fd1f84b9f678"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.1/herdr-macos-aarch64"
      sha256 "2080a7fc88f7b96df834d875813aab3198bf74f0da1e2b40140a86ddc81bb9a4"
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
