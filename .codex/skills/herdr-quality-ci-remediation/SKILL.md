---
name: herdr-quality-ci-remediation
description: Remediate Herdr PR Quality CI failures end-to-end. Use when CI / Quality gate fails, a PR has the quality-remediation label or sticky comment, the user mentions quality CI / autofix / remediation, or a herdr-quality-remediation repository_dispatch payload is provided.
---

# Herdr Quality CI remediation

Canonical Cursor skill lives at `.cursor/skills/herdr-quality-ci-remediation/SKILL.md`.

This Codex-compatible copy exists so cloud agents discover the same playbook from `.codex/skills/`.

## Pre-flight

1. Read `.github/quality-ci.md` and the Quality CI section in `AGENTS.md`.
2. Prefer the fuller Cursor skill path when present:
   - `.cursor/skills/herdr-quality-ci-remediation/SKILL.md`
   - `.cursor/skills/herdr-quality-ci-remediation/references/quality-ci.md`
3. Confirm PR number, failed run id, branch, head SHA.
4. Stop if the PR has `ci-autofix-disabled`.

## Procedure

1. Inspect:
   ```bash
   gh run view <run_id> --repo OnlineChefGroep/herdr --log-failed
   gh pr checks <pr> --repo OnlineChefGroep/herdr
   ```
2. Classify: Lint / Test / Maintenance / Windows lint / Release metadata / Release smoke.
3. Fix the root cause on the PR branch (smallest correct change).
4. Do not skip tests or weaken `-D warnings`.
5. Validate via GitHub Actions only on Cloud VMs that forbid local cargo:
   ```bash
   git push -u origin <branch>
   gh pr checks <pr> --repo OnlineChefGroep/herdr --watch
   ```
6. Stop after green or 3 unsuccessful rounds.

## Subagent

For isolated remediation loops, delegate to `.cursor/agents/herdr-quality-ci-remediator.md`.
