#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path
from typing import Any

try:
    from scripts.product_config import PRODUCT_GITHUB_REPO as DEFAULT_RELEASE_REPO
except ModuleNotFoundError:
    from product_config import PRODUCT_GITHUB_REPO as DEFAULT_RELEASE_REPO

ASSET_TARGETS = ("linux-x86_64",)
SHA256_RE = re.compile(r"^[a-f0-9]{64}$")
EXPECTED_ASSET_NAMES = {target: f"herdr-{target}" for target in ASSET_TARGETS}
HIDDEN_SUBJECTS = (
    "docs: update website manifest",
    "docs: update preview manifest",
    "docs: update dev manifest",
    "chore: approve contributor",
    "chore: approve merged contributor",
)
TYPE_HEADINGS = {
    "feat": "Added",
    "fix": "Fixed",
    "perf": "Performance",
    "docs": "Maintenance",
    "ci": "Maintenance",
    "test": "Maintenance",
    "refactor": "Maintenance",
    "chore": "Maintenance",
}
TYPE_ORDER = ("Added", "Fixed", "Performance", "Maintenance", "Other")
COMMIT_RE = re.compile(r"^(?P<kind>[a-z]+)(?:\([^)]+\))?!?:\s+(?P<body>.+)$")


def run_git(args: list[str]) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def normalize_version(version: str) -> str:
    return version.strip().removeprefix("v")


def latest_stable_tag(ref: str | None = None) -> str:
    args = ["describe", "--tags", "--match", "v[0-9]*", "--abbrev=0"]
    if ref:
        args.append(ref)
    return run_git(args)


