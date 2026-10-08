from __future__ import annotations

import json
import unittest

from scripts.changelog import (
    ChangelogError,
    PROMOTED_ASSET_TARGETS,
    PROMOTED_EXPECTED_ASSET_NAMES,
    archived_releases_from_current_manifest,
    build_latest_json,
    manifest_from_release_payload,
    parse_sha256sums,
    promoted_release_assets,
)


class ReleaseManifestHardeningTests(unittest.TestCase):
    def checksums(self) -> dict[str, str]:
        return {
            asset_name: f"{index:064x}"
            for index, asset_name in enumerate(
                PROMOTED_EXPECTED_ASSET_NAMES.values(), start=1
            )
        }

    def release_payload(self, checksums: dict[str, str]) -> dict[str, object]:
        assets = []
        for asset_name in PROMOTED_EXPECTED_ASSET_NAMES.values():
            assets.append(
                {
                    "name": asset_name,
                    "url": f"https://example.invalid/{asset_name}",
                    "digest": f"sha256:{checksums[asset_name]}",
                }
            )
        assets.append(
            {
                "name": "SHA256SUMS",
                "url": "https://example.invalid/SHA256SUMS",
            }
        )
        return {
            "tagName": "v1.2.3",
            "isDraft": False,
            "isPrerelease": False,
            "body": "### Fixed\n- Hardened release promotion.\n",
            "assets": assets,
        }

    def test_parse_sha256sums_accepts_all_release_assets(self) -> None:
        checksums = self.checksums()
        body = "\n".join(
            f"{checksum}  {asset_name}"
            for asset_name, checksum in checksums.items()
        )

        self.assertEqual(parse_sha256sums(body), checksums)

    def test_parse_sha256sums_rejects_paths_and_duplicates(self) -> None:
        checksum = "1" * 64
        with self.assertRaisesRegex(ChangelogError, "invalid asset name"):
            parse_sha256sums(f"{checksum}  ../herdr-linux-x86_64\n")
        with self.assertRaisesRegex(ChangelogError, "duplicate checksum"):
            parse_sha256sums(
                f"{checksum}  herdr-linux-x86_64\n"
                f"{'2' * 64}  herdr-linux-x86_64\n"
            )

    def test_promoted_release_assets_requires_complete_matrix(self) -> None:
        checksums = self.checksums()
        del checksums["herdr-macos-aarch64"]

        with self.assertRaisesRegex(ChangelogError, "missing herdr-macos-aarch64"):
            promoted_release_assets("1.2.3", checksums)

    def test_release_payload_builds_four_checksummed_entries(self) -> None:
        checksums = self.checksums()
        manifest = manifest_from_release_payload(
            self.release_payload(checksums),
            "1.2.3",
            protocol=42,
            checksums=checksums,
        )

        self.assertEqual(tuple(manifest["assets"]), PROMOTED_ASSET_TARGETS)
        for target, asset_name in PROMOTED_EXPECTED_ASSET_NAMES.items():
            self.assertEqual(
                manifest["assets"][target],
                {
                    "url": f"https://example.invalid/{asset_name}",
                    "sha256": checksums[asset_name],
                },
            )

    def test_release_payload_rejects_github_digest_mismatch(self) -> None:
        checksums = self.checksums()
        payload = self.release_payload(checksums)
        payload["assets"][0]["digest"] = f"sha256:{'f' * 64}"

        with self.assertRaisesRegex(ChangelogError, "does not match SHA256SUMS"):
            manifest_from_release_payload(
                payload,
                "1.2.3",
                protocol=42,
                checksums=checksums,
            )

    def test_legacy_archive_preserves_all_plain_string_targets(self) -> None:
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
                "### Fixed\n- Hardened release promotion.",
                promoted,
                protocol=42,
                releases={
                    "1.2.2": {
                        "notes": "### Fixed\n- Legacy release.",
                        "assets": legacy_assets,
                    }
                },
            )
        )
        self.assertEqual(manifest["releases"]["1.2.2"]["assets"], legacy_assets)

    def test_latest_manifest_mirrors_promoted_assets_and_preserves_legacy(self) -> None:
        checksums = self.checksums()
        promoted = promoted_release_assets("1.2.3", checksums)
        legacy = {
            "1.2.2": {
                "notes": "### Fixed\n- Legacy release.",
                "assets": {
                    "linux-x86_64": "https://example.invalid/legacy-linux-x86_64"
                },
            }
        }

        manifest = json.loads(
            build_latest_json(
                "1.2.3",
                "### Fixed\n- Hardened release promotion.",
                promoted,
                protocol=42,
                releases=legacy,
            )
        )

        self.assertEqual(manifest["assets"], promoted)
        self.assertEqual(manifest["releases"]["1.2.3"]["assets"], promoted)
        self.assertEqual(
            manifest["releases"]["1.2.2"]["assets"],
            legacy["1.2.2"]["assets"],
        )
        self.assertEqual(
            archived_releases_from_current_manifest(manifest)["1.2.3"]["assets"],
            promoted,
        )


if __name__ == "__main__":
    unittest.main()
