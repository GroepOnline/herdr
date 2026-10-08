---
name: herdr-upstream-sync
description: Rebase the GroepOnline/herdr fork onto upstream herdrdev/herdr as real ancestry plus a named overlay, keep the ledger honest, and land each wave behind its gate. Use when syncing the fork with upstream, when upstream stable tags ship, when the drift gate fails, or when the moshi hooks / integration and detection work must be ported from upstream.
---

# Herdr upstream sync (trunk + overlay)

Canonical process doc: [`.github/upstream-sync.md`](../../../.github/upstream-sync.md).
Read it before touching anything; this skill is the operational summary.

## Mental model

Upstream is the trunk (real commits, `git rebase --onto` works). Downstream is
an overlay: one commit per concern, each with a ledger entry. Never replay
upstream commits by hand when the trunk can carry them, and never mix overlay
work into a trunk commit.

## Order of work

1. Freeze and pin: tag the rollback point, refresh `sync/upstream.json` with
   `scripts/upstream_sync_ledger.py --generate`, and confirm `--check` passes.
2. Trunk: worktree/branch from the upstream stable tag, not from `main`.
3. Waves A→G from `.github/upstream-sync.md`. Hooks and detection (waves B and C)
   are the user-visible moshi-hook path; they come before TUI churn.
4. Land each wave as its own PR with its own gate. Red gate = no merge.

## Ledger discipline

- A path is overlay only when it has a row in `sync/overlay.tsv` with owner,
  rationale, tests and a removal condition.
- New downstream-only paths must be classified by the same PR that adds them.
- `python3 scripts/upstream_sync_ledger.py --check` is part of `just maintenance`;
  fix the ledger, never the check.

## Non-negotiables

- No `-X theirs`, no bulk replay scripts, no conflict shortcuts.
- Never weaken or skip a test to pass a gate; report unrunnable gates instead.
- Rust/Zig builds are CI-owned on the Cloud VM: validate with `gh pr checks` and
  `gh run view <id> --log-failed`, plus the downloaded smoke binary.
- Vendored patches follow `vendor/*.patches.md` (indexed and reverse-appliable).
- `PROTOCOL_VERSION` and integration asset versions follow the rules in
  `AGENTS.md`; stable docs only describe released behaviour.

## First checks when a sync session starts

```bash
git log --oneline -1 main
python3 scripts/upstream_sync_ledger.py --check
python3 scripts/upstream_sync_ledger.py --summary   # backlog by month and area
```
