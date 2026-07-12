# OnlineChefGroep/herdr-private - Downstream Distribution

Thin downstream distribution of [ogulcancelik/herdr](https://github.com/ogulcancelik/herdr).
NOT a standalone fork.

## Baseline (2026-07-12)

- Private HEAD: bc30461
- Upstream base: f36d804
- Current version: 0.7.1
- Target version: 0.7.3
- Downstream commits: 2
- Behind upstream master: 138
- Toolchain: Rust 1.96.1, Zig 0.15.2, Bun 1.3+

## Downstream Patches

### PATCH-001: Custom agent manifests (freebuff, junie, openclaude)
- ID: chef-agents-v1
- Status: 2 commits on f36d804
- Test: manual only
- Upstream: not submitted

### PATCH-002: Default prefix Ctrl+A
- ID: chef-prefix-v1
- Plan: move to config profile (Phase 3)

## Sync
- Rebase on upstream stable within 1 week
- Max downstream delta: less than 10 commits
## NPM Package (2026-07-12)
- Name: onlinechefgroep-herdr
- Directory: npm/
- Wrapper: npm/bin/herdr.js (cross-platform binary launcher)
- Postinstall: npm/install.js (downloads binary from GitHub Releases)
- Publish: cd npm && npm publish --access public
