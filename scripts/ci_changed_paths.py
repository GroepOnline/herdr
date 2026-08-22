#!/usr/bin/env python3
"""Emit CI lane outputs for selective heavy jobs.

Writes GitHub Actions outputs:
  rust, maintenance, release_meta, platform_heavy, docs_only
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _run(cmd: list[str]) -> str:
    return subprocess.check_output(cmd, text=True).strip()


def changed_files() -> list[str]:
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    if event == "pull_request":
        base = os.environ.get("GITHUB_BASE_REF", "main")
        try:
            _run(["git", "fetch", "origin", base, "--depth=1"])
            merge_base = _run(["git", "merge-base", "HEAD", f"origin/{base}"])
            out = _run(["git", "diff", "--name-only", f"{merge_base}...HEAD"])
        except subprocess.CalledProcessError:
            # Can't determine the diff (missing base ref, shallow clone, etc.);
            # fall back to the safe default of running the core lanes.
            return []
    else:
        # push / schedule / dispatch: compare against previous commit when available
        try:
            out = _run(["git", "diff", "--name-only", "HEAD~1...HEAD"])
        except subprocess.CalledProcessError:
            out = _run(["git", "ls-files"])
    return [line for line in out.splitlines() if line]


def matches(path: str, prefixes: tuple[str, ...], exact: tuple[str, ...] = ()) -> bool:
    if path in exact:
        return True
    return any(path == p or path.startswith(p.rstrip("/") + "/") or path.startswith(p) for p in prefixes)


DOCS_ONLY_PREFIXES = (
    "docs/",
    "website/",
    ".cursor/",
    ".codex/",
    ".claude/",
)
DOCS_ONLY_EXACT = (
    "README.md",
    "DOWNSTREAM.md",
    "AGENTS.md",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
    "LICENSE",
    ".github/design-system.json",
)
# Release/install metadata and CI tooling stay on their dedicated lanes.
DOCS_ONLY_EXCLUDE_EXACT = (
    "website/install.sh",
    "website/install.ps1",
    "website/latest.json",
)
DOCS_ONLY_EXCLUDE_PREFIXES = (
    "scripts/",
    ".github/workflows/",
)


def is_docs_only_path(path: str) -> bool:
    if path in DOCS_ONLY_EXCLUDE_EXACT:
        return False
    if matches(path, DOCS_ONLY_EXCLUDE_PREFIXES):
        return False
    return matches(path, DOCS_ONLY_PREFIXES, exact=DOCS_ONLY_EXACT)


def classify(files: list[str]) -> dict[str, bool]:
    if not files:
        return {
            "rust": True,
            "maintenance": True,
            "release_meta": True,
            "platform_heavy": True,
            "docs_only": False,
        }

    rust = any(
        matches(
            f,
            (
                "src/",
                "assets/",
                "vendor/",
                "tests/",
                "build.rs",
                "rust-toolchain.toml",
            ),
            exact=("Cargo.toml", "Cargo.lock"),
        )
        for f in files
    )
    maintenance = any(
        matches(
            f,
            (
                "scripts/",
                "src/integration/assets/",
                "workers/plugin-marketplace/",
                "docs/next/",
                "website/src/content/docs/",
                "plugins/",
                ".pi/",
            ),
            exact=("justfile", "AGENTS.md", "DOWNSTREAM.md", "website/install.sh"),
        )
        for f in files
    )
    release_meta = any(
        matches(
            f,
            ("npm/", "scripts/"),
            exact=(
                "Cargo.toml",
                "Cargo.lock",
                "CHANGELOG.md",
                "docs/next/CHANGELOG.md",
                "docs/next/product-announcement.json",
                "website/latest.json",
                "website/install.sh",
                "justfile",
            ),
        )
        for f in files
    )
    platform_heavy = any(
        matches(
            f,
            (
                "src/platform/",
                "src/pty/",
                "src/client/input/",
                "vendor/",
                "nix/",
                ".github/workflows/",
            ),
            exact=("Cargo.toml", "Cargo.lock", "build.rs", "flake.nix", "flake.lock"),
        )
        for f in files
    )
    docs_only = all(is_docs_only_path(f) for f in files)
    if docs_only:
        rust = False
        platform_heavy = False

    return {
        "rust": rust,
        "maintenance": maintenance,
        "release_meta": release_meta,
        "platform_heavy": platform_heavy,
        "docs_only": docs_only,
    }


def main() -> int:
    outputs = classify(changed_files())

    out_path = os.environ.get("GITHUB_OUTPUT")
    lines = [f"{key}={str(value).lower()}" for key, value in outputs.items()]
    text = "\n".join(lines) + "\n"
    if out_path:
        Path(out_path).write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
