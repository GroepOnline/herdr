class OnlinechefgroepHerdr < Formula
  desc "Herdr fork with OnlineChefGroep agent manifests"
  homepage "https://github.com/OnlineChefGroep/herdr"
  version "0.7.4"
  license "MIT"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_macos do
    on_arm do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-macos-aarch64"
      sha256 "138b4aeee8b677dd53d7bd46ce87e7fad8825e74d3a32c45420aa908f9d8b3c6"
    end
    on_intel do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-macos-x86_64"
      sha256 "1b823d563dae78f5c05f0d9f66783f6e441ba588f19f682f70b5c39b5f721d73"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-linux-aarch64"
      sha256 "768281c784c13a3b4da971959cedee972d84637e84e5d05c71e97846bfd4731e"
    end
    on_intel do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-linux-x86_64"
      sha256 "00a27f593e5af92ab0b1f54c76de2192dd5e83207623801bc6bd20cfee465ae7"
    end
  end

  def install
    bin.install Dir["herdr-*"].first => "herdr"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/herdr --version")
  end
end
