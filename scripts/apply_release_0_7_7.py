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


def patch_pending_release_manifest_validation() -> None:
    path = ROOT / "scripts/ci_quality.py"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        'SHA256_RE = re.compile(r"^[a-f0-9]{64}$")\n',
        'SHA256_RE = re.compile(r"^[a-f0-9]{64}$")\nSEMVER_RE = re.compile(r"^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)$")\n',
        "add semantic-version grammar",
    )
    text = replace_once(
        text,
        """def expected_manifest_assets(version: str) -> dict[str, str]:
""",
        """def parse_semver(version: str, label: str) -> tuple[int, int, int]:
    match = SEMVER_RE.fullmatch(version)
    if match is None:
        raise QualityError(f"{label} must be a stable X.Y.Z semantic version, got {version!r}")
    return tuple(int(part) for part in match.groups())


def expected_manifest_assets(version: str) -> dict[str, str]:
""",
        "add semantic-version parser",
    )
    old = """def check_latest_json_manifest(root: Path) -> None:
    version = read_cargo_version(root)
    manifest = load_json_object(root, LATEST_JSON_PATH)
    if manifest.get("version") != version:
        raise QualityError(
            f"{LATEST_JSON_PATH} version {manifest.get('version')!r} does not match Cargo.toml {version}"
        )

    validate_manifest_assets(manifest.get("assets"), version, "latest.json.assets")

    releases = manifest.get("releases")
    if not isinstance(releases, dict):
        raise QualityError(f"{LATEST_JSON_PATH} releases must be an object")
    release = releases.get(version)
    if not isinstance(release, dict):
        raise QualityError(f"{LATEST_JSON_PATH} releases is missing {version}")
    validate_manifest_assets(
        release.get("assets"), version, f"latest.json.releases.{version}.assets"
    )
"""
    new = """def check_latest_json_manifest(
    root: Path, *, expected_version: str | None = None
) -> str:
    manifest = load_json_object(root, LATEST_JSON_PATH)
    version = manifest.get("version")
    if not isinstance(version, str):
        raise QualityError(f"{LATEST_JSON_PATH} version must be a string")
    parse_semver(version, f"{LATEST_JSON_PATH} version")
    if expected_version is not None and version != expected_version:
        raise QualityError(
            f"{LATEST_JSON_PATH} version {version!r} does not match expected {expected_version}"
        )

    validate_manifest_assets(manifest.get("assets"), version, "latest.json.assets")

    releases = manifest.get("releases")
    if not isinstance(releases, dict):
        raise QualityError(f"{LATEST_JSON_PATH} releases must be an object")
    release = releases.get(version)
    if not isinstance(release, dict):
        raise QualityError(f"{LATEST_JSON_PATH} releases is missing {version}")
    validate_manifest_assets(
        release.get("assets"), version, f"latest.json.releases.{version}.assets"
    )
    return version
"""
    text = replace_once(text, old, new, "separate published manifest from source version")
    text = replace_once(
        text,
        """    check_release_note_bullets(changelog, version)
    check_latest_json_manifest(root)
    check_product_urls(root)
""",
        """    check_release_note_bullets(changelog, version)
    published_version = check_latest_json_manifest(root)
    if parse_semver(published_version, f"{LATEST_JSON_PATH} version") > parse_semver(
        version, f"{CARGO_TOML_PATH} version"
    ):
        raise QualityError(
            f"{LATEST_JSON_PATH} version {published_version} is newer than Cargo.toml {version}"
        )
    check_product_urls(root)
""",
        "permit a safely lagging published manifest",
    )
    text = replace_once(
        text,
        """def cmd_check_latest_json_manifest(args: argparse.Namespace) -> int:
    check_latest_json_manifest(Path(args.root))
""",
        """def cmd_check_latest_json_manifest(args: argparse.Namespace) -> int:
    root = Path(args.root)
    check_latest_json_manifest(root, expected_version=read_cargo_version(root))
""",
        "keep promotion gate strict",
    )
    path.write_text(text, encoding="utf-8")

    test_path = ROOT / "scripts/test_ci_quality.py"
    tests = test_path.read_text(encoding="utf-8")
    marker = """    def test_check_release_metadata_rejects_license_drift(self) -> None:
"""
    insertion = """    def test_check_release_metadata_accepts_pending_release_with_previous_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_fixture(root, "1.2.4", "1.2.4")
            manifest_path = root / "website/latest.json"
            previous = "1.2.3"
            assets = self.manifest_assets(previous)
            manifest_path.write_text(
                json.dumps(
                    {
                        "version": previous,
                        "protocol": 1,
                        "notes": "published fixture",
                        "assets": assets,
                        "releases": {
                            previous: {
                                "protocol": 1,
                                "notes": "published fixture",
                                "assets": assets,
                            }
                        },
                    },
                    indent=2,
                )
                + "\\n",
                encoding="utf-8",
            )

            check_release_metadata(root)
            self.assertEqual(check_latest_json_manifest(root), previous)
            with self.assertRaisesRegex(QualityError, "does not match expected 1.2.4"):
                check_latest_json_manifest(root, expected_version="1.2.4")

    def test_check_release_metadata_rejects_manifest_newer_than_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_fixture(root, "1.2.3", "1.2.3")
            manifest_path = root / "website/latest.json"
            future = "1.2.4"
            assets = self.manifest_assets(future)
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["version"] = future
            manifest["assets"] = assets
            manifest["releases"] = {
                future: {
                    "protocol": 1,
                    "notes": "future fixture",
                    "assets": assets,
                }
            }
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\\n", encoding="utf-8")

            with self.assertRaisesRegex(QualityError, "is newer than Cargo.toml"):
                check_release_metadata(root)

""" + marker
    tests = replace_once(tests, marker, insertion, "add pending release regression tests")
    test_path.write_text(tests, encoding="utf-8")


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
    patch_pending_release_manifest_validation()
    print(f"prepared release metadata for v{VERSION}")


if __name__ == "__main__":
    main()
