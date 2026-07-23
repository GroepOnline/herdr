#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

CARGO_TOML_PATH = Path("Cargo.toml")
NPM_PACKAGE_PATH = Path("npm/package.json")
NPM_INSTALL_PATH = Path("npm/install.js")
CHANGELOG_PATH = Path("CHANGELOG.md")
GATEWAY_PATH = Path("src/bin/herdr-gateway.rs")
CHANGELOG_SCRIPT_PATH = Path("scripts/changelog.py")
NPM_README_PATH = Path("npm/README.md")

EXPECTED_RELEASE_REPO = 'DEFAULT_RELEASE_REPO = "OnlineChefGroep/herdr"'
GATEWAY_VERSION_SOURCE = 'env!("CARGO_PKG_VERSION")'

INSTALLER_VERSION_RE = re.compile(r'(?m)^const VERSION = "(?P<version>[^"]+)";$')
CHANGELOG_SECTION_RE = re.compile(
    r"^##\s+(?:\[(?P<bracketed>[^\]]+)\]|(?P<plain>.+?))"
    r"(?:\s+-\s+\d{4}-\d{2}-\d{2})?\s*$",
    re.MULTILINE,
)
RELEASE_SUBHEADING_RE = re.compile(r"^### (?P<title>Added|Fixed|Changed)\s*$", re.MULTILINE)
ANY_SUBHEADING_RE = re.compile(r"^### .+$", re.MULTILINE)


class QualityError(ValueError):
    pass


def repo_path(root: Path, path: Path) -> Path:
    return root / path


def read_text(root: Path, path: Path) -> str:
    try:
        return repo_path(root, path).read_text(encoding="utf-8")
    except FileNotFoundError as exc:
        raise QualityError(f"file not found: {path}") from exc


def write_text(root: Path, path: Path, content: str) -> None:
    repo_path(root, path).write_text(content, encoding="utf-8")


def load_json_object(root: Path, path: Path) -> dict[str, Any]:
    try:
        data = json.loads(read_text(root, path))
    except json.JSONDecodeError as exc:
        raise QualityError(f"invalid JSON in {path}: {exc}") from exc
    if not isinstance(data, dict):
        raise QualityError(f"expected JSON object in {path}")
    return data


def read_cargo_version(root: Path) -> str:
    try:
        data = tomllib.loads(read_text(root, CARGO_TOML_PATH))
    except tomllib.TOMLDecodeError as exc:
        raise QualityError(f"invalid TOML in {CARGO_TOML_PATH}: {exc}") from exc
    version = data.get("package", {}).get("version")
    if not isinstance(version, str) or not version:
        raise QualityError(f"{CARGO_TOML_PATH} is missing package.version")
    return version


def read_npm_package_version(root: Path) -> str:
    package = load_json_object(root, NPM_PACKAGE_PATH)
    version = package.get("version")
    if not isinstance(version, str) or not version:
        raise QualityError(f"{NPM_PACKAGE_PATH} is missing version")
    return version


def read_installer_version(root: Path) -> str:
    installer = read_text(root, NPM_INSTALL_PATH)
    match = INSTALLER_VERSION_RE.search(installer)
    if match is None:
        raise QualityError(f"{NPM_INSTALL_PATH} is missing const VERSION")
    return match.group("version")


def normalize_section_title(match: re.Match[str]) -> str:
    return (match.group("bracketed") or match.group("plain") or "").strip()


def extract_changelog_section(changelog: str, version: str) -> str:
    matches = list(CHANGELOG_SECTION_RE.finditer(changelog))
    for index, match in enumerate(matches):
        if normalize_section_title(match) != version:
            continue
        end = matches[index + 1].start() if index + 1 < len(matches) else len(changelog)
        return changelog[match.end() : end]
    raise QualityError(f"{CHANGELOG_PATH} is missing ## [{version}]")


def check_release_note_bullets(changelog: str, version: str) -> None:
    section = extract_changelog_section(changelog, version)
    found_release_heading = False

    for match in RELEASE_SUBHEADING_RE.finditer(section):
        found_release_heading = True
        next_heading = ANY_SUBHEADING_RE.search(section, match.end())
        body_end = next_heading.start() if next_heading else len(section)
        body = section[match.end() : body_end].lstrip("\n")
        if not body.startswith("-"):
            heading = match.group("title")
            raise QualityError(f"{CHANGELOG_PATH} {version} ### {heading} needs a bullet")

    if not found_release_heading:
        raise QualityError(
            f"{CHANGELOG_PATH} {version} needs an Added, Fixed, or Changed section with bullets"
        )


