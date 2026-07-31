#!/usr/bin/env python3
"""One-shot checkout patch for large release-chain files."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


def patch_latest_json() -> None:
    path = ROOT / "website/latest.json"
    manifest = json.loads(path.read_text(encoding="utf-8"))
    version = "0.7.6"
    if str(manifest.get("version", "")).removeprefix("v") != version:
        raise RuntimeError(
            f"latest.json expected v{version}, found {manifest.get('version')!r}"
        )

    checksums = {
        "herdr-linux-x86_64": "8f0785c5e9e471e03e7611d6b987b60bf1f9a7db0f25bec95c11f54e156a561a",
        "herdr-linux-aarch64": "ee943dfdf577fe48d5430f6bd4631bbad67019719f6af57422bd345d015eb671",
        "herdr-macos-x86_64": "6347f7bf567d48a7617ff6c6b9f8d5ca13fe9b492ccd30ca4fda4c5a34a41dff",
        "herdr-macos-aarch64": "4abe05e0858758b166e1e08bd57305363e151dfa65b1bd3cd9e1f99e5ca1dfcd",
    }
    targets = {
        "linux-x86_64": "herdr-linux-x86_64",
        "linux-aarch64": "herdr-linux-aarch64",
        "macos-x86_64": "herdr-macos-x86_64",
        "macos-aarch64": "herdr-macos-aarch64",
    }
    assets = {
        target: {
            "url": (
                "https://github.com/OnlineChefGroep/herdr/releases/download/"
                f"v{version}/{asset}"
            ),
            "sha256": checksums[asset],
        }
        for target, asset in targets.items()
    }
    manifest["assets"] = assets
    releases = manifest.get("releases")
    if not isinstance(releases, dict) or not isinstance(releases.get(version), dict):
        raise RuntimeError(f"latest.json is missing releases.{version}")
    releases[version]["assets"] = assets
    path.write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def patch_release_workflow() -> None:
    text = read(".github/workflows/release.yml")
    marker = "\n  update-latest-json:\n"
    if text.count(marker) != 1:
        raise RuntimeError(
            f"release.yml: expected one update-latest-json job, found {text.count(marker)}"
        )
    text = text.split(marker, 1)[0].rstrip() + "\n"
    write(".github/workflows/release.yml", text)


def patch_portable_workflow() -> None:
    text = read(".github/workflows/release-portable-assets.yml")
    marker = "\n  update-manifest:\n"
    if text.count(marker) != 1:
        raise RuntimeError(
            f"release-portable-assets.yml: expected one update-manifest job, found {text.count(marker)}"
        )
    prefix = text.split(marker, 1)[0].rstrip()
    suffix = r'''

  update-manifest:
    name: Atomically promote stable manifest
    needs: [prepare, publish]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10 # v6
        with:
          ref: main
          fetch-depth: 0
          ssh-key: ${{ secrets.RELEASE_DEPLOY_KEY }}

      - name: Download and verify published release
        env:
          GH_TOKEN: ${{ github.token }}
          TAG: ${{ needs.prepare.outputs.tag }}
        run: |
          set -euo pipefail
          mkdir -p release-assets
          gh release download "$TAG" --repo "$GITHUB_REPOSITORY" \
            --pattern 'herdr-linux-x86_64' \
            --pattern 'herdr-linux-aarch64' \
            --pattern 'herdr-macos-x86_64' \
            --pattern 'herdr-macos-aarch64' \
            --pattern 'SHA256SUMS' \
            --dir release-assets

          expected=(
            herdr-linux-x86_64
            herdr-linux-aarch64
            herdr-macos-x86_64
            herdr-macos-aarch64
          )
          for asset in "${expected[@]}"; do
            test -s "release-assets/$asset"
            grep -Eq "^[a-fA-F0-9]{64}[[:space:]]+\\*?${asset}$" release-assets/SHA256SUMS
          done
          test "$(grep -Ec '^[a-fA-F0-9]{64}[[:space:]]+\\*?herdr-' release-assets/SHA256SUMS)" -eq 4
          (cd release-assets && sha256sum --check --strict SHA256SUMS)

      - name: Promote complete release into latest.json
        env:
          TAG: ${{ needs.prepare.outputs.tag }}
          VERSION: ${{ needs.prepare.outputs.version }}
        run: |
          set -euo pipefail
          ANNOUNCEMENT_PATH="$RUNNER_TEMP/product-announcement.json"
          ANNOUNCEMENT_ORIGINAL_PATH="$RUNNER_TEMP/product-announcement-original.json"
          git show "${TAG}:docs/next/product-announcement.json" > "$ANNOUNCEMENT_PATH"
          cp "$ANNOUNCEMENT_PATH" "$ANNOUNCEMENT_ORIGINAL_PATH"
          python3 scripts/changelog.py validate-product-announcement --path "$ANNOUNCEMENT_PATH"
          RELEASE_PROTOCOL="$(
            git show "${TAG}:src/protocol/wire.rs" |
              python3 -c 'import re, sys; match = re.search(r"pub const PROTOCOL_VERSION: u32 = (\d+);", sys.stdin.read()); sys.exit(1) if match is None else print(match.group(1))'
          )"

          python3 scripts/changelog.py sync-latest-json \
            --version "$VERSION" \
            --output website/latest.json \
            --announcement "$ANNOUNCEMENT_PATH" \
            --protocol "$RELEASE_PROTOCOL" \
            --checksums release-assets/SHA256SUMS \
            --allow-current-version

          if ! cmp -s "$ANNOUNCEMENT_ORIGINAL_PATH" "$ANNOUNCEMENT_PATH" &&
             cmp -s "$ANNOUNCEMENT_ORIGINAL_PATH" docs/next/product-announcement.json; then
            printf 'null\n' > docs/next/product-announcement.json
          fi

          python3 scripts/ci_quality.py check-latest-json-manifest

      - name: Commit stable manifest
        env:
          VERSION: ${{ needs.prepare.outputs.version }}
        run: |
          set -euo pipefail
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
          git add website/latest.json docs/next/product-announcement.json
          git diff --cached --quiet || git commit -m "docs: atomically promote v$VERSION"
          git push origin main

      - name: Trigger website deploy
        env:
          DEPLOY_HOOK: ${{ secrets.CLOUDFLARE_PAGES_DEPLOY_HOOK }}
        run: |
          if [ -z "$DEPLOY_HOOK" ]; then
            echo "CLOUDFLARE_PAGES_DEPLOY_HOOK not set; relying on Cloudflare git auto-deploy"
            exit 0
          fi
          curl -fsS -X POST "$DEPLOY_HOOK" > /dev/null
          echo "Triggered Cloudflare Pages production deploy"

  publish-distribution:
    name: Publish npm and Homebrew
    needs: [prepare, update-manifest]
    uses: ./.github/workflows/publish-distribution.yml
    with:
      tag: ${{ needs.prepare.outputs.tag }}
      dry_run: false
    secrets: inherit
'''
    write(".github/workflows/release-portable-assets.yml", prefix + suffix)


def patch_update_rs() -> None:
    text = read("src/update.rs")

    text = replace_once(
        text,
        'const MISE_UPDATE_COMMAND: &str = "mise upgrade herdr";\n',
        'const NPM_UPDATE_COMMAND: &str = "npm install --global onlinechefgroep-herdr@latest";\n'
        'const MISE_UPDATE_COMMAND: &str = "mise upgrade herdr";\n',
        "add npm update command",
    )

    text = replace_once(
        text,
        """    if is_homebrew_managed_install() {
        HOMEBREW_UPDATE_COMMAND
    } else if is_mise_managed_install() {
        MISE_UPDATE_COMMAND
""",
        """    if is_homebrew_managed_install() {
        HOMEBREW_UPDATE_COMMAND
    } else if is_npm_managed_install() {
        NPM_UPDATE_COMMAND
    } else if is_mise_managed_install() {
        MISE_UPDATE_COMMAND
""",
        "route npm update command",
    )

    text = replace_once(
        text,
        """        HOMEBREW_UPDATE_COMMAND => {
            "detach, run `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`, then restart this Herdr session when ready".to_string()
        }
        MISE_UPDATE_COMMAND => {
""",
        """        HOMEBREW_UPDATE_COMMAND => {
            "detach, run `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`, then restart this Herdr session when ready".to_string()
        }
        NPM_UPDATE_COMMAND => {
            "detach, run `npm install --global onlinechefgroep-herdr@latest`, then restart this Herdr session when ready".to_string()
        }
        MISE_UPDATE_COMMAND => {
""",
        "add npm update instruction",
    )

    text = replace_once(
        text,
        "fn is_nix_managed_install() -> bool {\n",
        """fn is_npm_managed_install() -> bool {
    let Ok(current_exe) = env::current_exe() else {
        return false;
    };

    is_npm_managed_exe_path_following_links(&current_exe)
}

fn is_nix_managed_install() -> bool {
""",
        "add npm current install detector",
    )

    text = replace_once(
        text,
        """    if is_homebrew_managed_install() {
        Some(
            "Use `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr` (or `brew upgrade herdr` if you installed the `herdr` formula alias) to update Homebrew installs.",
        )
    } else if is_mise_managed_install() {
""",
        """    if is_homebrew_managed_install() {
        Some(
            "Use `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr` (or `brew upgrade herdr` if you installed the `herdr` formula alias) to update Homebrew installs.",
        )
    } else if is_npm_managed_install() {
        Some("Use `npm install --global onlinechefgroep-herdr@latest` to update npm installs.")
    } else if is_mise_managed_install() {
""",
        "add npm package manager guidance",
    )

    text = replace_once(
        text,
        """    if is_homebrew_managed_exe_path_following_links(path) {
        Some(
            "preview and dev channels are only available for direct Herdr installs; Homebrew installs update through `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`",
        )
    } else if is_mise_managed_exe_path_following_links(path) {
""",
        """    if is_homebrew_managed_exe_path_following_links(path) {
        Some(
            "preview and dev channels are only available for direct Herdr installs; Homebrew installs update through `brew update && brew upgrade OnlineChefGroep/tap/onlinechefgroep-herdr`",
        )
    } else if is_npm_managed_exe_path_following_links(path) {
        Some(
            "preview and dev channels are only available for direct Herdr installs; npm installs update through `npm install --global onlinechefgroep-herdr@latest`",
        )
    } else if is_mise_managed_exe_path_following_links(path) {
""",
        "add npm prerelease rejection",
    )

    text = replace_once(
        text,
        """    is_homebrew_managed_exe_path_following_links(path)
        || is_mise_managed_exe_path_following_links(path)
""",
        """    is_homebrew_managed_exe_path_following_links(path)
        || is_npm_managed_exe_path_following_links(path)
        || is_mise_managed_exe_path_following_links(path)
""",
        "classify npm as package managed",
    )

    text = replace_once(
        text,
        "fn is_nix_store_exe_path_following_links(path: &Path) -> bool {\n",
        """fn is_npm_managed_exe_path_following_links(path: &Path) -> bool {
    if is_npm_managed_exe_path(path) {
        return true;
    }

    path.canonicalize()
        .is_ok_and(|path| is_npm_managed_exe_path(&path))
}

fn is_nix_store_exe_path_following_links(path: &Path) -> bool {
""",
        "add npm symlink detector",
    )

    text = replace_once(
        text,
        """fn is_nix_store_exe_path(path: &Path) -> bool {
    path.starts_with("/nix/store")
}
""",
        """fn is_npm_managed_exe_path(path: &Path) -> bool {
    if path.file_name() != Some(std::ffi::OsStr::new("herdr")) {
        return false;
    }
    let Some(bin_dir) = path.parent() else {
        return false;
    };
    if bin_dir.file_name() != Some(std::ffi::OsStr::new("bin")) {
        return false;
    }
    let Some(package_dir) = bin_dir.parent() else {
        return false;
    };
    if package_dir.file_name() != Some(std::ffi::OsStr::new("onlinechefgroep-herdr")) {
        return false;
    }
    package_dir
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name == "node_modules")
}

fn is_nix_store_exe_path(path: &Path) -> bool {
    path.starts_with("/nix/store")
}
""",
        "add npm path shape detector",
    )

    homebrew_guard = """    if is_homebrew_managed_install() {
        if channel.is_prerelease() {
            return Err(format!(
                "self-update is disabled for Homebrew installs; the {} channel is only available for direct Herdr installs",
                channel.as_str()
            ));
        }
        return Err(format!(
            "self-update is disabled for Homebrew installs; run `{HOMEBREW_UPDATE_COMMAND}`"
        ));
    }

"""
    npm_guard = homebrew_guard + """    if is_npm_managed_install() {
        if channel.is_prerelease() {
            return Err(format!(
                "self-update is disabled for npm installs; the {} channel is only available for direct Herdr installs",
                channel.as_str()
            ));
        }
        return Err(format!(
            "self-update is disabled for npm installs; run `{NPM_UPDATE_COMMAND}`"
        ));
    }

"""
    text = replace_once(text, homebrew_guard, npm_guard, "guard npm self update")

    test_marker = """    #[test]
    fn mise_install_path_is_detected() {
"""
    npm_tests = """    #[test]
    fn npm_global_install_path_is_detected() {
        let path = Path::new(
            "/usr/local/lib/node_modules/onlinechefgroep-herdr/bin/herdr",
        );

        assert!(is_npm_managed_exe_path(path));
        assert!(is_package_manager_managed_exe_path(path));
        assert_eq!(
            preview_channel_rejection_for_exe_path(path),
            Some(
                "preview and dev channels are only available for direct Herdr installs; npm installs update through `npm install --global onlinechefgroep-herdr@latest`"
            )
        );
        assert_eq!(
            update_install_instruction(NPM_UPDATE_COMMAND),
            "detach, run `npm install --global onlinechefgroep-herdr@latest`, then restart this Herdr session when ready"
        );
    }

    #[test]
    fn npm_path_requires_exact_package_and_native_binary_shape() {
        assert!(!is_npm_managed_exe_path(Path::new(
            "/usr/local/lib/node_modules/another-package/bin/herdr",
        )));
        assert!(!is_npm_managed_exe_path(Path::new(
            "/usr/local/lib/node_modules/onlinechefgroep-herdr/herdr",
        )));
        assert!(!is_npm_managed_exe_path(Path::new(
            "/usr/local/bin/herdr",
        )));
    }

""" + test_marker
    text = replace_once(text, test_marker, npm_tests, "add npm Rust tests")
    write("src/update.rs", text)


def patch_remote_mise_path() -> None:
    text = read("src/remote/unix.rs")
    old = "github-ogulcancelik-herdr"
    count = text.count(old)
    if count == 0:
        raise RuntimeError("src/remote/unix.rs: missing upstream mise path")
    text = text.replace(old, "github-OnlineChefGroep-herdr")
    write("src/remote/unix.rs", text)


def main() -> None:
    patch_latest_json()
    patch_release_workflow()
    patch_portable_workflow()
    patch_update_rs()
    patch_remote_mise_path()
    print("checkout patch applied")


if __name__ == "__main__":
    main()