def git_is_ancestor(ancestor: str, descendant: str) -> bool:
    result = subprocess.run(
        ["git", "merge-base", "--is-ancestor", ancestor, descendant],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode == 0


def read_json(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def previous_preview_commit(path: Path) -> str | None:
    data = read_json(path)
    if not data:
        return None
    commit = data.get("commit")
    return commit if isinstance(commit, str) and commit.strip() else None


def manifest_enabled(path: Path) -> bool:
    data = read_json(path)
    if data is None:
        return True
    return bool(data.get("enabled", True))


def hidden_subject(subject: str) -> bool:
    lowered = subject.strip().lower()
    return any(lowered.startswith(prefix) for prefix in HIDDEN_SUBJECTS)


def latest_publishable_commit(ref: str) -> str:
    output = run_git(["log", "--pretty=format:%H%x00%s", ref])
    for line in output.splitlines():
        commit, _, subject = line.partition("\x00")
        if commit and not hidden_subject(subject):
            return commit
    raise SystemExit(f"no publishable commit found in {ref}")


def commit_subjects(previous: str, commit: str) -> list[str]:
    output = run_git(["log", "--pretty=format:%s", f"{previous}..{commit}"])
    if not output:
        return []
    subjects = []
    for line in output.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if hidden_subject(stripped):
            continue
        subjects.append(stripped)
    return subjects


def preview_range_base(previous: str, commit: str) -> str:
    try:
        stable = latest_stable_tag(commit)
    except subprocess.CalledProcessError:
        return previous
    if not git_is_ancestor(previous, commit):
        # previous is not reachable from the target commit (for example after
        # an org migration rewrote history), so a notes range previous..commit
        # would fail git log with exit 128. Base the notes on the latest stable
        # tag instead, which is always reachable from commit.
        return stable
    if git_is_ancestor(previous, stable) and git_is_ancestor(stable, commit):
        return stable
    return previous


def humanize_subject(subject: str) -> tuple[str, str]:
    match = COMMIT_RE.match(subject)
    if not match:
        return "Other", subject[0].upper() + subject[1:]
    kind = match.group("kind")
    body = match.group("body").strip()
    heading = TYPE_HEADINGS.get(kind, "Other")
    if body:
        body = body[0].upper() + body[1:]
    else:
        body = subject
    return heading, body


def build_notes(
    previous: str,
    commit: str,
    build_id: str,
    base_version: str,
    repo: str,
    channel_label: str = "Preview",
    branch: str = "main",
) -> str:
    short = commit[:12]
    compare = f"https://github.com/{repo}/compare/{previous}...{commit}"
    lines = [
        f"{channel_label} build {build_id}",
        "",
        f"Built from `{short}` on `{branch}`.",
        f"Base stable: v{normalize_version(base_version)}",
        f"Compare: {compare}",
        "",
    ]
    grouped: dict[str, list[str]] = {heading: [] for heading in TYPE_ORDER}
    for subject in commit_subjects(previous, commit):
        heading, body = humanize_subject(subject)
        grouped.setdefault(heading, []).append(body)

    wrote = False
    for heading in TYPE_ORDER:
        items = grouped.get(heading, [])
        if not items:
            continue
        wrote = True
        lines.append(f"### {heading}")
        for item in items:
            lines.append(f"- {item}")
        lines.append("")

    if not wrote:
        lines.extend(
            [
                "### Changed",
                f"- Rebuilt {channel_label.lower()} from the current {branch} branch.",
                "",
            ]
        )

    return "\n".join(lines).rstrip() + "\n"


def default_asset_urls(repo: str, tag: str) -> dict[str, str]:
    return {
        target: f"https://github.com/{repo}/releases/download/{tag}/{EXPECTED_ASSET_NAMES[target]}"
        for target in ASSET_TARGETS
    }


def normalize_sha256(value: str, label: str) -> str:
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


def build_manifest(
    output: Path,
    repo: str,
    tag: str,
    build_id: str,
    commit: str,
    built_at: str,
    base_version: str,
    protocol: int,
    notes: str,
    shas: dict[str, str],
    retain: int,
    channel: str = "preview",
) -> str:
    urls = default_asset_urls(repo, tag)
    assets = asset_objects(urls, shas)
    current = read_json(output) or {}
    builds = current.get("builds") if isinstance(current.get("builds"), dict) else {}
    builds = dict(builds)
    builds[build_id] = {
        "base_version": normalize_version(base_version),
        "commit": commit,
        "built_at": built_at,
        "protocol": protocol,
        "tag": tag,
        "assets": assets,
    }
    ordered_builds = {
        key: builds[key]
        for key in sorted(
            builds,
            key=lambda key: str(builds[key].get("built_at", "")),
            reverse=True,
        )[:retain]
    }
    manifest = {
        "schema_version": 1,
        "channel": channel,
        "enabled": True,
        "base_version": normalize_version(base_version),
        "build_id": build_id,
        "commit": commit,
        "built_at": built_at,
        "protocol": protocol,
        "notes": notes.strip(),
        "assets": assets,
        "builds": ordered_builds,
    }
    return json.dumps(manifest, indent=2) + "\n"


def cmd_notes(args: argparse.Namespace) -> int:
    previous = args.previous or previous_preview_commit(Path(args.manifest)) or latest_stable_tag()
    notes = build_notes(previous, args.commit, args.build_id, args.base_version, args.repo)
    Path(args.output).write_text(notes, encoding="utf-8")
    return 0


def cmd_manifest(args: argparse.Namespace) -> int:
    notes = Path(args.notes).read_text(encoding="utf-8")
    shas = read_sha_file(Path(args.sha_file) if args.sha_file else None)
    content = build_manifest(
        output=Path(args.output),
        repo=args.repo,
        tag=args.tag,
        build_id=args.build_id,
        commit=args.commit,
        built_at=args.built_at,
        base_version=args.base_version,
        protocol=args.protocol,
        notes=notes,
        shas=shas,
        retain=args.retain,
    )
    Path(args.output).write_text(content, encoding="utf-8")
    return 0


def cmd_current_commit(args: argparse.Namespace) -> int:
    commit = previous_preview_commit(Path(args.manifest))
    if commit:
        print(commit)
    return 0


def cmd_select_commit(args: argparse.Namespace) -> int:
    if not manifest_enabled(Path(args.manifest)):
        raise SystemExit(
            "preview channel is disabled for the downstream distribution; "
            "publish only via explicit Preview workflow dispatch with a commit input"
        )
    print(latest_publishable_commit(args.ref))
    return 0


def cmd_range_base(args: argparse.Namespace) -> int:
    print(preview_range_base(args.previous, args.commit))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Preview channel release helpers")
    sub = parser.add_subparsers(required=True)

    notes = sub.add_parser("notes")
    notes.add_argument("--manifest", default="website/preview.json")
    notes.add_argument("--previous")
    notes.add_argument("--commit", required=True)
    notes.add_argument("--build-id", required=True)
    notes.add_argument("--base-version", required=True)
    notes.add_argument("--repo", default=DEFAULT_RELEASE_REPO)
    notes.add_argument("--output", required=True)
    notes.set_defaults(func=cmd_notes)

    manifest = sub.add_parser("manifest")
    manifest.add_argument("--output", default="website/preview.json")
    manifest.add_argument("--repo", default=DEFAULT_RELEASE_REPO)
    manifest.add_argument("--tag", required=True)
    manifest.add_argument("--build-id", required=True)
    manifest.add_argument("--commit", required=True)
    manifest.add_argument("--built-at", required=True)
    manifest.add_argument("--base-version", required=True)
    manifest.add_argument("--protocol", required=True, type=int)
    manifest.add_argument("--notes", required=True)
    manifest.add_argument("--sha-file")
    manifest.add_argument("--retain", type=int, default=30)
    manifest.set_defaults(func=cmd_manifest)

    current = sub.add_parser("current-commit")
    current.add_argument("--manifest", default="website/preview.json")
    current.set_defaults(func=cmd_current_commit)

    select = sub.add_parser("select-commit")
    select.add_argument("--ref", default="origin/main")
    select.add_argument("--manifest", default="website/preview.json")
    select.set_defaults(func=cmd_select_commit)

    range_base = sub.add_parser("range-base")
    range_base.add_argument("--previous", required=True)
    range_base.add_argument("--commit", required=True)
    range_base.set_defaults(func=cmd_range_base)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
