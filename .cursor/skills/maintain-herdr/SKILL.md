---
name: maintain-herdr
description: Maintain the GroepOnline/herdr fork — PR autopilot, rebase and conflict resolution, draft triage, quality-gate fixes, release/hygiene. Use when opening, rebasing, merging or closing PRs on the fork, handling draft/cursor PRs, resolving merge conflicts after bulk merges, or keeping fork/main healthy relative to upstream ogulcancelik/herdr.
---

# Maintain herdr (fork ops)

Fork: `GroepOnline/herdr`. Upstream: `ogulcancelik/herdr` (read-only).
All source changes land on the fork first — never push to upstream directly.

## Remotes

```bash
git remote -v
# origin  = https://github.com/ogulcancelik/herdr.git   (read-only upstream)
# fork    = git@github.com:GroepOnline/herdr.git    (push here)
# private = git@github.com:GroepOnline/herdr-private.git
```

Clone locally at `~/herdr`. Rebase on `fork/main`, never `origin/main`.

## PR autopilot loop

1. `gh pr list -R GroepOnline/herdr --state open --json number,title,isDraft,mergeable`
2. Per PR: `gh pr checks <n>` — merge only when every check passes or skips.
3. Green + not draft → `gh pr merge <n> --squash --delete-branch`.
4. Green + draft (cursor-generated, user opted in) → `gh pr ready <n>`, then merge.
5. `CONFLICTING` → rebase on `fork/main`, resolve, force-push, re-check, merge.
6. Oldest first to minimise conflict churn; re-sync `fork/main` after every few merges.

## Rebase playbook (full clone)

Always work from a full clone. A fresh clone only has `origin`, so rename it and
re-add the remotes to match the layout above before anything references `fork/*`:

```bash
git clone https://github.com/GroepOnline/herdr.git ~/herdr && cd ~/herdr
git remote rename origin fork
git remote add origin https://github.com/ogulcancelik/herdr.git   # read-only upstream
git fetch fork && git fetch origin
```

Shallow/partial clones break `git checkout <pr-branch>` and stale rebase state
(`.git/rebase-merge` leftover) must be aborted before new work:

```bash
git rebase --abort 2>/dev/null || true   # clear stale rebase first
git checkout main && git reset --hard fork/main
git fetch fork <pr-branch>
git checkout -B <pr-branch> fork/<pr-branch>
git rebase fork/main
# resolve conflicts, then:
git push --force-with-lease fork <pr-branch>
gh pr checks <n>   # wait for re-run
gh pr merge <n> -R GroepOnline/herdr --squash --delete-branch
```

## Conflict resolution patterns

- **Generated cursor index files** (`.cursor/INDEX.md`, `.cursor/commands/.index.yaml`,
  `.cursor/skills/.index.yaml`): main's version is authoritative — take HEAD, then
  regenerate with `python3 scripts/generate_cursor_index.py` (or run the repo hook).
- **`.cursor/environment.json`**: newer snapshot/install format on main wins; do not
  resurrect the old `install`/`terminals` shape.
- **Code conflicts**: resolve by hand, run `just check` / `just test` locally when
  toolchain is available; on cloud VMs validate via `gh pr checks`.
- **Overlapping one-line doc PRs** (e.g. several OWNERS.yaml/catalog PRs): merge the
  oldest, then rebase the rest — the conflicts resolve themselves.

## Quality gate

If `Apply mechanical quality fixes` / quality gate fails, use the
`herdr-quality-ci-remediation` project skill (`.cursor/skills/herdr-quality-ci-remediation`).
Do not bypass with `--admin` unless the change is docs-only and gated on a review bot.

## Draft triage

Cursor-generated drafts (`cursor/*` branches) are usually merge-ready once checks pass;
the author may have left them draft deliberately. Confirm with the user before marking
ready. WIP PRs (e.g. `freebuff/*` UI foundations) stay draft — never touch.

## Hygiene

- Keep `fork/main` up to date with upstream: `git fetch origin && git merge --ff-only origin/main` (or `gh pr` for upstream merges; usually upstream is a no-op since the fork drifts).
- Dependabot PRs (deps groups) merge once green.
- sofie has 7.5 GiB RAM: max 2–3 agent panes; kill runaway `mcp-server-cloudflare` (100% CPU) before it swap-thrashes.
- PR body lines: `refs #<n>` (not `fixes`/`closes`); lowercase conventional commits; no AI co-author trailers.
