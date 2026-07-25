# OnlineChefGroep/herdr — downstream distribution

Maintained public distribution of Herdr for OnlineChefGroep agent operations. This repository carries downstream product, agent-detection, gateway, fleet-control, packaging, and release changes that are validated independently before publication.

## v0.7.4 release baseline

- Release branch: `release/v0.7.4`
- Target branch: `main`
- Package version: `0.7.4`
- Toolchain: stable Rust, Zig `0.15.2`, Node.js `>=18`
- npm package: `onlinechefgroep-herdr`
- Release assets: Linux x86_64 only (Debian/amd64 fleet)
- Windows prebuilt: not published
- macOS / ARM64: not published by this fork

## Downstream patches

### Agent and operator support

- Agent manifests for `freebuff`, `junie`, and `openclaude`
- Fleet Ops Bar, fleet/plugin settings, workspace templates, and gateway API/SSE support

### Prefix and direct-attach behavior

- Default prefix is `ctrl+a`
- Direct attach uses the configured prefix without silently falling back
- Single-byte and multi-byte terminal sequences are preserved, including split input reads and literal doubled-prefix forwarding

### Distribution and release controls

- Cargo, npm, installer, changelog, and release metadata are version-aligned
- Release manifest generation reads `OnlineChefGroep/herdr`, not the upstream repository
- CI builds the single linux-x86_64 artifact produced by the release workflow
- Local Zig caches and build output are excluded from Git

## Release procedure

1. Merge the validated release pull request into `main`.
2. Create tag `v0.7.4` on the merge commit.
3. `release.yml` builds and publishes the linux-x86_64 GitHub release asset.
4. The published release triggers `publish-distribution.yml`, which verifies all assets before publishing npm.
5. Update the Homebrew formula URL and SHA-256 values after the immutable release assets exist.

## Sync policy

- Keep downstream changes explicit and covered by CI.
- Reconcile upstream changes on a dedicated sync branch; do not mix upstream sync work into a release closeout.
- Never reuse upstream binaries or checksums for an OnlineChefGroep release.
- Do not port Hermes-related upstream changes into this distribution.

## Version / install sources of truth

| Surface | Source |
|---|---|
| Package version | `Cargo.toml` (+ `npm/package.json` kept in sync) |
| Git tag | `vX.Y.Z` on `main` via `just release` |
| Stable curl install | `website/latest.json` → `https://herdr.chefgroep.nl/latest.json` |
| Homebrew | `OnlineChefGroep/homebrew-tap` formula `onlinechefgroep-herdr` |

Maintainer checks:

```bash
just release-status            # Cargo / tag / GitHub / local+live latest.json
python3 scripts/homebrew_formula.py --version X.Y.Z
```

After a release publishes `herdr-linux-x86_64`, regenerate the tap formula and open a PR on `OnlineChefGroep/homebrew-tap`. Do not leave macOS/ARM formula blocks pointing at older tags when those assets are not published for the new version.

## CI lanes

- Required PR check: `CI / Quality gate` (accepts skipped heavy jobs).
- Heavy Windows lint + musl smoke run on `main` pushes, `platform_heavy` path changes, or PRs labeled `ci-heavy`.
- Nightly/canary heavy lane: `.github/workflows/ci-heavy.yml` (not required for merge).
