---
name: herdr-autopilot
description: "Project-specific autopilot facts for GroepOnline/herdr: CI-first verification, dedicated worktrees, and merge-gated delivery. Chains up to chef-autopilot for shared graph procedure, invariants, and safety gates."
role: satellite
ambient: true
extends: chef-autopilot
chains:
  skills:
  - chef-autopilot
invocable-by:
- user
- agent
- subagent
disable-model-invocation: false
context:
  project_types:
  - rust
  file_patterns:
  - AGENTS.md
  - .github/workflows/ci.yml
  - .github/quality-ci.md
  - justfile
  tools:
  - gh
  - git
  repos:
  - /home/joep/Documents/herdr
  signals:
  - pattern: autopilot
    weight: 0.9
  - pattern: CI
    weight: 0.5
owner: chefgroep
risk: mutating
last_reviewed: '2026-08-10'
---
# herdr-autopilot

Project layer for **chef-autopilot** in `/home/joep/Documents/herdr`.
Load the `chef-autopilot` overview first; load its full procedure when running
autopilot. This satellite records Herdr facts only.

## Project facts

- Work in `../herdr-worktrees/<task-slug>`; integrate through `../herdr`.
- The CI quality gate is the completion authority: `gh pr checks <pr>` and
  `gh run view <run> --log-failed` are evidence; local Rust/Zig builds are
  forbidden on the Cloud VM.
- `just check` is the normal-machine CI contract, not a Cloud-VM bypass.
- Autopilot runtime state is account-owned; inspect it with
  `~/.agents/skills/chef-autopilot/scripts/autopilot status` before acting.
- Current repo surfaces: `.github/workflows/ci.yml`,
  `.github/workflows/quality-autofix.yml`,
  `.github/workflows/quality-remediation.yml`, and `.github/quality-ci.md`.

## Deviation

Herdr's maintainer workflow uses dedicated task worktrees and GitHub Actions as
the Cloud-VM evidence source. Follow the shared meta-skill for all safety gates
and event-driven behavior; this satellite only records that repo-specific
routing.
