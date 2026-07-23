---
name: herdr-quality-ci-remediation
description: Remediate Herdr PR Quality CI failures end-to-end. Use when CI / Quality gate fails, a PR has the quality-remediation label or sticky comment, the user mentions quality CI / autofix / remediation, or a herdr-quality-remediation repository_dispatch payload is provided.
---

# Herdr Quality CI remediation

Canonical playbook for autonomous PR quality fixes. Comment bots are not the goal — make `CI / Quality gate` green.

## Pre-flight

1. Read [references/quality-ci.md](references/quality-ci.md) (or `.github/quality-ci.md` if the reference copy is stale).
2. Read the Quality CI section in `AGENTS.md`.
3. Confirm inputs: PR number, failed run id, branch, head SHA.
4. If the PR has label `ci-autofix-disabled`, stop and report that remediation is opted out.

## Procedure

1. **Inspect failure**
   ```bash
   gh run view <run_id> --repo OnlineChefGroep/herdr --log-failed
   gh pr checks <pr> --repo OnlineChefGroep/herdr
   ```
   Also read the sticky PR comment marked `<!-- herdr-quality-remediation -->` when present.

2. **Classify the failing surface**
   - `CI / Lint` — fmt / clippy
   - `CI / Test` — nextest
   - `CI / Maintenance` — python unittests / bun suites
   - `CI / Windows lint` — native Windows clippy
   - `CI / Release metadata` — `scripts/ci_quality.py`
   - `CI / Release smoke build (...)` — musl release smoke
   - `CI / Quality gate` — aggregator; fix the upstream job, not the gate itself

3. **Fix the root cause**
   - Prefer the smallest correct change on the PR branch.
   - Mechanical drift (npm VERSION vs Cargo.toml, rustfmt): rely on / verify `quality-autofix.yml`; do not fight it.
   - Real bugs / test failures / config-reference drift: fix production code or fixtures.
   - Do not skip tests, weaken `-D warnings`, or delete checks to go green.

4. **Respect Herdr constraints**
   - No production `unwrap()`.
   - Platform code stays in `src/platform/`.
   - Do not edit stable website docs / root CHANGELOG for unreleased work; use `docs/next/` when docs are needed.
   - Lowercase conventional commits; `refs #<n>` when applicable. No AI co-author lines.

5. **Validate via GitHub Actions**
   On this Cloud VM, local `cargo` / `just test|check|lint|ci` are forbidden.
   ```bash
   git push -u origin <branch>
   gh pr checks <pr> --repo OnlineChefGroep/herdr --watch
   ```
   On failure: `gh run view <new-run-id> --log-failed`, fix, push again.

6. **Stop conditions**
   - Success: `CI / Quality gate` passes.
   - Give up after **3** unsuccessful fix rounds; report remaining blocker with log excerpt and hypothesized root cause.
   - Do not spam PR comments. At most update the sticky remediation brief if the workflow did not.

## Parallelism

When multiple independent failures exist (e.g. Test + Maintenance), dispatch parallel fix agents / the `herdr-quality-ci-remediator` subagent per surface, then re-watch the gate once.

## Final report shape

- PR / run / branch / head SHA
- Failing check(s)
- Root cause
- Files changed + commit SHAs
- Validation (`Quality gate` pass/fail)
- Remaining risk (if any)
