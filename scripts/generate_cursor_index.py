#!/usr/bin/env python3
"""Regenerate Cursor control-plane indexes from the filesystem."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

GENERATOR = "scripts/generate_cursor_index.py"
INDEX_VERSION = 1
WHEN_MAX = 60

PROD_DOMAIN_RE = re.compile(r"chefgroep\.(nl|online)", re.IGNORECASE)
ORG_HANDLE_RE = re.compile(r"online" + r"chefgroep", re.IGNORECASE)

HOOK_SUMMARIES: dict[str, tuple[str, str]] = {
    "sessionStart": (
        "Inject artifact catalog at session start",
        "sessionStart → fetch-cursor-artifacts.sh injects catalog + hash",
    ),
    "beforeSubmitPrompt": (
        "Allow prompt submission without catalog inject",
        "beforeSubmitPrompt → continue only (schema cannot inject here)",
    ),
    "postToolUse": (
        "Inject catalog once per conversation (cloud cold-start substitute)",
        "postToolUse → first tool use injects catalog; later calls dedupe",
    ),
    "workspaceOpen": (
        "Register `.cursor/plugins` paths when workspace opens",
        "workspaceOpen → returns pluginPaths for Cursor plugin discovery",
    ),
    "beforeShellExecution": (
        "Shell guard before command execution",
        "beforeShellExecution → policy hook (e.g. deny local Rust builds)",
    ),
}


@dataclass(frozen=True)
class IndexItem:
    name: str
    path: str
    kind: str
    summary: str
    when: str

    def as_dict(self) -> dict[str, str]:
        return {
            "name": self.name,
            "path": self.path,
            "kind": self.kind,
            "summary": self.summary,
            "when": self.when,
        }


def repo_root(start: Path | None = None) -> Path:
    if start is not None:
        return start
    script = Path(__file__).resolve()
    if script.parent.name == "scripts":
        parent = script.parent.parent
        if parent.name == ".cursor":
            return parent.parent
        return parent
    return script.parent


def truncate(text: str, limit: int = WHEN_MAX) -> str:
    collapsed = " ".join(text.split())
    if len(collapsed) <= limit:
        return collapsed
    return collapsed[: limit - 1].rstrip() + "…"


def assert_no_org_leak(text: str, label: str) -> None:
    if PROD_DOMAIN_RE.search(text):
        raise ValueError(f"{label}: org production domain in generated output")
    if ORG_HANDLE_RE.search(text):
        raise ValueError(f"{label}: org handle in generated output")


def parse_frontmatter(path: Path) -> dict[str, Any]:
    raw = path.read_text(encoding="utf-8")
    if not raw.startswith("---"):
        return {}
    parts = raw.split("---", 2)
    if len(parts) < 3:
        return {}
    meta = yaml.safe_load(parts[1])
    return meta if isinstance(meta, dict) else {}


def first_line_summary(description: str) -> str:
    text = " ".join(description.split())
    if not text:
        return ""
    first = text.split(". ", 1)[0]
    if not first.endswith("."):
        first = first.rstrip(".") + "."
    return first


def collect_from_glob(
    scan_root: Path,
    pattern: str,
    kind: str,
    name_fn,
    repo: Path,
) -> list[IndexItem]:
    items: list[IndexItem] = []
    for path in sorted(scan_root.glob(pattern)):
        if not path.is_file():
            continue
        meta = parse_frontmatter(path)
        name = str(meta.get("name") or name_fn(path))
        description = str(meta.get("description") or "").strip()
        summary = first_line_summary(description) or f"{kind} `{name}`."
        rel = path.relative_to(repo).as_posix()
        items.append(
            IndexItem(
                name=name,
                path=rel,
                kind=kind,
                summary=summary,
                when=truncate(description or summary),
            )
        )
    return items


def collect_skills(root: Path, extra_roots: list[Path] | None = None) -> list[IndexItem]:
    items = collect_from_glob(
        root / ".cursor" / "skills",
        "*/SKILL.md",
        "skill",
        lambda p: p.parent.name,
        root,
    )
    for extra in extra_roots or []:
        items.extend(
            collect_from_glob(
                extra, "*/SKILL.md", "skill", lambda p: p.parent.name, root
            )
        )
    return sorted(items, key=lambda i: (i.path, i.name))


def collect_agents(root: Path, extra_roots: list[Path] | None = None) -> list[IndexItem]:
    items = collect_from_glob(
        root / ".cursor" / "agents",
        "*.md",
        "agent",
        lambda p: p.stem,
        root,
    )
    for extra in extra_roots or []:
        items.extend(
            collect_from_glob(extra, "*.md", "agent", lambda p: p.stem, root)
        )
    return sorted(items, key=lambda i: (i.path, i.name))


def collect_commands(root: Path, extra_roots: list[Path] | None = None) -> list[IndexItem]:
    items = collect_from_glob(
        root / ".cursor" / "commands",
        "*.md",
        "command",
        lambda p: p.stem,
        root,
    )
    for extra in extra_roots or []:
        items.extend(
            collect_from_glob(extra, "*.md", "command", lambda p: p.stem, root)
        )
    return sorted(items, key=lambda i: (i.path, i.name))


def collect_rules(root: Path) -> list[IndexItem]:
    return collect_from_glob(
        root / ".cursor" / "rules",
        "*.mdc",
        "rule",
        lambda p: p.stem,
        root,
    )


def collect_hooks(root: Path) -> list[IndexItem]:
    hooks_json = root / ".cursor" / "hooks.json"
    if not hooks_json.is_file():
        return []
    data = json.loads(hooks_json.read_text(encoding="utf-8"))
    hooks = data.get("hooks") or {}
    items: list[IndexItem] = []
    for event in sorted(hooks.keys()):
        summary, when = HOOK_SUMMARIES.get(
            event,
            (f"Hook event `{event}`", f"Hook event `{event}` via hooks.json"),
        )
        items.append(
            IndexItem(
                name=event,
                path=".cursor/hooks.json",
                kind="hook",
                summary=summary,
                when=truncate(when),
            )
        )
    return items


def write_index_yaml(path: Path, items: list[IndexItem], allow_org_leak: bool) -> None:
    payload = {
        "version": INDEX_VERSION,
        "generated_by": GENERATOR,
        "items": [item.as_dict() for item in items],
    }
    rendered = yaml.safe_dump(payload, sort_keys=False, allow_unicode=True)
    if not allow_org_leak:
        assert_no_org_leak(rendered, str(path))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(rendered, encoding="utf-8")


def markdown_table(headers: list[str], rows: list[list[str]]) -> str:
    lines = [
        "| " + " | ".join(headers) + " |",
        "| " + " | ".join(["---"] * len(headers)) + " |",
    ]
    for row in rows:
        lines.append("| " + " | ".join(row) + " |")
    return "\n".join(lines)


def render_index_md(
    skills: list[IndexItem],
    agents: list[IndexItem],
    rules: list[IndexItem],
    commands: list[IndexItem],
    hooks: list[IndexItem],
    allow_org_leak: bool,
    regen_cmd: str,
) -> str:
    skill_rows = [[f"`{i.name}`", f"`{i.path}`", i.summary] for i in skills]
    agent_rows = [[f"`{i.name}`", f"`{i.path}`", i.summary] for i in agents]
    rule_rows = [[f"`{i.name}`", f"`{i.path}`", i.summary] for i in rules]
    command_rows = [[f"`/{i.name}`", f"`{i.path}`", i.summary] for i in commands]
    hook_rows = [[f"`{i.name}`", f"`{i.path}`", i.summary] for i in hooks]

    parts = [
        "# Cursor control plane index",
        "",
        f"> Auto-generated by `{GENERATOR}`. Regenerate instead of hand-editing.",
        "",
        "## Skills",
        "",
        markdown_table(["Name", "Path", "Summary"], skill_rows)
        if skill_rows
        else "_No skills found._",
        "",
        "## Agents",
        "",
        markdown_table(["Name", "Path", "Summary"], agent_rows)
        if agent_rows
        else "_No agents found._",
        "",
        "## Rules",
        "",
        markdown_table(["Name", "Path", "Summary"], rule_rows)
        if rule_rows
        else "_No rules found._",
        "",
        "## Commands",
        "",
        markdown_table(["Slash", "Path", "Summary"], command_rows)
        if command_rows
        else "_No commands found._",
        "",
        "## Hooks",
        "",
        markdown_table(["Event", "Path", "Summary"], hook_rows)
        if hook_rows
        else "_No hooks found._",
        "",
        "## Regenerate",
        "",
        "```bash",
        regen_cmd,
        "```",
        "",
    ]
    text = "\n".join(parts)
    if not allow_org_leak:
        assert_no_org_leak(text, ".cursor/INDEX.md")
    return text


INDEX_PATHS = (
    Path(".cursor/INDEX.md"),
    Path(".cursor/skills/.index.yaml"),
    Path(".cursor/agents/.index.yaml"),
    Path(".cursor/rules/.index.yaml"),
    Path(".cursor/commands/.index.yaml"),
    Path(".cursor/hooks/.index.yaml"),
)


def generate(
    root: Path | None = None,
    extra_skill_roots: list[Path] | None = None,
    extra_agent_roots: list[Path] | None = None,
    extra_command_roots: list[Path] | None = None,
    allow_org_leak: bool = False,
    regen_cmd: str = "python3 scripts/generate_cursor_index.py",
) -> dict[str, int]:
    repo = repo_root(root)
    skills = collect_skills(repo, extra_skill_roots)
    agents = collect_agents(repo, extra_agent_roots)
    rules = collect_rules(repo)
    commands = collect_commands(repo, extra_command_roots)
    hooks = collect_hooks(repo)

    write_index_yaml(repo / ".cursor" / "skills" / ".index.yaml", skills, allow_org_leak)
    write_index_yaml(repo / ".cursor" / "agents" / ".index.yaml", agents, allow_org_leak)
    write_index_yaml(repo / ".cursor" / "rules" / ".index.yaml", rules, allow_org_leak)
    write_index_yaml(repo / ".cursor" / "commands" / ".index.yaml", commands, allow_org_leak)
    write_index_yaml(repo / ".cursor" / "hooks" / ".index.yaml", hooks, allow_org_leak)

    index_md = render_index_md(
        skills, agents, rules, commands, hooks, allow_org_leak, regen_cmd
    )
    (repo / ".cursor" / "INDEX.md").write_text(index_md, encoding="utf-8")

    return {
        "skills": len(skills),
        "agents": len(agents),
        "rules": len(rules),
        "commands": len(commands),
        "hooks": len(hooks),
    }


def check(root: Path | None = None, **kwargs: Any) -> None:
    repo = repo_root(root)
    with tempfile.TemporaryDirectory(prefix="cursor-index-check-") as tmp:
        tmp_root = Path(tmp)
        cursor_src = repo / ".cursor"
        cursor_dst = tmp_root / ".cursor"
        shutil.copytree(
            cursor_src,
            cursor_dst,
            ignore=shutil.ignore_patterns(".state", "__pycache__"),
        )
        for extra_name in ("skills", "agents", "commands"):
            extra_src = repo / extra_name
            if extra_src.is_dir():
                shutil.copytree(extra_src, tmp_root / extra_name)

        # Extra roots are caller-supplied paths inside the real repo. generate()
        # resolves collected paths relative to the root it is given, so rebase
        # them onto the temp copy or collect_from_glob() raises on relative_to().
        for key in ("extra_skill_roots", "extra_agent_roots", "extra_command_roots"):
            roots = kwargs.get(key)
            if not roots:
                continue
            kwargs[key] = [tmp_root / Path(r).relative_to(repo) for r in roots]

        generate(tmp_root, **kwargs)

        for rel in INDEX_PATHS:
            expected = (tmp_root / rel).read_text(encoding="utf-8")
            actual_path = repo / rel
            if not actual_path.is_file():
                raise ValueError(f"missing index: {rel} (run generate_cursor_index.py)")
            actual = actual_path.read_text(encoding="utf-8")
            if actual != expected:
                raise ValueError(
                    f"stale index: {rel} — run: python3 scripts/generate_cursor_index.py"
                )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=None)
    ap.add_argument("--check", action="store_true")
    ap.add_argument(
        "--allow-org-leak",
        action="store_true",
        help="Skip org-handle/domain guard (repos with existing org references)",
    )
    ap.add_argument(
        "--include-root-catalog",
        action="store_true",
        help="Also index repo-root skills/, agents/, commands/",
    )
    ap.add_argument(
        "--regen-cmd",
        default="python3 scripts/generate_cursor_index.py",
        help="Command shown in INDEX.md regenerate section",
    )
    args = ap.parse_args()

    repo = repo_root(args.root)
    extra_skills = [repo / "skills"] if args.include_root_catalog else None
    extra_agents = [repo / "agents"] if args.include_root_catalog else None
    extra_cmds = [repo / "commands"] if args.include_root_catalog else None

    kwargs = {
        "root": args.root,
        "extra_skill_roots": extra_skills,
        "extra_agent_roots": extra_agents,
        "extra_command_roots": extra_cmds,
        "allow_org_leak": args.allow_org_leak,
        "regen_cmd": args.regen_cmd,
    }

    try:
        if args.check:
            check(**kwargs)
            print("generate_cursor_index: check ok")
            return 0
        counts = generate(**kwargs)
    except (OSError, ValueError, yaml.YAMLError) as exc:
        print(f"generate_cursor_index: error: {exc}", file=sys.stderr)
        return 1
    print(
        "generate_cursor_index: ok — "
        f"skills={counts['skills']} agents={counts['agents']} "
        f"rules={counts['rules']} commands={counts['commands']} "
        f"hooks={counts['hooks']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