def check_release_metadata(root: Path) -> None:
    version = read_cargo_version(root)
    package_version = read_npm_package_version(root)
    installer_version = read_installer_version(root)
    changelog = read_text(root, CHANGELOG_PATH)
    gateway = read_text(root, GATEWAY_PATH)
    changelog_script = read_text(root, CHANGELOG_SCRIPT_PATH)
    npm_readme = read_text(root, NPM_README_PATH)

    package = load_json_object(root, NPM_PACKAGE_PATH)
    if package.get("os") != ["linux"]:
        raise QualityError(f'{NPM_PACKAGE_PATH} os must be ["linux"]')
    if package_version != version:
        raise QualityError(
            f"{NPM_PACKAGE_PATH} version {package_version} does not match Cargo.toml {version}"
        )
    if installer_version != version:
        raise QualityError(
            f"{NPM_INSTALL_PATH} VERSION {installer_version} does not match Cargo.toml {version}"
        )
    if f"## [{version}]" not in changelog:
        raise QualityError(f"{CHANGELOG_PATH} is missing ## [{version}]")
    if GATEWAY_VERSION_SOURCE not in gateway:
        raise QualityError(f"{GATEWAY_PATH} must use {GATEWAY_VERSION_SOURCE}")
    if EXPECTED_RELEASE_REPO not in changelog_script:
        raise QualityError(f"{CHANGELOG_SCRIPT_PATH} must use {EXPECTED_RELEASE_REPO}")
    if "RMEOF" in npm_readme:
        raise QualityError(f"{NPM_README_PATH} contains RMEOF")

    check_release_note_bullets(changelog, version)


def sync_release_metadata(root: Path) -> bool:
    version = read_cargo_version(root)
    changed = False

    package_path = repo_path(root, NPM_PACKAGE_PATH)
    package = load_json_object(root, NPM_PACKAGE_PATH)
    if package.get("version") != version:
        package["version"] = version
        package_path.write_text(json.dumps(package, indent=2) + "\n", encoding="utf-8")
        changed = True

    installer = read_text(root, NPM_INSTALL_PATH)
    updated_installer, count = INSTALLER_VERSION_RE.subn(
        f'const VERSION = "{version}";',
        installer,
        count=1,
    )
    if count != 1:
        raise QualityError(f"{NPM_INSTALL_PATH} is missing const VERSION")
    if updated_installer != installer:
        write_text(root, NPM_INSTALL_PATH, updated_installer)
        changed = True

    return changed


def needs_rustfmt(root: Path) -> bool:
    if shutil.which("cargo") is None:
        return False
    result = subprocess.run(
        ["cargo", "fmt", "--all", "--", "--check"],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return False
    combined = f"{result.stdout}\n{result.stderr}"
    return "Diff in" in combined


def detect_autofix(root: Path) -> dict[str, bool]:
    version = read_cargo_version(root)
    return {
        "needs_fmt": needs_rustfmt(root),
        "needs_metadata_sync": read_npm_package_version(root) != version
        or read_installer_version(root) != version,
    }


def cmd_check_release_metadata(args: argparse.Namespace) -> int:
    check_release_metadata(Path(args.root))
    print("release metadata: OK")
    return 0


def cmd_sync_release_metadata(args: argparse.Namespace) -> int:
    root = Path(args.root)
    version = read_cargo_version(root)
    changed = sync_release_metadata(root)
    if changed:
        print(f"synced release metadata to {version}")
    else:
        print(f"release metadata already synced to {version}")
    return 0


def cmd_detect_autofix(args: argparse.Namespace) -> int:
    print(json.dumps(detect_autofix(Path(args.root)), sort_keys=True))
    return 0


def add_root_argument(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--root", default=".", help="repository root")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Check and sync CI quality metadata")
    subparsers = parser.add_subparsers(dest="command", required=True)

    check = subparsers.add_parser(
        "check-release-metadata",
        help="Validate release metadata consistency",
    )
    add_root_argument(check)
    check.set_defaults(func=cmd_check_release_metadata)

    sync = subparsers.add_parser(
        "sync-release-metadata",
        help="Sync npm release metadata from Cargo.toml",
    )
    add_root_argument(sync)
    sync.set_defaults(func=cmd_sync_release_metadata)

    detect = subparsers.add_parser(
        "detect-autofix",
        help="Print mechanical autofix needs as JSON",
    )
    add_root_argument(detect)
    detect.set_defaults(func=cmd_detect_autofix)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except QualityError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
