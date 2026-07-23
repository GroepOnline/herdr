---
name: herdr-quality-ci-remediation
description: Remediate Herdr PR Quality CI failures end-to-end. Use when CI / Quality gate fails, a PR has the quality-remediation label or sticky comment, the user mentions quality CI / autofix / remediation, or a herdr-quality-remediation repository_dispatch payload is provided.
---

# Herdr Quality CI remediation

Self-contained Codex playbook for autonomous PR quality fixes. Comment bots are not the goal — make `CI / Quality gate` green.

Shared architecture notes live in the neutral repo doc `.github/quality-ci.md` (not another skill file).

## Pre-flight

1. Read `.github/quality-ci.md` and the Quality CI section in `AGENTS.md`.
2. Confirm PR number, failed run id, branch, head SHA.
3. Stop if the PR has `ci-autofix-disabled`.

## Procedure

1. Inspect failure:

   ```bash
   gh run view <run_id> --repo OnlineChefGroep/herdr --log-failed
   gh pr checks <pr> --repo OnlineChefGroep/herdr
   ```

   Also read the sticky PR comment marked `<!-- herdr-quality-remediation -->` when present.

2. Classify the failing surface:
   - `CI / Lint` — fmt / clippy
   - `CI / Test` — nextest
   - `CI / Maintenance` — python unittests / bun suites
   - `CI / Windows lint` — native Windows clippy
   - `CI / Release metadata` — `scripts/ci_quality.py`
   - `CI / Release smoke build (...)` — musl release smoke
   - `CI / Quality gate` — aggregator; fix the upstream job, not the gate itself

3. Fix the root cause with the smallest correct change on the PR branch.
   - Mechanical drift (npm VERSION vs Cargo.toml, rustfmt): rely on / verify `quality-autofix.yml`.
   - Do not skip tests, weaken `-D warnings`, or delete checks to go green.

4. Respect Herdr constraints from `AGENTS.md` (no production `unwrap()`, platform code isolated, unreleased docs only under `docs/next/`, lowercase conventional commits).

5. Validate via GitHub Actions on Cloud VMs that forbid local cargo:

   ```bash
   git push -u origin <branch>
   gh pr checks <pr> --repo OnlineChefGroep/herdr --watch
   ```

6. Stop after green or after **3** unsuccessful fix rounds.

## Parallelism

Do not run multiple remediator agents against the same PR branch unless a diagnoser has confirmed disjoint file ownership and push/rebase work is serialized. Prefer one remediator, or isolated branches/worktrees when parallelism is required.

## Final report shape

- PR / run / branch / head SHA
- Failing check(s)
- Root cause
- Files changed + commit SHAs
- Validation (`Quality gate` pass/fail)
- Remaining risk (if any)
