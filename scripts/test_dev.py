import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import scripts.dev as dev
import scripts.preview as preview
from scripts.product_config import PRODUCT_GITHUB_REPO

REPO_ROOT = Path(__file__).resolve().parent.parent
DEV_WORKFLOW_PATH = REPO_ROOT / ".github/workflows/dev.yml"


def workflow_step(workflow: str, name: str) -> str:
    marker = f"      - name: {name}\n"
    start = workflow.index(marker)
    end = workflow.find("\n      - name: ", start + len(marker))
    return workflow[start:] if end == -1 else workflow[start:end]


class DevManifestTests(unittest.TestCase):
    def test_build_manifest_uses_dev_channel(self):
        checksum = "d" * 64
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
                shas={"linux-x86_64": checksum},
                retain=30,
                channel="dev",
            )
            data = json.loads(content)
            self.assertEqual(data["channel"], "dev")
            self.assertEqual(data["build_id"], "2026-06-02-abcdef123456")
            self.assertEqual(data["assets"]["linux-x86_64"]["sha256"], checksum)
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

    def test_publish_checkout_uses_release_deploy_key(self):
        workflow = DEV_WORKFLOW_PATH.read_text(encoding="utf-8")
        publish_job = workflow.split("\n  publish:\n", maxsplit=1)[1]
        checkout = publish_job.split("      - name: Download all artifacts", maxsplit=1)[0]

        self.assertIn("ref: main", checkout)
        self.assertIn("ssh-key: ${{ secrets.RELEASE_DEPLOY_KEY }}", checkout)
        self.assertNotIn("persist-credentials: false", checkout)

    def test_manifest_push_never_rewires_origin_to_github_token(self):
        workflow = DEV_WORKFLOW_PATH.read_text(encoding="utf-8")
        commit_step = workflow_step(workflow, "Commit dev manifest")

        self.assertIn("git push origin HEAD:main", commit_step)
        self.assertNotIn("GH_TOKEN", commit_step)
        self.assertNotIn("x-access-token", commit_step)
        self.assertNotIn("git remote set-url", commit_step)


if __name__ == "__main__":
    unittest.main()
