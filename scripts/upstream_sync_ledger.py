#!/usr/bin/env python3
"""Upstream sync ledger for GroepOnline/herdr.

Modes
-----
``--check``     Offline validation of ``sync/upstream.json``,
                ``sync/overlay.tsv`` and ``sync/ledger.json``. Runs in
                ``just maintenance`` and never touches the network.
``--generate``  Refreshes ``sync/ledger.json`` from a blobless upstream clone.
``--plan``      Shows overlay coverage per row plus the downstream-only paths
                that no overlay glob covers yet.
``--summary``   Prints the stored snapshot without re-reading upstream.

Process context: ``.github/upstream-sync.md``.
"""

from __future__ import annotations

import argparse
import datetime as dt
import fnmatch
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SYNC_DIR = REPO_ROOT / "sync"
PIN_PATH = SYNC_DIR / "upstream.json"
OVERLAY_PATH = SYNC_DIR / "overlay.tsv"
LEDGER_PATH = SYNC_DIR / "ledger.json"

SCHEMA_VERSION = 1
UPSTREAM_REPO = "herdrdev/herdr"
UPSTREAM_URL = f"https://github.com/{UPSTREAM_REPO}"
DEFAULT_CACHE = REPO_ROOT / ".local" / "upstream.git"
DEFAULT_REFERENCE = "master"
DEFAULT_WINDOW_SINCE = "2026-04-01"

OVERLAY_COLUMNS = (
    "path_glob",
    "concern",
    "owner",
    "rationale",
    "tests",
    "removal_condition",
)

CRITICAL_PREFIXES = (
    "src/integration/",
    "src/detect/",
    "src/protocol/",
    "vendor/",
    "crates/",
    "distribution/agent-detection/",
)

PIN_REQUIRED = ("repo", "base_tag", "base_commit", "reference", "window_since", "generated_at")
SHA_RE = re.compile(r"^[0-9a-f]{7,40}$")
PR_SUFFIX_RE = re.compile(r"\s*\(#\d+\)\s*$")


# --------------------------------------------------------------------------- #
# pure helpers (unit tested, no I/O)
# --------------------------------------------------------------------------- #


def normalize_subject(subject: str) -> str:
    """Fingerprint a commit subject: drop a trailing ``(#123)`` and lowercase."""
    return PR_SUFFIX_RE.sub("", subject).strip().lower()


def _glob_to_regex(pattern: str) -> re.Pattern[str]:
    """Translate a path glob where ``*`` never crosses ``/`` (gitignore-like)."""
    out: list[str] = []
    index = 0
    while index < len(pattern):
        char = pattern[index]
        if pattern.startswith("**/", index):
            out.append("(?:.*/)?")
            index += 3
            continue
        if pattern.startswith("**", index):
            out.append(".*")
            index += 2
            continue
        if char == "*":
            out.append("[^/]*")
        elif char == "?":
            out.append("[^/]")
        else:
            out.append(re.escape(char))
        index += 1
    return re.compile("^" + "".join(out) + "$")


def glob_matches(pattern: str, path: str) -> bool:
    """Match ``pattern`` against a repo-relative path.

    ``dir/**`` matches the subtree, ``**/*.ext`` matches at any depth and ``*``
    stays inside one path segment.
    """
    if pattern.endswith("/**"):
        prefix = pattern[: -len("/**")]
        return path == prefix or path.startswith(prefix + "/")
    if pattern in ("**", "*"):
        return True
    if _glob_to_regex(pattern).match(path):
        return True
    return _glob_to_regex(pattern.rstrip("/") + "/**").match(path)


@dataclass(frozen=True)
class OverlayRow:
    path_glob: str
    concern: str
    owner: str
    rationale: str
    tests: str
    removal_condition: str


