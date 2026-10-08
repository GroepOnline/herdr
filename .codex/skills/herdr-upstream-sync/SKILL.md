---
name: herdr-upstream-sync
description: Codex playbook for bringing GroepOnline/herdr in step with upstream herdrdev/herdr using a real-ancestry trunk plus a named overlay, with a checked ledger and per-wave gates.
---

# Herdr upstream sync (Codex playbook)

Shared architecture and process notes live in the neutral repo doc
`.github/upstream-sync.md`. Read it first; this file is the Codex-side summary.

## Goal

`main` must descend from a real upstream commit so future syncs are a
`git rebase --onto upstream/<tag>`, while every downstream-only change is a
named overlay commit with a ledger row.

## Session start

```bash
python3 scripts/upstream_sync_ledger.py --check
python3 scripts/upstream_sync_ledger.py --summary
git fetch https://github.com/herdrdev/herdr refs/heads/master:refs/remotes/upstream/master
```

## Work rules

1. One concern per overlay commit; ledger row in the same PR.
2. Trunk commits are upstream commits — do not replay upstream work manually.
3. Each wave (A–G in `.github/upstream-sync.md`) lands behind its own gate.
4. Hooks and detection first: they are the mobile-client (Moshi) path.
5. No `-X theirs`, no bulk cherry-pick scripts, no test weakening. Report a gate
   that cannot run as "not run", never as passed.
6. Validate through GitHub Actions (`gh pr checks`), never with local Rust builds.

## Definition of done for a wave

- gate green, overlay rows carry test names, ledger `--check` green, and the
  wave's changes visible in `git log` as trunk-vs-overlay without mixing.
