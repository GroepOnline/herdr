# GroepOnline/herdr — downstream distribution

Maintained public Herdr distribution for GroepOnline agent operations. Downstream product, agent-detection, gateway, fleet-control, packaging, and release changes remain explicit and independently validated.

## Distribution contract

- Canonical repository: `GroepOnline/herdr`
- Package version source: `Cargo.toml`; `npm/package.json` is mechanically synchronized
- Toolchain: pinned Rust in CI, Zig `0.15.2`, Node.js `>=18`
- Stable native assets: Linux and macOS, x86_64 and ARM64
- npm package: `groeponline-herdr`
- Homebrew tap/formula: `GroepOnline/homebrew-tap` / `groeponline-herdr`
- Stable install manifest: `website/latest.json` → `https://herdr.chefgroep.nl/latest.json`
- Native Windows: preview-only until a stable Windows release contract is declared

## Release trust chain

1. `just release-prepare X.Y.Z` finalizes changelog/docs, updates Cargo, synchronizes npm metadata, and runs the full validation set.
2. `just release-publish X.Y.Z` tags the validated `main` commit.
3. `release.yml` builds Linux x86_64 and creates the GitHub release. It does not promote `latest.json`.
4. `release-portable-assets.yml` builds the other three stable targets, uploads all four binaries plus `SHA256SUMS`, downloads them again, and verifies every checksum.
5. Only after that complete verification does the workflow atomically promote `website/latest.json`, mirror the current asset metadata under `releases`, and generate the four-target Homebrew formula.
6. `publish-distribution.yml` smoke-tests the npm tarball/postinstall, publishes npm idempotently, and pushes the generated formula to the Homebrew tap. Manual runs default to `dry_run`.
7. `just release-verify X.Y.Z` strictly compares the GitHub release, checksums, local manifest, live manifest, and asset URLs.

A partial release can exist on GitHub while portable builders finish, but it cannot become the public stable manifest or package-manager release.

## Installer/update ownership

- Direct Linux/macOS installs require manifest SHA-256 metadata and atomically replace the binary only after verification.
- npm postinstall verifies `SHA256SUMS` and installs inside `node_modules/groeponline-herdr/bin`.
- `herdr update` detects Homebrew, npm, mise, and Nix paths and refuses to overwrite package-managed files.
- Homebrew, npm, mise, and Nix installations update through their respective package manager.

## Preview ownership and rollback

- Owner: `@GroepOnline/ci-bots`, enforced for the preview workflow, helper, and manifest through `.github/CODEOWNERS`.
- The preview channel is disabled by default for the downstream distribution; publishes happen only via explicit workflow dispatch with a `commit` input (rollback path).
- Source branch: `main`; requested commits must be reachable from `origin/main`.
- Artifact namespace: preview prereleases in `GroepOnline/herdr` only. CI rejects references outside the downstream release namespace.
- Publication requires a complete checksum target matrix. Missing, extra, or malformed SHA-256 values abort before the manifest commit.
- Rollback: dispatch the Preview workflow with an earlier downstream commit reachable from `main`. If no safe downstream preview exists, keep top-level `assets` empty so clients fail closed; never restore external asset URLs.
- Website ownership follows the repository-backed `website/preview.json`; no separate `*.pages.dev` binding is a source of truth.

## Sync policy

- Reconcile upstream on dedicated sync branches; do not combine upstream sync with release closeout.
- Never reuse upstream binaries or checksums for an GroepOnline release.
- Keep downstream behavior covered by CI and preserve the baseline rather than removing tests or weakening runtime functionality.
- Do not port Hermes-related upstream changes into this distribution.

## Maintainer checks

```bash
just release-metadata
just maintenance
just release-status
just release-verify 0.7.6
```

The required PR check is `CI / Quality gate`. Heavy platform lanes run on relevant paths, `main` pushes, or PRs labeled `ci-heavy`; the nightly/canary lane remains non-required.
