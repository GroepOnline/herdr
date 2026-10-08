#!/usr/bin/env python3
"""Generate the multi-platform GroepOnline Homebrew formula.

The generator consumes the release's SHA256SUMS file and verifies GitHub's
published asset digest when one is available. It never re-downloads the four
large binaries merely to calculate hashes.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import ssl
import sys
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

_ALLOWED_HOSTS = frozenset({"api.github.com", "github.com"})
# GitHub redirects asset downloads to its object storage hosts. Redirect
# targets are re-validated against this allowlist and never carry the token.
_ALLOWED_REDIRECT_HOSTS = _ALLOWED_HOSTS | {
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
}
_API_HOST = "api.github.com"
_SHA256_RE = re.compile(r"^[a-f0-9]{64}$")

try:
    from scripts.product_config import PRODUCT_GITHUB_REPO, RELEASE_TARGETS
except ImportError:  # pragma: no cover - direct script execution
    from product_config import PRODUCT_GITHUB_REPO, RELEASE_TARGETS


def _validate_url(url: str, allowed_hosts: frozenset[str] | set[str] = _ALLOWED_HOSTS) -> None:
    parts = urllib.parse.urlsplit(url)
    if parts.scheme != "https":
        raise ValueError(f"refusing non-HTTPS URL: {url}")
    if parts.hostname not in allowed_hosts:
        raise ValueError(f"refusing URL outside GitHub allowlist: {url}")


class _StrictRedirectHandler(urllib.request.HTTPRedirectHandler):
    """Re-validate every redirect target and never forward credentials."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):  # type: ignore[no-untyped-def]
        _validate_url(newurl, _ALLOWED_REDIRECT_HOSTS)
        redirected = super().redirect_request(req, fp, code, msg, headers, newurl)
        if redirected is not None:
            redirected.remove_header("Authorization")
        return redirected


def _get(url: str, *, authenticate: bool = False) -> bytes:
    _validate_url(url)
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "groeponline-herdr-homebrew-generator",
    }
    # The token is only ever sent to the API host: asset URLs redirect to
    # storage hosts that must never receive the credential.
    if authenticate and urllib.parse.urlsplit(url).hostname == _API_HOST:
        token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
        if token:
            headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, headers=headers)
    ctx = ssl.create_default_context()
    opener = urllib.request.build_opener(
        urllib.request.HTTPSHandler(context=ctx),
        _StrictRedirectHandler(),
    )
    with opener.open(req, timeout=30) as resp:
        return resp.read()


def normalize_checksum(value: str, label: str) -> str:
    checksum = value.strip().lower()
    if _SHA256_RE.fullmatch(checksum) is None:
        raise ValueError(f"{label} must be 64 hexadecimal characters")
    return checksum


def parse_sha256sums(text: str, label: str = "SHA256SUMS") -> dict[str, str]:
    checksums: dict[str, str] = {}
    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        line = raw_line.strip()
        if not line:
            continue
        match = re.fullmatch(r"([a-fA-F0-9]{64})\s+\*?(.+)", line)
        if match is None:
            raise ValueError(f"{label}:{line_number} is not a valid SHA256SUMS line")
        name = match.group(2).strip()
        if not name or "/" in name or "\\" in name:
            raise ValueError(f"{label}:{line_number} contains an invalid asset name")
        if name in checksums:
            raise ValueError(f"{label} contains duplicate checksum for {name}")
        checksums[name] = match.group(1).lower()
    if not checksums:
        raise ValueError(f"{label} is empty")
    return checksums


def asset_digest(asset: dict[str, Any]) -> str | None:
    raw = asset.get("digest")
    if not isinstance(raw, str) or not raw.strip():
        return None
    algorithm, separator, checksum = raw.strip().partition(":")
    if separator != ":" or algorithm.lower() != "sha256":
        raise ValueError(f"unsupported GitHub digest for {asset.get('name')}: {raw}")
    return normalize_checksum(checksum, f"GitHub digest for {asset.get('name')}")


def release_asset_map(release: dict[str, Any]) -> dict[str, dict[str, Any]]:
    raw_assets = release.get("assets")
    if not isinstance(raw_assets, list):
        raise ValueError("GitHub release response is missing assets")
    assets: dict[str, dict[str, Any]] = {}
    for raw_asset in raw_assets:
        if not isinstance(raw_asset, dict):
            continue
        name = raw_asset.get("name")
        if isinstance(name, str) and name and name not in assets:
            assets[name] = raw_asset
    return assets


