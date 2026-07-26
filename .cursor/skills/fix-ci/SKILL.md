---
name: fix-ci
description: Fix Herdr PR CI until Linux Quality gate is green. Use for failing PR checks, fix-ci loops, or when asked to babysit CI without local cargo.
---

# Herdr fix-ci

Minimal loop for PR CI failures on OnlineChefGroep/herdr.

## Rules

- **Never** run `cargo`, `rustc`, `cargo-nextest`, `clippy`, or `zig build` locally. Validate only via GitHub Actions.
- **SSOT merge gate:** `CI / Quality gate` on Linux (`Lint`, `Test`, `Maintenance`, `Release metadata`).
- **Ignore for PR merge:** `CI heavy`, Windows lint, musl smoke, CodeQL (unless explicitly scoped).
- Push to the PR branch; do not merge or force-push.

## Loop

1. `gh auth switch --user OnlineChef`
2. `gh pr checks <pr> --repo OnlineChefGroep/herdr --json name,state,link`
3. If workflows need approval (fork/bot PRs): `gh api -X POST repos/OnlineChefGroep/herdr/actions/runs/<id>/approve`
4. First actionable failure:
   ```bash
   gh run view <run_id> --repo OnlineChefGroep/herdr --log-failed
   ```
5. Smallest correct fix on the PR branch (isolated worktree OK).
6. Push, then `gh pr checks <pr> --repo OnlineChefGroep/herdr --watch`
7. Stop at green Quality gate or after three fix rounds.

## Classify failures

| Check | Typical fix |
|---|---|
| Lint / fmt | rustfmt drift; often autofixed by `quality-autofix.yml` |
| Lint / clippy | unused imports, type errors, `-D warnings` |
| Test / nextest | compile errors, failing unit tests |
| Maintenance | Python/Bun script tests under `scripts/` |
| Release metadata | `scripts/ci_quality.py`, npm pack dry-run |

For deeper remediation (sticky brief, dispatch payload), also load `.cursor/skills/herdr-quality-ci-remediation/SKILL.md`.
