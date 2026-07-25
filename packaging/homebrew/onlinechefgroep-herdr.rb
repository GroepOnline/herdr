class OnlinechefgroepHerdr < Formula
  desc "Herdr fork with OnlineChefGroep agent manifests"
  homepage "https://github.com/OnlineChefGroep/herdr"
  version "0.7.6"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
      url "https://github.com/OnlineChefGroep/herdr/releases/download/v0.7.6/herdr-linux-x86_64"
      sha256 "8f0785c5e9e471e03e7611d6b987b60bf1f9a7db0f25bec95c11f54e156a561a"
    end
  end

  def install
    bin.install "herdr-linux-x86_64" => "herdr"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/herdr --version")
  end
end
