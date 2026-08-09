class GroeponlineHerdr < Formula
  desc "Terminal workspace manager for AI coding agents"
  homepage "https://github.com/GroepOnline/herdr"
  version "0.8.0"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.0/herdr-linux-x86_64"
      sha256 "e66a473fd8c350d148b91b5173a6335f5c56f04e068c42722aab65e719bfad05"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.0/herdr-linux-aarch64"
      sha256 "6db980fa072c0efe3611891b1a8b5431e51b304a9874c307509b1113db388583"
    end
  end

  on_macos do
    on_intel do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.0/herdr-macos-x86_64"
      sha256 "24217f2cd1474b83791ca97488880b220f88e70b6413d5117b7d18646b79464b"
    end
    on_arm do
      url "https://github.com/GroepOnline/herdr/releases/download/v0.8.0/herdr-macos-aarch64"
      sha256 "2edc02575454c26b1093b7bb776ab4143e32733f851cd9d6c5b969b3783f954d"
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
