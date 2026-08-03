---
name: maintain-herdr
description: Maintain the OnlineChefGroep/herdr fork — PR triage to merge-ready, rebase/conflict playbooks, draft triage, quality-gate redirects, fork hygiene. Use when opening, rebasing, or preparing PRs on the fork, handling draft/cursor PRs, resolving merge conflicts after bulk landings, or keeping fork main healthy relative to upstream ogulcancelik/herdr. Stops at merge-ready; never autonomously merges or force-pushes.
---

# Maintain herdr (fork ops)

Fork product home: `OnlineChefGroep/herdr`. Upstream (read-only): `ogulcancelik/herdr`.
All source changes land on the fork first — never push to upstream.

This skill ends at **merge-ready**. Merge, force-push, hard reset, draft→ready
mutation, `--admin`, CI bypass, and unconditional Dependabot merge are **Joep-gated**
and must not run autonomously.

## Identity gate (required first)

Before any GitHub mutation (`push`, PR open/edit, label, comment):

1. `gh auth status` — record the active login.
2. Maintainer path only when the acting account is `OnlineChefGroep` **or** Can
   explicitly authorizes maintainer work (see `AGENTS.md`). Joep’s operator login
   for org API/`gh` is typically `OnlineChef`; switch with
   `gh auth switch --user OnlineChef` before OnlineChefGroep GitHub actions when
   that account is the intended identity.
3. Otherwise follow the **external contributor** guardrail in `AGENTS.md` /
   `CONTRIBUTING.md`:
   - Do **not** open GitHub issues on a human’s behalf (CLI/API/browser).
   - Feature ideas → GitHub Discussions; bugs → draft a report the human submits.
   - First-time PR path needs an accepted issue plus maintainer `/approve @user`.
   - Never skip contribution process when asked.

If identity cannot be determined, treat as external contributor (fail closed).

## Remotes by URL (never assume names)

Remote **names** vary (`origin`/`upstream`/`fork`). Discover by URL every session:

```bash
git remote -v
# Resolve:
#   FORK_REMOTE     = remote whose fetch URL matches github.com[/:]OnlineChefGroep/herdr
#   UPSTREAM_REMOTE = remote whose fetch URL matches github.com[/:]ogulcancelik/herdr
```

Examples of valid layouts (do not hardcode one):

| Checkout style | Fork remote | Upstream remote |
| --- | --- | --- |
| Clone of OnlineChefGroep/herdr | often `origin` | often `upstream` |
| Clone of ogulcancelik/herdr + fork remote | often `fork` / custom | often `origin` |

If either URL is missing, add it explicitly before any playbook that needs it:

```bash
# Example when the working clone is the fork and upstream is absent:
git remote add upstream https://github.com/ogulcancelik/herdr.git
git fetch upstream

# Example when the working clone is upstream and the fork remote is absent:
git remote add fork https://github.com/OnlineChefGroep/herdr.git
git fetch fork
```

Always fetch the resolved remotes before rebase/sync. Rebase onto
`${FORK_REMOTE}/main`, not a guessed remote name.

Private mirror (optional): `OnlineChefGroep/herdr-private` — discover the same way
if present; never put secrets in skill docs or `fleet_ops.json`.

## Isolated worktrees (required)

Do **not** edit the shared integration checkout for substantive fork-ops work.

Layout (from `AGENTS.md`):

- shared integration: `~/Documents/herdr` (or `../herdr`)
- task worktrees: `~/Documents/herdr-worktrees/<task-slug>` (or `/tmp/herdr-<slug>`)
- task branches: `issue/<id>-<slug>` when an issue exists

All edits, commits, and validation run inside the task worktree. No nested
worktrees. After landing, parent/integration may fast-forward; this skill does
not push `main`.

## Merge-ready contract (end station)

A PR is **merge-ready** only when **all** of the following hold on the **exact
current head SHA** (pin it; re-verify after every push):

1. Required gates green on that SHA (Quality gate / CI) — evidence from notify or
   a **one-shot** `gh pr checks` / `gh pr view`, not a poll loop.
2. Independent review exists (another human/bot review). The acting agent must
   **not** count itself as the independent reviewer/approval.
3. No open **relevant P1** in the change scope (review threads / Greptile / latch).
4. Not blocked by draft/WIP policy (see Draft triage).
5. **Joep explicitly approves** any merge or force-push. Until then: stop and
   report merge-ready (or blockers) with the pinned SHA.

Never run:

- `gh pr merge` / squash-merge / `--admin`
- `gh pr ready` (draft→ready mutation)
- `git push --force` / `--force-with-lease` without Joep approval in-session
- `git reset --hard`
- CI bypass, skipping required checks, or “merge anyway”
- Unconditional Dependabot merge when green

## PR triage loop (safe)