def parse_overlay(text: str) -> tuple[list[OverlayRow], list[str]]:
    """Parse ``overlay.tsv``; return (rows, errors)."""
    rows: list[OverlayRow] = []
    errors: list[str] = []
    seen: set[str] = set()
    lines = [line for line in text.splitlines() if line.strip() and not line.lstrip().startswith("#")]
    if not lines:
        return rows, ["overlay.tsv has no data rows"]
    header = lines[0].split("\t")
    if tuple(header) != OVERLAY_COLUMNS:
        return rows, [f"overlay.tsv header must be {OVERLAY_COLUMNS}, found {tuple(header)}"]
    for number, line in enumerate(lines[1:], start=2):
        cells = line.split("\t")
        if len(cells) != len(OVERLAY_COLUMNS):
            errors.append(f"overlay.tsv line {number}: expected {len(OVERLAY_COLUMNS)} columns, found {len(cells)}")
            continue
        row = OverlayRow(*[cell.strip() for cell in cells])
        if not row.path_glob:
            errors.append(f"overlay.tsv line {number}: empty path_glob")
            continue
        if row.path_glob in seen:
            errors.append(f"overlay.tsv line {number}: duplicate path_glob {row.path_glob}")
            continue
        seen.add(row.path_glob)
        for column in ("concern", "owner", "rationale", "tests", "removal_condition"):
            if not getattr(row, column):
                errors.append(f"overlay.tsv line {number}: missing {column} for {row.path_glob}")
        rows.append(row)
    return rows, errors


def validate_pin(pin: dict) -> list[str]:
    errors: list[str] = []
    for key in PIN_REQUIRED:
        if not pin.get(key):
            errors.append(f"upstream.json: missing {key}")
    commit = pin.get("base_commit", "")
    if commit and not SHA_RE.match(str(commit)):
        errors.append(f"upstream.json: base_commit is not a sha: {commit!r}")
    if pin.get("repo") and pin["repo"] != UPSTREAM_REPO:
        errors.append(f"upstream.json: repo must be {UPSTREAM_REPO}, found {pin['repo']!r}")
    return errors


def validate_overlay(rows: list[OverlayRow], files: list[str], recorded: list[str] = ()) -> list[str]:
    """Check that every overlay glob still describes something real.

    A glob may match the current tree or the recorded downstream snapshot, so
    the inventory stays meaningful while a sync branch is still building up.
    """
    errors: list[str] = []
    candidates = list(files) + list(recorded)
    for row in rows:
        if not any(glob_matches(row.path_glob, path) for path in candidates):
            errors.append(
                f"overlay.tsv: path_glob {row.path_glob} matches no tracked file "
                "and no recorded downstream path"
            )
    return errors


def fingerprint_backlog(upstream: list[tuple[str, str, str]], downstream: list[tuple[str, str, str]]) -> list[tuple[str, str, str]]:
    """Return upstream commits that no downstream commit fingerprints.

    A commit is only counted when its normalized subject is unique upstream, so
    repeated bot subjects (``docs: update preview manifest``) never inflate the
    backlog. Entries are ``(sha, date, subject)``.
    """
    counts: dict[str, int] = {}
    for _, _, subject in upstream:
        counts[normalize_subject(subject)] = counts.get(normalize_subject(subject), 0) + 1
    have = {normalize_subject(subject) for _, _, subject in downstream}
    return [
        (sha, date, subject)
        for sha, date, subject in upstream
        if counts[normalize_subject(subject)] == 1 and normalize_subject(subject) not in have
    ]


def classify_paths(upstream: dict[str, str], downstream: dict[str, str]) -> dict:
    common = sorted(set(upstream) & set(downstream))
    differing = [path for path in common if upstream[path] != downstream[path]]
    upstream_only = sorted(set(upstream) - set(downstream))
    downstream_only = sorted(set(downstream) - set(upstream))
    relocated: dict[str, str] = {}
    upstream_by_blob: dict[str, str] = {}
    for path, blob in upstream.items():
        upstream_by_blob.setdefault(blob, path)
    for path in downstream_only:
        source = upstream_by_blob.get(downstream[path])
        if source:
            relocated[path] = source
    return {
        "paths_common": len(common),
        "paths_identical": len(common) - len(differing),
        "paths_differing": differing,
        "paths_only_upstream": upstream_only,
        "paths_only_downstream": downstream_only,
        "relocated_only_downstream": relocated,
        "critical_paths_changed": sorted(
            path for path in differing if path.startswith(CRITICAL_PREFIXES)
        )
        + sorted(path for path in upstream_only if path.startswith(CRITICAL_PREFIXES)),
    }


def uncovered_overlay_paths(downstream_only: list[str], rows: list[OverlayRow]) -> list[str]:
    return [path for path in downstream_only if not any(glob_matches(row.path_glob, path) for row in rows)]


