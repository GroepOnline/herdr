#!/usr/bin/env python3
"""One-shot final distribution hardening patch.

This script is intentionally deleted by the commit it creates. It keeps large
workflow/document edits deterministic and makes every replacement fail closed.
"""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8")


def replace_once(path: str, old: str, new: str, label: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match in {path}, found {count}")
    write(path, text.replace(old, new, 1))


def patch_workflows() -> None:
    replace_once(
        ".github/workflows/publish-distribution.yml",
        '          ruby -c "$RUNNER_TEMP/onlinechefgroep-herdr.rb"\n'
        '          diff -u packaging/homebrew/onlinechefgroep-herdr.rb "$RUNNER_TEMP/onlinechefgroep-herdr.rb"\n',
        '          ruby -c "$RUNNER_TEMP/onlinechefgroep-herdr.rb"\n',
        "remove impossible pre-release formula diff",
    )

    replace_once(
        ".github/workflows/release-portable-assets.yml",
        """        env:
          TAG: ${{ needs.prepare.outputs.tag }}
          VERSION: ${{ needs.prepare.outputs.version }}
""",
        """        env:
          TAG: ${{ needs.prepare.outputs.tag }}
          VERSION: ${{ needs.prepare.outputs.version }}
          GH_TOKEN: ${{ github.token }}
""",
        "give promotion formula generator GitHub credentials",
    )
    replace_once(
        ".github/workflows/release-portable-assets.yml",
        """          python3 scripts/ci_quality.py check-latest-json-manifest

      - name: Commit stable manifest
""",
        """          python3 scripts/ci_quality.py check-latest-json-manifest
          python3 scripts/homebrew_formula.py \\
            --version "$VERSION" \\
            --checksums release-assets/SHA256SUMS \\
            --output packaging/homebrew/onlinechefgroep-herdr.rb
          ruby -c packaging/homebrew/onlinechefgroep-herdr.rb

      - name: Commit stable manifest and formula
""",
        "generate formula only after immutable assets exist",
    )
    replace_once(
        ".github/workflows/release-portable-assets.yml",
        "          git add website/latest.json docs/next/product-announcement.json\n",
        "          git add website/latest.json docs/next/product-announcement.json packaging/homebrew/onlinechefgroep-herdr.rb\n",
        "commit generated formula with manifest",
    )

    replace_once(
        ".github/workflows/ci.yml",
        "python3 -m unittest scripts.test_agent_detection_manifest_check scripts.test_changelog scripts.test_ci_changed_paths scripts.test_ci_quality scripts.test_config_reference_check scripts.test_dev scripts.test_docs_translation_parity scripts.test_preview scripts.test_vendor_libghostty_vt scripts.test_vendor_portable_pty",
        "python3 -m unittest scripts.test_agent_detection_manifest_check scripts.test_changelog scripts.test_ci_changed_paths scripts.test_ci_quality scripts.test_config_reference_check scripts.test_dev scripts.test_docs_translation_parity scripts.test_homebrew_formula scripts.test_install_sh scripts.test_preview scripts.test_release_manifest_hardening scripts.test_vendor_libghostty_vt scripts.test_vendor_portable_pty",
        "add distribution tests to maintenance CI",
    )
    replace_once(
        ".github/workflows/ci.yml",
        """          node --check npm/bin/herdr.js
          (cd npm && npm pack --dry-run --ignore-scripts)
          python3 scripts/ci_quality.py check-release-metadata
""",
        """          node --check npm/bin/herdr.js
          sh -n website/install.sh
          python3 -m unittest scripts.test_install_sh
          (cd npm && npm pack --dry-run --ignore-scripts)
          python3 scripts/ci_quality.py check-release-metadata
""",
        "gate direct installer in release metadata CI",
    )


def patch_ci_path_classification() -> None:
    replace_once(
        "scripts/ci_changed_paths.py",
        '                exact=("justfile", "AGENTS.md", "DOWNSTREAM.md"),\n',
        '                exact=("justfile", "AGENTS.md", "DOWNSTREAM.md", "website/install.sh"),\n',
        "classify installer as maintenance",
    )
    replace_once(
        "scripts/ci_changed_paths.py",
        '                    "website/latest.json",\n                    "justfile",\n',
        '                    "website/latest.json",\n                    "website/install.sh",\n                    "justfile",\n',
        "classify installer as release metadata",
    )
    replace_once(
        "scripts/test_ci_changed_paths.py",
        """    def test_platform_heavy_classification(self) -> None:
""",
        """    def test_installer_runs_maintenance_and_release_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "out"
            with (
                patch.object(
                    ci_changed_paths,
                    "changed_files",
                    return_value=["website/install.sh"],
                ),
                patch.dict(os.environ, {"GITHUB_OUTPUT": str(out)}, clear=False),
            ):
                self.assertEqual(ci_changed_paths.main(), 0)
            text = out.read_text(encoding="utf-8")
            self.assertIn("maintenance=true", text)
            self.assertIn("release_meta=true", text)
            self.assertIn("docs_only=false", text)

    def test_platform_heavy_classification(self) -> None:
""",
        "test installer CI classification",
    )


def patch_quality_metadata() -> None:
    replace_once(
        "scripts/ci_quality.py",
        """def read_npm_package_version(root: Path) -> str:
""",
        """def read_cargo_license(root: Path) -> str:
    try:
        data = tomllib.loads(read_text(root, CARGO_TOML_PATH))
    except tomllib.TOMLDecodeError as exc:
        raise QualityError(f"invalid TOML in {CARGO_TOML_PATH}: {exc}") from exc
    license_name = data.get("package", {}).get("license")
    if not isinstance(license_name, str) or not license_name:
        raise QualityError(f"{CARGO_TOML_PATH} is missing package.license")
    return license_name


def read_npm_package_version(root: Path) -> str:
""",
        "read canonical Cargo license",
    )
    replace_once(
        "scripts/ci_quality.py",
        '    Path("website/src/content/docs/install.mdx"),\n',
        '    Path("website/install.sh"),\n    Path("website/src/content/docs/install.mdx"),\n    Path("docs/next/website/src/content/docs/install.mdx"),\n',
        "scan all installer surfaces for product drift",
    )
    replace_once(
        "scripts/ci_quality.py",
        """    package = load_json_object(root, NPM_PACKAGE_PATH)
    if package.get("os") != ["linux", "darwin"]:
""",
        """    package = load_json_object(root, NPM_PACKAGE_PATH)
    cargo_license = read_cargo_license(root)
    if package.get("license") != cargo_license:
        raise QualityError(
            f"{NPM_PACKAGE_PATH} license {package.get('license')!r} does not match Cargo.toml {cargo_license!r}"
        )
    if package.get("os") != ["linux", "darwin"]:
""",
        "enforce npm/Cargo license parity",
    )

    replace_once(
        "scripts/test_ci_quality.py",
        "f'[package]\\nname = \"herdr\"\\nversion = \"{cargo_version}\"\\n',",
        "f'[package]\\nname = \"herdr\"\\nversion = \"{cargo_version}\"\\nlicense = \"AGPL-3.0-or-later\"\\n',",
        "add Cargo license to fixture",
    )
    replace_once(
        "scripts/test_ci_quality.py",
        '                    "version": npm_version,\n                    "repository": {\n',
        '                    "version": npm_version,\n                    "license": "AGPL-3.0-or-later",\n                    "repository": {\n',
        "add npm license to fixture",
    )
    replace_once(
        "scripts/test_ci_quality.py",
        """    def test_check_release_metadata_uses_matching_changelog_section(self) -> None:
""",
        """    def test_check_release_metadata_rejects_license_drift(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_fixture(root, "1.2.3", "1.2.3")
            package_path = root / "npm/package.json"
            package = json.loads(package_path.read_text(encoding="utf-8"))
            package["license"] = "MIT"
            package_path.write_text(json.dumps(package, indent=2) + "\\n", encoding="utf-8")

            with self.assertRaisesRegex(QualityError, "does not match Cargo.toml"):
                check_release_metadata(root)

    def test_check_release_metadata_uses_matching_changelog_section(self) -> None:
""",
        "test license drift gate",
    )


def patch_manifest_legacy_preservation() -> None:
    replace_once(
        "scripts/changelog.py",
        """    normalized_assets: dict[str, str] = {}
    for target in ASSET_TARGETS:
        url = value.get(target)
        if not isinstance(url, str) or not url.strip():
            raise ChangelogError(f"{label} is missing asset URL for {target}")
        normalized_assets[target] = url.strip()
    return normalized_assets
""",
        """    normalized_assets: dict[str, str] = {}
    for target, url in value.items():
        if not isinstance(target, str) or not target.strip():
            raise ChangelogError(f"{label} contains an invalid asset target")
        if not isinstance(url, str) or not url.strip():
            raise ChangelogError(f"{label} is missing asset URL for {target}")
        normalized_assets[target] = url.strip()
    return normalized_assets
""",
        "preserve every legacy plain-string asset",
    )
    replace_once(
        "scripts/test_release_manifest_hardening.py",
        """    def test_latest_manifest_mirrors_promoted_assets_and_preserves_legacy(self) -> None:
""",
        """    def test_legacy_archive_preserves_all_plain_string_targets(self) -> None:
        checksums = self.checksums()
        promoted = promoted_release_assets("1.2.3", checksums)
        legacy_assets = {
            "linux-x86_64": "https://example.invalid/legacy-linux-x86_64",
            "linux-aarch64": "https://example.invalid/legacy-linux-aarch64",
            "macos-aarch64": "https://example.invalid/legacy-macos-aarch64",
        }
        manifest = json.loads(
            build_latest_json(
                "1.2.3",
                "### Fixed\\n- Hardened release promotion.",
                promoted,
                protocol=42,
                releases={
                    "1.2.2": {
                        "notes": "### Fixed\\n- Legacy release.",
                        "assets": legacy_assets,
                    }
                },
            )
        )
        self.assertEqual(manifest["releases"]["1.2.2"]["assets"], legacy_assets)

    def test_latest_manifest_mirrors_promoted_assets_and_preserves_legacy(self) -> None:
""",
        "test multi-target legacy preservation",
    )


def patch_homebrew_auth() -> None:
    replace_once(
        "scripts/homebrew_formula.py",
        "import json\nimport re\n",
        "import json\nimport os\nimport re\n",
        "import environment for GitHub token",
    )
    replace_once(
        "scripts/homebrew_formula.py",
        """    req = urllib.request.Request(
        url,
        headers={
            "Accept": "application/vnd.github+json",
            "User-Agent": "onlinechefgroep-herdr-homebrew-generator",
        },
    )
""",
        """    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "onlinechefgroep-herdr-homebrew-generator",
    }
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, headers=headers)
""",
        "authenticate Homebrew release lookups",
    )


def patch_justfile() -> None:
    replace_once(
        "justfile",
        "python3 -m unittest scripts.test_agent_detection_manifest_check scripts.test_changelog scripts.test_ci_changed_paths scripts.test_ci_quality scripts.test_config_reference_check scripts.test_dev scripts.test_docs_translation_parity scripts.test_preview scripts.test_vendor_libghostty_vt scripts.test_vendor_portable_pty",
        "python3 -m unittest scripts.test_agent_detection_manifest_check scripts.test_changelog scripts.test_ci_changed_paths scripts.test_ci_quality scripts.test_config_reference_check scripts.test_dev scripts.test_docs_translation_parity scripts.test_homebrew_formula scripts.test_install_sh scripts.test_preview scripts.test_release_manifest_hardening scripts.test_vendor_libghostty_vt scripts.test_vendor_portable_pty",
        "add distribution tests to local maintenance",
    )
    replace_once(
        "justfile",
        """    node --check npm/bin/herdr.js
    (cd npm && npm pack --dry-run --ignore-scripts)
""",
        """    node --check npm/bin/herdr.js
    sh -n website/install.sh
    python3 -m unittest scripts.test_install_sh
    (cd npm && npm pack --dry-run --ignore-scripts)
""",
        "test direct installer locally",
    )
    replace_once(
        "justfile",
        """    cargo update -p herdr --offline
    just check
    git add CHANGELOG.md docs/next/CHANGELOG.md Cargo.toml Cargo.lock
""",
        """    cargo update -p herdr --offline
    python3 scripts/ci_quality.py sync-release-metadata
    just check
    git add CHANGELOG.md docs/next/CHANGELOG.md Cargo.toml Cargo.lock npm/package.json
""",
        "sync npm version in release preparation",
    )
    replace_once(
        "justfile",
        """    @echo "v{{version}} released — GitHub Actions building binaries and updating website/latest.json"
    @echo "After assets publish: python3 scripts/homebrew_formula.py --version {{version}}"
    @echo "Then update OnlineChefGroep/homebrew-tap Formula/onlinechefgroep-herdr.rb"
""",
        """    @echo "v{{version}} tagged — Release builds Linux x86_64 first"
    @echo "Portable assets then verify all four binaries plus SHA256SUMS, atomically promote latest.json, and publish npm + Homebrew"
""",
        "document automated release chain",
    )
    replace_once(
        "justfile",
        """# Show Cargo / tag / GitHub release / local+live latest.json alignment
""",
        """# Strictly verify GitHub release, checksums, local manifest, live manifest, and asset URLs
release-verify version="":
    #!/usr/bin/env bash
    set -euo pipefail
    cargo_version="$(sed -n 's/^version = "\\(.*\\)"/\\1/p' Cargo.toml | head -1)"
    version="${1:-$cargo_version}"
    python3 scripts/changelog.py verify-release-state \\
      --version "$version" \\
      --live-url https://herdr.chefgroep.nl/latest.json

# Show Cargo / tag / GitHub release / local+live latest.json alignment
""",
        "add strict release verification recipe",
    )


def patch_docs() -> None:
    replace_once(
        "README.md",
        "The OnlineChefGroep distribution tracks upstream Herdr and adds the CHEF release, integration and deployment layer. The current stable line is **v0.7.5**; `main` can contain validated post-release fixes before the next tag.",
        "The OnlineChefGroep distribution tracks upstream Herdr and adds the CHEF release, integration and deployment layer. The current stable line is **v0.7.6**; `main` can contain validated post-release fixes before the next tag.",
        "update stable README version",
    )
    replace_once(
        "README.md",
        "Linux and macOS direct install:\n",
        "Linux and macOS direct install (the manifest SHA-256 is verified before replacement):\n",
        "document direct installer verification",
    )
    replace_once(
        "README.md",
        "Homebrew (OnlineChefGroep tap — Linux x86_64 currently):\n",
        "Homebrew (OnlineChefGroep tap — Linux/macOS, Intel/ARM):\n",
        "document Homebrew matrix",
    )
    replace_once(
        "README.md",
        """brew install OnlineChefGroep/tap/onlinechefgroep-herdr
```

mise:
""",
        """brew install OnlineChefGroep/tap/onlinechefgroep-herdr
```

npm or Bun (Linux/macOS, Intel/ARM; postinstall verifies SHA256SUMS):

```bash
npm install --global onlinechefgroep-herdr
# or: bun add --global onlinechefgroep-herdr
```

mise:
""",
        "add npm README installation lane",
    )
    replace_once(
        "README.md",
        """brew upgrade herdr
mise upgrade herdr
""",
        """brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr
npm install --global onlinechefgroep-herdr@latest
mise upgrade herdr
""",
        "document package-managed updates",
    )

    replace_once(
        "docs/next/README.md",
        """curl -fsSL https://herdr.chefgroep.nl/install.sh | sh
```

on windows preview beta:
""",
        """curl -fsSL https://herdr.chefgroep.nl/install.sh | sh
```

The Linux/macOS installer verifies the selected manifest SHA-256 before atomically replacing the binary.

on windows preview beta:
""",
        "document next README installer verification",
    )
    replace_once(
        "docs/next/README.md",
        """update later with `brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`. the upstream `brew install herdr` formula can lag behind [herdr.chefgroep.nl/latest.json](https://herdr.chefgroep.nl/latest.json).

or install with mise:
""",
        """update later with `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`. the upstream `brew install herdr` formula can lag behind [herdr.chefgroep.nl/latest.json](https://herdr.chefgroep.nl/latest.json).

or install with npm/bun (Linux/macOS, Intel/ARM):

```bash
npm install --global onlinechefgroep-herdr
# or: bun add --global onlinechefgroep-herdr
```

The npm postinstall verifies the release `SHA256SUMS` before installing the native binary.

or install with mise:
""",
        "add next README npm lane",
    )
    replace_once(
        "docs/next/README.md",
        "Homebrew, mise, and Nix installs update through `brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`, `mise upgrade herdr`, or your Nix workflow",
        "Homebrew, npm, mise, and Nix installs update through `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`, `npm install --global onlinechefgroep-herdr@latest`, `mise upgrade herdr`, or your Nix workflow",
        "document next package updates",
    )

    install_doc = '''---
title: Install Herdr
description: Install, verify, and update Herdr on Linux, macOS, and Windows beta.
---

Stable Herdr binaries are published for Linux and macOS on x86_64 and ARM64. Native Windows support remains preview-only beta.

## Direct install

On Linux or macOS:

```bash
curl -fsSL https://herdr.chefgroep.nl/install.sh | sh
```

The installer reads the selected channel manifest, requires a 64-hex SHA-256 for the platform asset, downloads into the destination directory, verifies the binary, and only then atomically replaces `herdr`. A failed download or checksum leaves the existing binary untouched.

On Windows preview beta:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://herdr.chefgroep.nl/install.ps1 | iex"
```

Windows uses versioned install folders and a `current` junction so an update does not overwrite a running `herdr.exe`.

## Homebrew

Use the OnlineChefGroep tap for the release synchronized with Herdr's manifest:

```bash
brew tap OnlineChefGroep/tap
brew install OnlineChefGroep/tap/onlinechefgroep-herdr
```

The formula supports Linux and macOS on Intel and ARM. Update with:

```bash
brew update
brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr
```

## npm or Bun

The npm package supports Linux and macOS on Intel and ARM. Its postinstall downloads the matching release binary and verifies it against `SHA256SUMS` before installation.

```bash
npm install --global onlinechefgroep-herdr
# or
bun add --global onlinechefgroep-herdr
```

Update with:

```bash
npm install --global onlinechefgroep-herdr@latest
```

## mise

```bash
mise use -g herdr
```

If mise reports `herdr not found in mise tool registry`, update mise and retry. Older mise versions can temporarily use:

```bash
mise use -g github:OnlineChefGroep/herdr
```

Update with `mise upgrade herdr`.

## Nix

Herdr provides a flake that builds from source:

```bash
nix run github:OnlineChefGroep/herdr/v0.x.y
nix build github:OnlineChefGroep/herdr/v0.x.y
nix profile install github:OnlineChefGroep/herdr/v0.x.y
```

Replace `v0.x.y` with the intended release tag. For profile installs, use `nix profile list` followed by `nix profile upgrade <index-or-name>`. For flake inputs, update the Herdr input and rebuild the owning system or environment.

## Manual download

Download the binary and `SHA256SUMS` from [GitHub releases](https://github.com/OnlineChefGroep/herdr/releases), verify the selected asset, then place it on your PATH.

| System | Asset |
| --- | --- |
| Linux x86_64 | `herdr-linux-x86_64` |
| Linux aarch64 | `herdr-linux-aarch64` |
| macOS Intel | `herdr-macos-x86_64` |
| macOS Apple silicon | `herdr-macos-aarch64` |

Example:

```bash
sha256sum --check SHA256SUMS
chmod +x herdr-linux-x86_64
mv herdr-linux-x86_64 ~/.local/bin/herdr
```

On macOS, use `shasum -a 256` when `sha256sum` is unavailable.

## Direct-install update channels

`herdr update` is only for direct installs managed by Herdr. Package-manager installs are intentionally detected and must update through Homebrew, npm, mise, or Nix.

Linux and macOS direct installs use `stable` by default:

```bash
herdr update
herdr channel set preview
herdr channel set dev
herdr channel set stable
```

Preview and dev are direct-install channels. Windows beta defaults to preview and cannot switch to stable until stable Windows assets exist.

A running server continues using its current process after the binary changes. Restart the default session with `herdr server stop` followed by `herdr`; restart a named session with `herdr session stop <name>` followed by `herdr session attach <name>`. `herdr update --handoff` remains experimental.

## Verify

```bash
herdr --version
herdr
```

If the shell cannot find `herdr`, restart it or add the selected install directory to `PATH`.
'''
    write("website/src/content/docs/install.mdx", install_doc)
    write("docs/next/website/src/content/docs/install.mdx", install_doc)

    write(
        "npm/README.md",
        '''# onlinechefgroep-herdr

OnlineChefGroep's Herdr distribution: a terminal-native multiplexer and control surface for AI coding agents.

## Install

```bash
npm install --global onlinechefgroep-herdr
# or
bun add --global onlinechefgroep-herdr
```

The package supports Linux and macOS on x64 and ARM64. During postinstall it downloads the matching binary from the same GitHub release as the package version, validates it against `SHA256SUMS`, and atomically installs it inside the package. Native Windows binaries remain preview-only and are not installed by this package.

## Update

```bash
npm install --global onlinechefgroep-herdr@latest
```

`herdr update` detects npm-managed binaries and directs them back to npm instead of overwriting files inside `node_modules`.

## Quick start

```bash
herdr
herdr --version
herdr config
```

## Build from source

```bash
git clone https://github.com/OnlineChefGroep/herdr.git
cd herdr
cargo build --release --locked
```

## License

AGPL-3.0-or-later — see [LICENSE](https://github.com/OnlineChefGroep/herdr/blob/main/LICENSE).
''',
    )

    write(
        "DOWNSTREAM.md",
        '''# OnlineChefGroep/herdr — downstream distribution

Maintained public Herdr distribution for OnlineChefGroep agent operations. Downstream product, agent-detection, gateway, fleet-control, packaging, and release changes remain explicit and independently validated.

## Distribution contract

- Canonical repository: `OnlineChefGroep/herdr`
- Package version source: `Cargo.toml`; `npm/package.json` is mechanically synchronized
- Toolchain: pinned Rust in CI, Zig `0.15.2`, Node.js `>=18`
- Stable native assets: Linux and macOS, x86_64 and ARM64
- npm package: `onlinechefgroep-herdr`
- Homebrew tap/formula: `OnlineChefGroep/homebrew-tap` / `onlinechefgroep-herdr`
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
- npm postinstall verifies `SHA256SUMS` and installs inside `node_modules/onlinechefgroep-herdr/bin`.
- `herdr update` detects Homebrew, npm, mise, and Nix paths and refuses to overwrite package-managed files.
- Homebrew, npm, mise, and Nix installations update through their respective package manager.

## Sync policy

- Reconcile upstream on dedicated sync branches; do not combine upstream sync with release closeout.
- Never reuse upstream binaries or checksums for an OnlineChefGroep release.
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
''',
    )


def main() -> None:
    patch_workflows()
    patch_ci_path_classification()
    patch_quality_metadata()
    patch_manifest_legacy_preservation()
    patch_homebrew_auth()
    patch_justfile()
    patch_docs()
    print("final distribution hardening patch applied")


if __name__ == "__main__":
    main()
