import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import scripts.dev as dev
import scripts.preview as preview
from scripts.product_config import PRODUCT_GITHUB_REPO


class DevManifestTests(unittest.TestCase):
    def test_build_manifest_uses_dev_channel(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "dev.json"
            content = preview.build_manifest(
                output=output,
                repo=PRODUCT_GITHUB_REPO,
                tag="dev-2026-06-02-abcdef123456",
                build_id="2026-06-02-abcdef123456",
                commit="abcdef1234567890",
                built_at="2026-06-02T03:00:00Z",
                base_version="0.7.6",
                protocol=17,
                notes="Dev notes\n",
                shas={"linux-x86_64": "deadbeef"},
                retain=30,
                channel="dev",
            )
            data = json.loads(content)
            self.assertEqual(data["channel"], "dev")
            self.assertEqual(data["build_id"], "2026-06-02-abcdef123456")
            self.assertEqual(data["assets"]["linux-x86_64"]["sha256"], "deadbeef")
            self.assertEqual(
                data["assets"]["linux-x86_64"]["url"],
                f"https://github.com/{PRODUCT_GITHUB_REPO}/releases/download/"
                "dev-2026-06-02-abcdef123456/herdr-linux-x86_64",
            )
            self.assertIn("2026-06-02-abcdef123456", data["builds"])

    def test_notes_use_dev_label_and_main_branch(self):
        with mock.patch.object(
            preview, "commit_subjects", return_value=["feat: add dev channel"]
        ):
            notes = preview.build_notes(
                "v0.7.6",
                "abcdef1234567890",
                "2026-06-02-abcdef123456",
                "0.7.6",
                PRODUCT_GITHUB_REPO,
                channel_label=dev.CHANNEL_LABEL,
                branch=dev.DEFAULT_BRANCH,
            )
        self.assertIn("Dev build 2026-06-02-abcdef123456", notes)
        self.assertIn("on `main`", notes)
        self.assertIn("### Added", notes)

    def test_dev_manifest_hidden_subject(self):
        self.assertTrue(preview.hidden_subject("docs: update dev manifest"))

    def test_dev_defaults(self):
        self.assertEqual(dev.CHANNEL, "dev")
        self.assertEqual(dev.DEFAULT_MANIFEST, "website/dev.json")


if __name__ == "__main__":
    unittest.main()
