#!/usr/bin/env python3
"""One-shot patch that moves preview fully under OnlineChefGroep ownership."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")


def replace_once(path: str, old: str, new: str, label: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match in {path}, found {count}")
    write(path, text.replace(old, new, 1))


def patch_workflow() -> None:
    path = ".github/workflows/preview.yml"
    text = read(path)
    if "master" not in text:
        raise RuntimeError("preview workflow no longer contains the expected stale master references")
    write(path, text.replace("master", "main"))


def patch_preview_helpers() -> None:
    replace_once(
        "scripts/preview.py",
        'ASSET_TARGETS = ("linux-x86_64",)\n',
        'ASSET_TARGETS = ("linux-x86_64",)\nSHA256_RE = re.compile(r"^[a-f0-9]{64}$")\n',
        "add preview checksum grammar",
    )
    replace_once(
        "scripts/preview.py",
        '    branch: str = "master",\n',
        '    branch: str = "main",\n',
        "write main in preview notes",
    )
    replace_once(
        "scripts/preview.py",
        '''def read_sha_file(path: Path | None) -> dict[str, str]:
    if path is None:
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise SystemExit("sha file must be a JSON object")
    return {str(key): str(value) for key, value in data.items()}


def asset_objects(urls: dict[str, str], shas: dict[str, str]) -> dict[str, dict[str, str]]:
    assets: dict[str, dict[str, str]] = {}
    for target in ASSET_TARGETS:
        url = urls[target]
        entry = {"url": url}
        sha = shas.get(target)
        if sha:
            entry["sha256"] = sha
        assets[target] = entry
    return assets
''',
        '''def normalize_sha256(value: str, label: str) -> str:
    checksum = value.strip().lower()
    if SHA256_RE.fullmatch(checksum) is None:
        raise SystemExit(f"{label} must be 64 hexadecimal characters")
    return checksum


def read_sha_file(path: Path | None) -> dict[str, str]:
    if path is None:
        raise SystemExit("sha file is required for preview publication")
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise SystemExit("sha file must be a JSON object")
    return {
        str(key): normalize_sha256(str(value), f"checksum for {key}")
        for key, value in data.items()
    }


def asset_objects(urls: dict[str, str], shas: dict[str, str]) -> dict[str, dict[str, str]]:
    expected = set(ASSET_TARGETS)
    actual = set(shas)
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        details = []
        if missing:
            details.append(f"missing {', '.join(missing)}")
        if extra:
            details.append(f"unexpected {', '.join(extra)}")
        raise SystemExit(f"preview checksum target matrix mismatch: {'; '.join(details)}")

    return {
        target: {
            "url": urls[target],
            "sha256": normalize_sha256(shas[target], f"checksum for {target}"),
        }
        for target in ASSET_TARGETS
    }
''',
        "make preview checksums mandatory",
    )
    replace_once(
        "scripts/preview.py",
        '    select.add_argument("--ref", default="origin/master")\n',
        '    select.add_argument("--ref", default="origin/main")\n',
        "select preview commits from main",
    )


def patch_preview_tests() -> None:
    path = "scripts/test_preview.py"
    text = read(path).replace('"deadbeef"', '"d" * 64')
    marker = "    def test_build_manifest_accepts_dev_channel(self):\n"
    if text.count(marker) != 1:
        raise RuntimeError("preview test insertion marker changed")
    test = '''    def test_build_manifest_rejects_missing_or_invalid_checksums(self):
        urls = preview.default_asset_urls(
            PRODUCT_GITHUB_REPO,
            "preview-2026-06-02-abcdef123456",
        )
        with self.assertRaisesRegex(SystemExit, "missing linux-x86_64"):
            preview.asset_objects(urls, {})
        with self.assertRaisesRegex(SystemExit, "64 hexadecimal"):
            preview.asset_objects(urls, {"linux-x86_64": "deadbeef"})

    def test_preview_defaults_to_main(self):
        with mock.patch.object(preview, "commit_subjects", return_value=[]):
            notes = preview.build_notes(
                "previous",
                "abcdef1234567890",
                "2026-06-02-abcdef123456",
                "0.7.6",
                PRODUCT_GITHUB_REPO,
            )
        self.assertIn("on `main`", notes)

'''
    write(path, text.replace(marker, test + marker, 1))


def patch_quality_gate() -> None:
    replace_once(
        "scripts/ci_quality.py",
        '    Path("website/install.sh"),\n',
        '    Path("website/install.sh"),\n    Path("website/preview.json"),\n    Path("website/dev.json"),\n',
        "scan channel manifests for upstream ownership drift",
    )


def disable_upstream_preview() -> None:
    now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    manifest = {
        "schema_version": 1,
        "channel": "preview",
        "base_version": "0.7.6",
        "build_id": "disabled-upstream-preview",
        "commit": "",
        "built_at": now,
        "protocol": 18,
        "notes": (
            "Preview is temporarily disabled because the previous manifest referenced "
            "upstream ogulcancelik/herdr artifacts. Dispatch the Preview workflow from "
            "main to publish the first OnlineChefGroep-owned checksummed preview."
        ),
        "assets": {},
        "builds": {},
    }
    write("website/preview.json", json.dumps(manifest, indent=2) + "\n")


def patch_ownership_docs() -> None:
    codeowners = '''# Distribution and channel ownership
/.github/workflows/preview.yml @OnlineChef
/scripts/preview.py @OnlineChef
/website/preview.json @OnlineChef
/.github/workflows/release*.yml @OnlineChef
/.github/workflows/publish-distribution.yml @OnlineChef
/website/latest.json @OnlineChef
'''
    write(".github/CODEOWNERS", codeowners)

    marker = "## Sync policy\n"
    text = read("DOWNSTREAM.md")
    if text.count(marker) != 1:
        raise RuntimeError("DOWNSTREAM preview ownership insertion marker changed")
    section = '''## Preview ownership and rollback

- Owner: `@OnlineChef`, enforced for the preview workflow, helper, and manifest through `.github/CODEOWNERS`.
- Source branch: `main`; requested commits must be reachable from `origin/main`.
- Artifact namespace: preview prereleases in `OnlineChefGroep/herdr` only. The manifest is rejected by CI if it references `ogulcancelik/herdr`.
- Publication requires a complete checksum target matrix. Missing, extra, or malformed SHA-256 values abort before the manifest commit.
- Rollback: dispatch the Preview workflow with an earlier downstream commit reachable from `main`. If no safe downstream preview exists, keep top-level `assets` empty so clients fail closed; never restore upstream asset URLs.
- Website ownership follows the repository-backed `website/preview.json`; no separate `*.pages.dev` binding is a source of truth.

'''
    write("DOWNSTREAM.md", text.replace(marker, section + marker, 1))


def main() -> None:
    patch_workflow()
    patch_preview_helpers()
    patch_preview_tests()
    patch_quality_gate()
    disable_upstream_preview()
    patch_ownership_docs()
    print("preview ownership patch applied")


if __name__ == "__main__":
    main()