def overlay_scope(head_ref: str, base_commit: str, cwd: Path = REPO_ROOT) -> list[str]:
    """Paths our overlay changed relative to the pinned upstream base.

    Only meaningful on a sync branch whose trunk is the pinned base: it is the
    exact footprint of the downstream overlay.
    """
    out = run_git(["diff", "--name-only", base_commit, head_ref], cwd)
    return [line for line in out.splitlines() if line]


def validate_scope(scope: list[str], rows: list[OverlayRow]) -> list[str]:
    """Every path the overlay touches must be declared by an overlay glob."""
    errors: list[str] = []
    undeclared = [
        path for path in scope if not any(glob_matches(row.path_glob, path) for row in rows)
    ]
    if undeclared:
        errors.append(
            f"overlay scope: {len(undeclared)} path(s) changed relative to the pinned base are not "
            f"declared in sync/overlay.tsv (first: {undeclared[0]})"
        )
    return errors


def validate_ledger(ledger: dict, pin: dict, rows: list[OverlayRow]) -> list[str]:
    errors: list[str] = []
    if ledger.get("schema_version") != SCHEMA_VERSION:
        errors.append(f"ledger.json: schema_version must be {SCHEMA_VERSION}")
    upstream = ledger.get("upstream") or {}
    if upstream.get("base_commit") != pin.get("base_commit"):
        errors.append(
            "ledger.json: base_commit does not match upstream.json "
            f"({upstream.get('base_commit')} != {pin.get('base_commit')})"
        )
    counts = ledger.get("counts") or {}
    backlog = ledger.get("unported_commits") or []
    if counts.get("unported") != len(backlog):
        errors.append("ledger.json: counts.unported does not match the unported commit list")
    downstream_only = ledger.get("paths_only_downstream_list") or []
    if counts.get("paths_only_downstream") != len(downstream_only):
        errors.append(
            "ledger.json: counts.paths_only_downstream does not match the stored path list"
        )
    errors.extend(validate_scope(ledger.get("overlay_scope") or [], rows))
    uncovered = uncovered_overlay_paths(downstream_only, rows)
    if uncovered:
        errors.append(
            f"ledger.json: {len(uncovered)} downstream-only path(s) are not covered by sync/overlay.tsv "
            f"(first: {uncovered[0]})"
        )
    return errors


# --------------------------------------------------------------------------- #
# git / io
# --------------------------------------------------------------------------- #


