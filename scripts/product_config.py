"""Canonical OnlineChefGroep Herdr distribution metadata.

All release tooling should import this module instead of duplicating repository,
package, target, asset, or install-method constants.
"""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Final

PRODUCT_GITHUB_ORG: Final = "OnlineChefGroep"
PRODUCT_GITHUB_REPO_NAME: Final = "herdr"
PRODUCT_GITHUB_REPO: Final = f"{PRODUCT_GITHUB_ORG}/{PRODUCT_GITHUB_REPO_NAME}"
UPSTREAM_GITHUB_REPO: Final = "ogulcancelik/herdr"
PRODUCT_SITE_URL: Final = "https://herdr.chefgroep.nl"
PRODUCT_CONTACT_EMAIL: Final = "hey@chefgroep.online"

NPM_PACKAGE_NAME: Final = "onlinechefgroep-herdr"
HOMEBREW_TAP_REPO: Final = "OnlineChefGroep/homebrew-tap"
HOMEBREW_FORMULA_NAME: Final = "onlinechefgroep-herdr"
HOMEBREW_INSTALL_HINT: Final = (
    "brew tap OnlineChefGroep/tap && "
    "brew install OnlineChefGroep/tap/onlinechefgroep-herdr"
)

DEFAULT_LIVE_MANIFEST_URL: Final = f"{PRODUCT_SITE_URL}/latest.json"
DEFAULT_PREVIEW_MANIFEST_URL: Final = f"{PRODUCT_SITE_URL}/preview.json"
DEFAULT_DEV_MANIFEST_URL: Final = f"{PRODUCT_SITE_URL}/dev.json"

# Stable public release matrix. Keep asset names compatible with existing
# releases and latest.json consumers.
RELEASE_TARGETS: Final = (
    {
        "platform": "linux",
        "arch": "x86_64",
        "npm_platform": "linux",
        "npm_arch": "x64",
        "rust_target": "x86_64-unknown-linux-gnu",
        "asset": "herdr-linux-x86_64",
    },
    {
        "platform": "linux",
        "arch": "aarch64",
        "npm_platform": "linux",
        "npm_arch": "arm64",
        "rust_target": "aarch64-unknown-linux-gnu",
        "asset": "herdr-linux-aarch64",
    },
    {
        "platform": "macos",
        "arch": "x86_64",
        "npm_platform": "darwin",
        "npm_arch": "x64",
        "rust_target": "x86_64-apple-darwin",
        "asset": "herdr-macos-x86_64",
    },
    {
        "platform": "macos",
        "arch": "aarch64",
        "npm_platform": "darwin",
        "npm_arch": "arm64",
        "rust_target": "aarch64-apple-darwin",
        "asset": "herdr-macos-aarch64",
    },
)


def cargo_version(root: Path = Path(".")) -> str:
    """Return package.version from Cargo.toml."""

    cargo_toml = root / "Cargo.toml"
    data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    version = data.get("package", {}).get("version")
    if not isinstance(version, str) or not version:
        raise ValueError(f"{cargo_toml} is missing package.version")
    return version


def release_tag(version: str) -> str:
    """Return the canonical GitHub release tag for a semantic version."""

    normalized = version.removeprefix("v")
    if not normalized:
        raise ValueError("release version must not be empty")
    return f"v{normalized}"


def release_asset_url(version: str, asset: str) -> str:
    """Return the canonical GitHub release download URL for one asset."""

    return (
        f"https://github.com/{PRODUCT_GITHUB_REPO}/releases/download/"
        f"{release_tag(version)}/{asset}"
    )


def target_by_asset(asset: str) -> dict[str, str]:
    """Return release target metadata for an asset name."""

    for target in RELEASE_TARGETS:
        if target["asset"] == asset:
            return dict(target)
    raise KeyError(f"unknown release asset: {asset}")


def target_by_npm(platform: str, arch: str) -> dict[str, str]:
    """Return release target metadata for Node's platform/arch pair."""

    for target in RELEASE_TARGETS:
        if target["npm_platform"] == platform and target["npm_arch"] == arch:
            return dict(target)
    raise KeyError(f"unsupported npm platform: {platform}-{arch}")


def release_assets(version: str) -> dict[str, str]:
    """Return platform/architecture keys mapped to canonical asset URLs."""

    return {
        f"{target['platform']}-{target['arch']}": release_asset_url(
            version, target["asset"]
        )
        for target in RELEASE_TARGETS
    }
