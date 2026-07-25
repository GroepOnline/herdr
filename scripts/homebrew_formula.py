#!/usr/bin/env python3
"""Generate or print the OnlineChefGroep Homebrew formula from a GitHub release.

The tap lives in OnlineChefGroep/homebrew-tap (not this repo). Maintainers can:

  python3 scripts/homebrew_formula.py --version 0.7.6 > /tmp/onlinechefgroep-herdr.rb

then copy into the tap Formula/ directory. Linux x86_64 is the only asset
published by the current release workflow; other platforms are omitted until
assets exist.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import ssl
import sys
import urllib.parse
import urllib.request
from pathlib import Path

# This script only fetches GitHub release metadata (API) and repo-hosted release
# assets (`browser_download_url`), so restrict outbound requests to HTTPS on the
# expected GitHub hosts to avoid SSRF via a crafted URL.
_ALLOWED_HOSTS = frozenset({"api.github.com", "github.com"})


def _validate_url(url: str) -> None:
    parts = urllib.parse.urlsplit(url)
    if parts.scheme != "https":
        raise ValueError(f"refusing non-HTTPS URL: {url}")
    if parts.hostname not in _ALLOWED_HOSTS:
        raise ValueError(f"refusing URL outside GitHub allowlist: {url}")

try:
    from scripts.product_config import PRODUCT_GITHUB_REPO, PRODUCT_SITE_URL
except ImportError:  # pragma: no cover - direct script execution
    from product_config import PRODUCT_GITHUB_REPO, PRODUCT_SITE_URL


def _get(url: str) -> bytes:
    _validate_url(url)
    req = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json"})
    ctx = ssl.create_default_context()
    with urllib.request.urlopen(req, context=ctx, timeout=30) as resp:
        return resp.read()


def sha256_url(url: str) -> str:
    data = _get(url)
    return hashlib.sha256(data).hexdigest()


def render(version: str, linux_x86_url: str, linux_x86_sha: str, repo: str) -> str:
    return f'''class OnlinechefgroepHerdr < Formula
  desc "Herdr fork with OnlineChefGroep agent manifests"
  homepage "https://github.com/{repo}"
  version "{version}"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
      url "{linux_x86_url}"
      sha256 "{linux_x86_sha}"
    end
  end

  def install
    bin.install "herdr-linux-x86_64" => "herdr"
  end

  test do
    assert_match version.to_s, shell_output("#{{bin}}/herdr --version")
  end
end
'''


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, help="Semver without v prefix")
    parser.add_argument(
        "--repo",
        default=PRODUCT_GITHUB_REPO,
        help="GitHub repo owner/name",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Optional path to write the formula",
    )
    args = parser.parse_args()

    tag = f"v{args.version}"
    api = f"https://api.github.com/repos/{args.repo}/releases/tags/{tag}"
    release = json.loads(_get(api).decode())
    assets = {a["name"]: a["browser_download_url"] for a in release.get("assets", [])}
    name = "herdr-linux-x86_64"
    if name not in assets:
        print(f"error: release {tag} missing {name}", file=sys.stderr)
        return 1
    url = assets[name]
    digest = sha256_url(url)
    body = render(args.version, url, digest, args.repo)
    if args.output:
        args.output.write_text(body, encoding="utf-8")
        print(f"wrote {args.output}", file=sys.stderr)
    else:
        sys.stdout.write(body)
    print(
        f"tip: after tap update, verify curl install via {PRODUCT_SITE_URL}/latest.json",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