1. List: `gh pr list -R OnlineChefGroep/herdr --state open --json number,title,isDraft,mergeable,headRefOid,author`
2. Oldest first to reduce conflict churn; re-fetch `${FORK_REMOTE}/main` between batches.
3. Per PR, pin `headRefOid`. On notify or when Joep asks, **one-shot** checks:
   `gh pr checks <n> -R OnlineChefGroep/herdr` (no `--watch`, no re-poll loops).
   Incomplete one-shot → report `CHECKS_UNKNOWN` and stop that PR.
4. Classify:
   - Green + not draft + merge-ready contract → **report merge-ready** (pinned SHA).
   - Green + draft → report; do **not** mark ready unless Joep asks.
   - `CONFLICTING` → rebase playbook in an isolated worktree; stop before any
     force-push; ask Joep before updating the remote branch.
   - Red quality gate → `herdr-quality-ci-remediation` skill (fix + push branch only).

## CI: notify-first

- Prefer GitHub notify / sticky PR comments over agent watchers.
- Forbidden: `gh run watch`, check poll loops, `ci-watcher` subagents, ending a
  turn with only “waiting on CI”.
- Allowed: at most one-shot status when Joep asks, after a notify, or when
  collecting merge-ready evidence for a pinned SHA.
- On laptop/cloud hooks that deny `cargo`/`just check`: validate via CI only.

## Rebase playbook (full clone / worktree)

Always use a full history clone or worktree. Shallow/partial clones break
`git checkout <pr-branch>`. Do not abort or reset rebase state in a shared
checkout; create a fresh task worktree instead.

Discover `FORK_REMOTE` / `UPSTREAM_REMOTE` by URL (above), then:

```bash
git fetch "$FORK_REMOTE" main
git fetch "$FORK_REMOTE" "<pr-branch>"

# Use a unique local repair branch; never reset an existing local PR branch.
git worktree add -b "repair/pr-<n>" /tmp/herdr-pr-<n> "${FORK_REMOTE}/<pr-branch>"
cd /tmp/herdr-pr-<n>
git rebase "${FORK_REMOTE}/main"
# resolve conflicts, commit as needed, then STOP.
# With Joep approval only:
#   git push --force-with-lease "$FORK_REMOTE" "HEAD:<pr-branch>"
# Then one-shot re-check on the new head SHA; report merge-ready or blockers.
```

If a shared checkout must be updated to fork main, prefer
`git switch main && git pull --ff-only "$FORK_REMOTE" main` (or a fresh worktree
at `${FORK_REMOTE}/main`). Do **not** `git reset --hard`.

## Conflict resolution patterns

- **Generated cursor indexes** (`.cursor/INDEX.md`, `.cursor/commands/.index.yaml`,
  `.cursor/skills/.index.yaml`, and sibling generated indexes): main’s version is
  authoritative during conflict — take the main-side file, then regenerate once
  with `python3 scripts/generate_cursor_index.py` (or `--allow-org-leak` when this
  fork already references org hosts). Do not hand-edit generated indexes.
- **`.cursor/environment.json`**: newer snapshot/install format on main wins; do
  not resurrect the old `install`/`terminals` shape.
- **Code conflicts**: resolve by hand. Local `just check` / `just test` when the
  machine allows; otherwise push and use CI (notify-first).
- **Overlapping one-line doc PRs**: land the oldest (after Joep merge approval),
  then rebase the rest — conflicts often dissolve.

## Quality gate

If `Apply mechanical quality fixes` / `CI / Quality gate` fails, use
`.cursor/skills/herdr-quality-ci-remediation`. Fix the real failure, push the PR
branch, and rely on notify / one-shot checks. **Never** bypass with `--admin`
or by weakening workflows.

## Draft triage

- Cursor-generated drafts (`cursor/*`) may be close to merge-ready once checks
  pass; authors sometimes leave them draft on purpose. Report status; do **not**
  run `gh pr ready` unless Joep explicitly asks in-session.
- WIP foundations (e.g. `freebuff/*`) stay draft — never mark ready or merge.
- Dependabot: triage like any other PR (checks, review, P1 latch). Do **not**
  merge solely because it is green.

## Hygiene

- Keep fork main aware of upstream: fetch `UPSTREAM_REMOTE`, then open/prepare an
  upstream-sync PR or ff-only integration path as appropriate. Prefer PR review
  over silent main mutation. Never push upstream.
- Commit / PR style: lowercase conventional commits; body `refs #<n>` (not
  `fixes`/`closes`/`resolves`); no AI co-author trailers.
- Propose commit messages before committing when working as a maintainer agent.
- Do not kill unrelated host processes as part of this skill.

## Related skills

- `herdr` — day-to-day development invariants
- `herdr-quality-ci-remediation` — Quality CI failures
- `verify-herdr` — user-like binary/TUI proof
- `chef-fleet` — Linear/GitHub/UDO/Kater SSOT and plugin boundaries
