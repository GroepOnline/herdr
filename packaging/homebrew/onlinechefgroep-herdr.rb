class OnlinechefgroepHerdr < Formula
  desc "Herdr fork with OnlineChefGroep agent manifests"
  homepage "https://github.com/OnlineChefGroep/herdr"
  version "0.7.3"
  license "MIT"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_macos do
    on_arm do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-macos-aarch64"
      sha256 "b31345392d004ec1f1b2c821e1ad601019fa8385fe1e4c6931321eb58a920773"
    end
    on_intel do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-macos-x86_64"
      sha256 "9b5f35d283b0877eeda0cf66ba1ef1d95ae40f32e858a04da0041f3a20df027c"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-linux-aarch64"
      sha256 "ea490094f2c7c39099870857d00c64c628ef7b5eba1967df4258033455ee2cb1"
    end
    on_intel do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v#{version}/herdr-linux-x86_64"
      sha256 "043ef43ecbabda28465dcff1eec3184518150d567b8b8f20cda9c6c88770641d"
    end
  end

  def install
    if OS.mac?
      binary = Hardware::CPU.arm? ? "herdr-macos-aarch64" : "herdr-macos-x86_64"
    else
      binary = Hardware::CPU.arm? ? "herdr-linux-aarch64" : "herdr-linux-x86_64"
    end
    bin.install binary => "herdr"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/herdr --version")
  end
end
