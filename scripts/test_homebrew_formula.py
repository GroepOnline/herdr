from __future__ import annotations

import unittest

from scripts.homebrew_formula import (
    parse_sha256sums,
    render,
    verified_release_assets,
)
from scripts.product_config import RELEASE_TARGETS


class HomebrewFormulaTests(unittest.TestCase):
    def checksums(self) -> dict[str, str]:
        return {
            target["asset"]: f"{index:064x}"
            for index, target in enumerate(RELEASE_TARGETS, start=1)
        }

    def release(self, checksums: dict[str, str]) -> dict[str, object]:
        assets = [
            {
                "name": target["asset"],
                "browser_download_url": (
                    "https://github.com/OnlineChefGroep/herdr/releases/download/"
                    f"v1.2.3/{target['asset']}"
                ),
                "digest": f"sha256:{checksums[target['asset']]}",
            }
            for target in RELEASE_TARGETS
        ]
        assets.append(
            {
                "name": "SHA256SUMS",
                "browser_download_url": (
                    "https://github.com/OnlineChefGroep/herdr/releases/download/"
                    "v1.2.3/SHA256SUMS"
                ),
            }
        )
        return {"assets": assets}

    def test_parse_sha256sums(self) -> None:
        checksums = self.checksums()
        text = "\n".join(
            f"{checksum}  {name}" for name, checksum in checksums.items()
        )
        self.assertEqual(parse_sha256sums(text), checksums)

    def test_verified_assets_reject_digest_mismatch(self) -> None:
        checksums = self.checksums()
        release = self.release(checksums)
        release["assets"][0]["digest"] = f"sha256:{'f' * 64}"
        with self.assertRaisesRegex(ValueError, "does not match SHA256SUMS"):
            verified_release_assets(release, checksums)

    def test_render_contains_all_platform_architecture_stanzas(self) -> None:
        checksums = self.checksums()
        verified = verified_release_assets(self.release(checksums), checksums)
        formula = render("1.2.3", verified)

        self.assertIn("on_linux do", formula)
        self.assertIn("on_macos do", formula)
        self.assertEqual(formula.count("on_intel do"), 2)
        self.assertEqual(formula.count("on_arm do"), 2)
        for target in RELEASE_TARGETS:
            self.assertIn(target["asset"], formula)
            self.assertIn(checksums[target["asset"]], formula)


if __name__ == "__main__":
    unittest.main()
