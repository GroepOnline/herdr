#!/usr/bin/env python3
"""Dev channel release helpers.

The dev channel mirrors the preview channel but publishes a fresh build for
every push to `main`, so maintainers (and the project owner) can dogfood the
bleeding edge with `herdr channel set dev` without waiting for a preview or
stable release. It reuses the preview manifest/notes machinery with a `dev`
channel label so the runtime updater treats it as its own prerelease stream.
"""

from __future__ import annotations

import argparse
from pathlib import Path

try:
    from scripts import preview
    from scripts.product_config import PRODUCT_GITHUB_REPO as DEFAULT_RELEASE_REPO
except ModuleNotFoundError:
    import preview  # type: ignore[no-redef]
    from product_config import PRODUCT_GITHUB_REPO as DEFAULT_RELEASE_REPO

DEFAULT_MANIFEST = "website/dev.json"
CHANNEL = "dev"
CHANNEL_LABEL = "Dev"
DEFAULT_BRANCH = "main"


def cmd_notes(args: argparse.Namespace) -> int:
    previous = (
        args.previous
        or preview.previous_preview_commit(Path(args.manifest))
        or preview.latest_stable_tag()
    )
    notes = preview.build_notes(
        previous,
        args.commit,
        args.build_id,
        args.base_version,
        args.repo,
        channel_label=CHANNEL_LABEL,
        branch=DEFAULT_BRANCH,
    )
    Path(args.output).write_text(notes, encoding="utf-8")
    return 0


def cmd_manifest(args: argparse.Namespace) -> int:
    notes = Path(args.notes).read_text(encoding="utf-8")
    shas = preview.read_sha_file(Path(args.sha_file) if args.sha_file else None)
    content = preview.build_manifest(
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
        channel=CHANNEL,
    )
    Path(args.output).write_text(content, encoding="utf-8")
    return 0


def cmd_current_commit(args: argparse.Namespace) -> int:
    commit = preview.previous_preview_commit(Path(args.manifest))
    if commit:
        print(commit)
    return 0


def cmd_select_commit(args: argparse.Namespace) -> int:
    print(preview.latest_publishable_commit(args.ref))
    return 0


def cmd_range_base(args: argparse.Namespace) -> int:
    print(preview.preview_range_base(args.previous, args.commit))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Dev channel release helpers")
    sub = parser.add_subparsers(required=True)

    notes = sub.add_parser("notes")
    notes.add_argument("--manifest", default=DEFAULT_MANIFEST)
    notes.add_argument("--previous")
    notes.add_argument("--commit", required=True)
    notes.add_argument("--build-id", required=True)
    notes.add_argument("--base-version", required=True)
    notes.add_argument("--repo", default=DEFAULT_RELEASE_REPO)
    notes.add_argument("--output", required=True)
    notes.set_defaults(func=cmd_notes)

    manifest = sub.add_parser("manifest")
    manifest.add_argument("--output", default=DEFAULT_MANIFEST)
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
    current.add_argument("--manifest", default=DEFAULT_MANIFEST)
    current.set_defaults(func=cmd_current_commit)

    select = sub.add_parser("select-commit")
    select.add_argument("--ref", default="origin/main")
    select.set_defaults(func=cmd_select_commit)

    range_base = sub.add_parser("range-base")
    range_base.add_argument("--previous", required=True)
    range_base.add_argument("--commit", required=True)
    range_base.set_defaults(func=cmd_range_base)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