def run_git(args: list[str], cwd: Path) -> str:
    result = subprocess.run(
        ["git", *args], cwd=str(cwd), capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout


def repo_files() -> list[str]:
    out = run_git(["ls-files", "-z"], REPO_ROOT)
    return [line for line in out.split("\0") if line]


def tree_of(cache: Path, ref: str) -> dict[str, str]:
    out = run_git(["ls-tree", "-r", ref], cache)
    mapping: dict[str, str] = {}
    for line in out.splitlines():
        meta, path = line.split("\t", 1)
        mapping[path] = meta.split()[2]
    return mapping


def commits_of(cache: Path, ref: str, since: str) -> list[tuple[str, str, str]]:
    out = run_git(
        ["log", f"--since={since}", "--format=%H%x09%ad%x09%s", "--date=short", ref],
        cache,
    )
    commits = []
    for line in out.splitlines():
        parts = line.split("\t", 2)
        if len(parts) == 3:
            commits.append((parts[0], parts[1], parts[2]))
    return commits


def ensure_cache(cache: Path, url: str) -> None:
    if (cache / "HEAD").exists():
        run_git(["fetch", "--quiet", "--filter=blob:none", "--tags", url, f"+refs/heads/{DEFAULT_REFERENCE}:refs/remotes/upstream/{DEFAULT_REFERENCE}"], cache)
    else:
        cache.parent.mkdir(parents=True, exist_ok=True)
        run_git(["clone", "--bare", "--filter=blob:none", "--quiet", url, str(cache)], REPO_ROOT)
    run_git(["config", "gc.auto", "0"], cache)


def load_pin() -> dict:
    return json.loads(PIN_PATH.read_text(encoding="utf-8"))


def load_overlay() -> tuple[list[OverlayRow], list[str]]:
    if not OVERLAY_PATH.exists():
        return [], ["sync/overlay.tsv is missing"]
    return parse_overlay(OVERLAY_PATH.read_text(encoding="utf-8"))


# --------------------------------------------------------------------------- #
# commands
# --------------------------------------------------------------------------- #


def command_check(quiet: bool) -> int:
    errors: list[str] = []
    if not PIN_PATH.exists():
        errors.append("sync/upstream.json is missing")
        pin: dict = {}
    else:
        pin = load_pin()
        errors.extend(validate_pin(pin))
    rows, overlay_errors = load_overlay()
    errors.extend(overlay_errors)
    ledger: dict = {}
    if not LEDGER_PATH.exists():
        errors.append("sync/ledger.json is missing; run --generate")
    else:
        ledger = json.loads(LEDGER_PATH.read_text(encoding="utf-8"))
    if rows:
        errors.extend(
            validate_overlay(rows, repo_files(), ledger.get("paths_only_downstream_list") or [])
        )
    if ledger:
        errors.extend(validate_ledger(ledger, pin, rows))
    for error in errors:
        print(f"upstream-sync ledger: {error}", file=sys.stderr)
    if not quiet:
        print(f"upstream-sync ledger: {'FAIL' if errors else 'ok'}")
    return 1 if errors else 0


def command_generate(args: argparse.Namespace) -> int:
    cache = Path(args.cache).resolve()
    ensure_cache(cache, args.upstream_url)
    reference = args.reference
    reference_ref = reference if reference != DEFAULT_REFERENCE else f"refs/remotes/upstream/{reference}"
    base_commit = run_git(["rev-parse", f"{args.base_tag}^{{commit}}"], cache).strip()
    reference_commit = run_git(["rev-parse", f"{reference_ref}^{{commit}}"], cache).strip()
    downstream_commit = run_git(["rev-parse", f"{args.downstream_ref}^{{commit}}"], REPO_ROOT).strip()

    upstream_tree = tree_of(cache, reference_commit)
    downstream_tree = tree_of(REPO_ROOT, downstream_commit)
    classification = classify_paths(upstream_tree, downstream_tree)

    backlog = fingerprint_backlog(
        commits_of(cache, reference_commit, args.window_since),
        commits_of(REPO_ROOT, downstream_commit, args.window_since),
    )
    by_month: dict[str, int] = {}
    for _, date, _ in backlog:
        by_month[date[:7]] = by_month.get(date[:7], 0) + 1

    rows, overlay_errors = load_overlay()
    scope = overlay_scope(args.overlay_ref, base_commit)
    uncovered = uncovered_overlay_paths(classification["paths_only_downstream"], rows)

    pin = {
        "repo": UPSTREAM_REPO,
        "base_tag": args.base_tag,
        "base_commit": base_commit,
        "reference": reference,
        "reference_commit": reference_commit,
        "window_since": args.window_since,
        "generated_at": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    }
    SYNC_DIR.mkdir(exist_ok=True)
    PIN_PATH.write_text(json.dumps(pin, indent=2) + "\n", encoding="utf-8")

    ledger = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": pin["generated_at"],
        "upstream": {
            "repo": UPSTREAM_REPO,
            "base_tag": pin["base_tag"],
            "base_commit": base_commit,
            "reference": reference,
            "reference_commit": reference_commit,
        },
        "downstream": {"ref": args.downstream_ref, "commit": downstream_commit},
        "counts": {
            "unported": len(backlog),
            "unported_by_month": dict(sorted(by_month.items())),
            "paths_common": classification["paths_common"],
            "paths_identical": classification["paths_identical"],
            "paths_differing": len(classification["paths_differing"]),
            "paths_only_upstream": len(classification["paths_only_upstream"]),
            "paths_only_downstream": len(classification["paths_only_downstream"]),
            "relocated_only_downstream": len(classification["relocated_only_downstream"]),
        },
        "critical_paths_changed": classification["critical_paths_changed"],
        "paths_only_downstream_list": classification["paths_only_downstream"],
        "overlay_scope": scope,
        "overlay_scope_undeclared": [
            path for path in scope if not any(glob_matches(row.path_glob, path) for row in rows)
        ],
        "unported_commits": [
            {"sha": sha, "date": date, "subject": subject} for sha, date, subject in backlog
        ],
    }
    LEDGER_PATH.write_text(json.dumps(ledger, separators=(",", ":")) + "\n", encoding="utf-8")

    print(f"upstream pin:      {UPSTREAM_REPO} {pin['base_tag']} ({base_commit[:10]})")
    print(f"reference:         {reference} ({reference_commit[:10]})")
    print(f"downstream:        {args.downstream_ref} ({downstream_commit[:10]})")
    print(f"unported commits:  {len(backlog)} {dict(sorted(by_month.items()))}")
    print(f"paths differing:   {len(classification['paths_differing'])}")
    print(f"paths only down:   {len(classification['paths_only_downstream'])} (relocated: {len(classification['relocated_only_downstream'])})")
    print(f"overlay uncovered: {len(uncovered)}")
    print(f"overlay scope:     {len(scope)} changed path(s) vs {args.base_tag}")
    print(f"critical changed:  {len(classification['critical_paths_changed'])}")
    if overlay_errors:
        for error in overlay_errors[:10]:
            print(f"overlay: {error}", file=sys.stderr)
    return 0


