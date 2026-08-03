---
name: herdr-thermos-audit
description: >-
  Report-only parallel thermos audit with a live Herdr pane/tab canvas (map,
  findings, connections, where-live). Use for /audit, thermos audit, "niet
  handelen", puur review, report-only branch/commit audits inside HERDR_ENV=1,
  or when wiring forked OnlineChefGroep/herdr + Cursor Thermos + herdr-ops.
  Never spawn review-fixer or edit the audited product tree from this skill.
disable-model-invocation: true
---

# Herdr Thermos Audit

Report-only dirigent: parallel Thermos lanes + optional connections mapper,
painted into Herdr tabs beside the conductor. Findings stay findings.

## Best-fit (forked herdr + plugins)

**Right seam today**

| Layer | Role |
|-------|------|
| Cursor Thermos plugin | Dual Task subagents (bugs + code quality) |
| This skill + hooks | Report-only policy, fan-out/fan-in, brain ingest |
| Herdr fork CLI (`HERDR_ENV=1`) | Tabs/panes as the human canvas |
| herdr-ops | Global install into `~/.cursor` (SSOT for Joep machines) |

**Not the right seam (yet)**

- Shipping this as a `herdr-plugins` Fleet Ops plugin — no need; it is agent orchestration, not a fleet bar action.
- Chrome / cursor-ide-browser canvas — forbidden; Herdr is the surface.

**Ceiling later** (do not block on these): Herdr workspace template `audit` with pre-labeled tabs; optional plugin that persists the board under `$HERDR_PLUGIN_STATE_DIR`. Current pane+tab+markdown approach is the best we can do without Herdr product work.

## Hard contract

1. **No mutations** of the audited repo (no edit/commit/push/fix agents).
2. **`audit-report-only` outranks `review-means-fix`** on `/audit`, `thermos`, `niet handelen`, `report-only`, `geen fix`.
3. **Parallel 2–3 lanes** in one turn (`run_in_background: true`).
4. **Herdr canvas when `HERDR_ENV=1`** — neighbor pane + tabs; sticky `/usr/bin/less -R` (never `glow`/`less` via bash that races with `send-keys q`).
5. **Brain** `ingest_report` at pack time and at FINAL fan-in.
6. Stop at findings. Offer `review-means-fix` only if Joep asks.

## Lane model

| Lane | Subagent | Job |
|------|----------|-----|
| A | `thermo-nuclear-review-subagent` | Bugs, security, breakages, flag leaks |
| B | `thermo-nuclear-code-quality-review-subagent` | Maintainability, structure, spaghetti |
| C | `audit-scope-mapper` (or Task generalPurpose) | Connections / mermaid / hot edges |

## Workflow

0. **Look first** — `herdr workspace/tab/pane/worktree list` + existing `/tmp/<repo>-audit-*/canvas`. Reuse labeled panes/tabs; restore content if `/tmp` was wiped (brain reports). Never blind-create duplicate tabs. Details: [references/herdr-canvas.md](references/herdr-canvas.md).
1. **Scope** — repo, `base..HEAD` or named commit/PR. Patch under `/tmp/<repo>-audit-<date>/diff/`.
2. **Gather** — file list + hot excerpts (absolute paths). No megabyte dumps in prompts.
3. **Canvas** — tabs `map` / `findings` / `connections` + neighbor `where-live`. Prefer `herdr pane run <id> /usr/bin/less -R <file>` on **existing** panes. Optional fix lanes: `herdr worktree create|open` instead of orphan `/tmp` worktrees.
4. **Fan-out** — A+B(+C) same scope package, REPORT ONLY in every prompt.
5. **Fan-in** — dedupe; A∩B raises weight. Write `01-findings.md` + `03-FINAL.md`.
6. **Paint** — reload less viewers (`R` or re-run less binary). Focus `findings` for Joep.
7. **Brain** — FINAL ingest with SHA + top findings + artifact paths (also the restore source if `/tmp` dies).
8. **Stop** — no fixers unless asked. Chain handoff: see joep-workflow-templates `Audit → Thermos → Canvas`, then optional `Review → Fix → CI`.

## Prompt package (every lane)

- Absolute repo path + HEAD SHA + base
- Absolute patch / excerpts paths
- `REPORT ONLY — do not edit, commit, push, or spawn fixers`
- Return: severity-sorted `path:line`, evidence, suggested fix (text only)

## Herdr CLI pitfalls (learned)

- **Look first** — inventory panes/tabs/worktrees before create.
- Prefer `herdr pane run <pane> /usr/bin/less -R <absfile>` — avoids `q`+`bash` paste races.
- Do not `send-keys q` then immediately `pane run` with `bash -lc`.
- Restore content onto existing panes when `/tmp` canvas vanished; only recreate tabs if labels are gone.
- Record current tab/pane ids in `WHERE-LIVE.md` after every restore.
- Keep focus on Joep's workspace; use `--no-focus` when creating background tabs.
- Prefer `herdr worktree open|create` for multi-agent fix lanes Joep should see.

## Related

- Thermos plugin (Cursor cache) — lane A/B implementations
- `find-bugs` — single-agent report-only alternative
- `review-means-fix` — only after report-only lifts
- herdr-ops `install.sh` — global install of this skill + agents + RO hook
- OnlineChefGroep/herdr `.cursor/skills/herdr-thermos-audit` — repo-local copy for herdr workspaces