def verified_release_assets(
    release: dict[str, Any], checksums: dict[str, str]
) -> dict[str, tuple[str, str]]:
    assets = release_asset_map(release)
    verified: dict[str, tuple[str, str]] = {}
    for target in RELEASE_TARGETS:
        name = target["asset"]
        asset = assets.get(name)
        if asset is None:
            raise ValueError(f"release is missing {name}")
        url = asset.get("browser_download_url") or asset.get("url")
        if not isinstance(url, str) or not url.strip():
            raise ValueError(f"release asset {name} is missing a download URL")
        _validate_url(url)
        checksum = checksums.get(name)
        if checksum is None:
            raise ValueError(f"SHA256SUMS is missing {name}")
        checksum = normalize_checksum(checksum, f"checksum for {name}")
        digest = asset_digest(asset)
        if digest is not None and digest != checksum:
            raise ValueError(f"GitHub digest for {name} does not match SHA256SUMS")
        verified[name] = (url, checksum)
    return verified


def render(
    version: str,
    assets: dict[str, tuple[str, str]],
    repo: str = PRODUCT_GITHUB_REPO,
) -> str:
    by_name = {target["asset"]: target for target in RELEASE_TARGETS}
    required = set(by_name)
    if set(assets) != required:
        missing = sorted(required - set(assets))
        extra = sorted(set(assets) - required)
        details = []
        if missing:
            details.append(f"missing {', '.join(missing)}")
        if extra:
            details.append(f"unexpected {', '.join(extra)}")
        raise ValueError(f"formula asset matrix mismatch: {'; '.join(details)}")

    def stanza(name: str, indent: str = "      ") -> str:
        url, checksum = assets[name]
        return f'{indent}url "{url}"\n{indent}sha256 "{checksum}"'

    return f'''class GroeponlineHerdr < Formula
  desc "Terminal workspace manager for AI coding agents"
  homepage "https://github.com/{repo}"
  version "{version}"
  license "AGPL-3.0-or-later"

  livecheck do
    url :homepage
    strategy :github_latest
  end

  on_linux do
    on_intel do
{stanza("herdr-linux-x86_64")}
    end
    on_arm do
{stanza("herdr-linux-aarch64")}
    end
  end

  on_macos do
    on_intel do
{stanza("herdr-macos-x86_64")}
    end
    on_arm do
{stanza("herdr-macos-aarch64")}
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
    assert_match version.to_s, shell_output("#{{bin}}/herdr --version")
  end
end
'''


def load_checksums(
    path: Path | None,
    release_assets: dict[str, dict[str, Any]],
) -> dict[str, str]:
    if path is not None:
        return parse_sha256sums(path.read_text(encoding="utf-8"), str(path))
    sums_asset = release_assets.get("SHA256SUMS")
    if sums_asset is None:
        raise ValueError("release is missing SHA256SUMS")
    url = sums_asset.get("browser_download_url") or sums_asset.get("url")
    if not isinstance(url, str) or not url.strip():
        raise ValueError("SHA256SUMS asset is missing a download URL")
    return parse_sha256sums(_get(url).decode("utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, help="Semver without v prefix")
    parser.add_argument("--repo", default=PRODUCT_GITHUB_REPO, help="GitHub owner/repo")
    parser.add_argument(
        "--checksums",
        type=Path,
        help="Verified SHA256SUMS path; fetched from the release when omitted",
    )
    parser.add_argument("--output", type=Path, help="Optional formula output path")
    args = parser.parse_args()

    try:
        tag = f"v{args.version.removeprefix('v')}"
        api = f"https://api.github.com/repos/{args.repo}/releases/tags/{tag}"
        release = json.loads(_get(api, authenticate=True).decode("utf-8"))
        if not isinstance(release, dict):
            raise ValueError("unexpected GitHub release response")
        release_assets = release_asset_map(release)
        checksums = load_checksums(args.checksums, release_assets)
        verified = verified_release_assets(release, checksums)
        body = render(args.version.removeprefix("v"), verified, args.repo)
    except (OSError, UnicodeError, ValueError, json.JSONDecodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(body, encoding="utf-8")
        print(f"wrote {args.output}", file=sys.stderr)
    else:
        sys.stdout.write(body)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
