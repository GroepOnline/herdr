#!/usr/bin/env python3
"""Prepare the v0.7.7 release metadata on a clean checkout."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.7.7"
DATE = "2026-08-01"

NOTES = """### Added
- Added a checksum-backed four-platform distribution contract for Linux and macOS on x86_64 and ARM64, including npm and Homebrew publication.
- Added plugin catalog settings and install UX for managing Herdr plugins from the settings surface.

### Changed
- Stable release metadata is now promoted atomically only after all four native assets and `SHA256SUMS` have been downloaded and verified.
- Homebrew, npm, mise, and Nix installs are detected as package-managed and update through their owning package manager.
- Preview builds now publish from `main` under the OnlineChefGroep release namespace with mandatory checksums and explicit ownership.

### Fixed
- Direct, npm, self-update, and remote-bootstrap downloads now fail closed when SHA-256 metadata is missing, malformed, or mismatched.
- npm reinstalls now verify an existing native binary instead of trusting stale or corrupted package contents.
- The Windows lint lane now avoids restoring fragile Zig build caches and removes generated Zig outputs before clippy.
- Corrected release metadata, documentation, product URLs, and the four-target Homebrew formula generation path.
"""


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


def main() -> None:
    changelog = ROOT / "CHANGELOG.md"
    text = changelog.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "## Unreleased\n\n",
        f"## Unreleased\n\n## [{VERSION}] - {DATE}\n\n{NOTES}\n",
        "insert release notes",
    )
    changelog.write_text(text, encoding="utf-8")
    (ROOT / "docs/next/CHANGELOG.md").write_text(text, encoding="utf-8")

    cargo = ROOT / "Cargo.toml"
    cargo_text = cargo.read_text(encoding="utf-8")
    cargo_text = replace_once(
        cargo_text,
        'version = "0.7.6"',
        f'version = "{VERSION}"',
        "bump Cargo version",
    )
    cargo.write_text(cargo_text, encoding="utf-8")
    print(f"prepared release metadata for v{VERSION}")


if __name__ == "__main__":
    main()
