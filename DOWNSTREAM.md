# OnlineChefGroep/herdr — downstream distribution

Maintained public distribution of Herdr for OnlineChefGroep agent operations. This repository carries downstream product, agent-detection, gateway, fleet-control, packaging, and release changes that are validated independently before publication.

## v0.7.4 release baseline

- Release branch: `release/v0.7.4`
- Target branch: `main`
- Package version: `0.7.4`
- Toolchain: stable Rust, Zig `0.15.2`, Node.js `>=18`
- npm package: `onlinechefgroep-herdr`
- Release assets: Linux x86_64/arm64 and macOS x86_64/arm64
- Windows prebuilt: not published for v0.7.4

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
- CI builds the exact four artifacts produced by the release workflow
- macOS uses the patched Homebrew Zig 0.15 build path used by release CI
- Local Zig caches and build output are excluded from Git

## Release procedure

1. Merge the validated release pull request into `main`.
2. Create tag `v0.7.4` on the merge commit.
3. `release.yml` builds and publishes the four GitHub release assets.
4. The published release triggers `publish-distribution.yml`, which verifies all assets before publishing npm.
5. Update the Homebrew formula URL and SHA-256 values after the immutable release assets exist.

## Sync policy

- Keep downstream changes explicit and covered by CI.
- Reconcile upstream changes on a dedicated sync branch; do not mix upstream sync work into a release closeout.
- Never reuse upstream binaries or checksums for an OnlineChefGroep release.