def command_plan() -> int:
    ledger = json.loads(LEDGER_PATH.read_text(encoding="utf-8"))
    rows, errors = load_overlay()
    for error in errors:
        print(f"overlay: {error}", file=sys.stderr)
    downstream_only = ledger.get("paths_only_downstream_list") or []
    uncovered = uncovered_overlay_paths(downstream_only, rows)
    print(f"{'covers':>7}  {'concern':28}  path_glob")
    for row in sorted(rows, key=lambda r: r.concern):
        covered = sum(1 for path in downstream_only if glob_matches(row.path_glob, path))
        print(f"{covered:7d}  {row.concern:28}  {row.path_glob}")
    print(f"\nuncovered downstream-only paths: {len(uncovered)}")
    grouped: dict[str, int] = {}
    for path in uncovered:
        key = "/".join(path.split("/")[:2])
        grouped[key] = grouped.get(key, 0) + 1
    for key, count in sorted(grouped.items(), key=lambda item: -item[1])[:40]:
        print(f"  {count:4d} {key}")
    return 0


def command_summary() -> int:
    ledger = json.loads(LEDGER_PATH.read_text(encoding="utf-8"))
    counts = ledger["counts"]
    print(f"base:        {ledger['upstream']['base_tag']} ({ledger['upstream']['base_commit'][:10]})")
    print(f"reference:   {ledger['upstream']['reference']} ({ledger['upstream']['reference_commit'][:10]})")
    print(f"generated:   {ledger['generated_at']}")
    print(f"unported:    {counts['unported']} {counts['unported_by_month']}")
    print(
        f"paths:       {counts['paths_common']} common ({counts['paths_identical']} identical, "
        f"{counts['paths_differing']} differing), {counts['paths_only_upstream']} upstream-only, "
        f"{counts['paths_only_downstream']} downstream-only "
        f"({counts['relocated_only_downstream']} relocated)"
    )
    critical = ledger.get("critical_paths_changed") or []
    print(f"critical:    {len(critical)} changed path(s) under {', '.join(CRITICAL_PREFIXES)}")
    grouped: dict[str, int] = {}
    for commit in ledger["unported_commits"]:
        for prefix in CRITICAL_PREFIXES:
            if prefix.strip("/").split("/")[-1] in commit["subject"].lower():
                grouped[prefix] = grouped.get(prefix, 0) + 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="offline ledger validation")
    mode.add_argument("--generate", action="store_true", help="refresh sync/ledger.json from upstream")
    mode.add_argument("--plan", action="store_true", help="show overlay coverage and uncovered paths")
    mode.add_argument("--summary", action="store_true", help="print the stored snapshot")
    parser.add_argument("--quiet", action="store_true", help="only print failures")
    parser.add_argument("--cache", default=str(DEFAULT_CACHE), help="blobless upstream clone")
    parser.add_argument("--upstream-url", default=UPSTREAM_URL)
    parser.add_argument("--base-tag", default="v0.9.3", help="trunk base tag")
    parser.add_argument("--reference", default=DEFAULT_REFERENCE, help="upstream comparison ref")
    parser.add_argument("--window-since", default=DEFAULT_WINDOW_SINCE, help="fork date for the backlog window")
    parser.add_argument("--downstream-ref", default="HEAD", help="ref used as the downstream side")
    parser.add_argument("--overlay-ref", default="HEAD", help="ref whose diff against the base is the overlay footprint")
    args = parser.parse_args()

    if args.check:
        return command_check(args.quiet)
    if args.generate:
        return command_generate(args)
    if args.plan:
        return command_plan()
    return command_summary()


if __name__ == "__main__":
    raise SystemExit(main())
