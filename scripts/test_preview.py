import json
import os
import subprocess
import tempfile
import unittest
from unittest import mock
from pathlib import Path

import scripts.conventional_commits as conventional_commits
import scripts.preview as preview
from scripts.product_config import PRODUCT_GITHUB_REPO


class PreviewNotesTests(unittest.TestCase):
    def test_humanize_groups_conventional_subjects(self):
        self.assertEqual(
            preview.humanize_subject("feat(update): add preview channel"),
            ("Added", "Add preview channel"),
        )
        self.assertEqual(
            preview.humanize_subject("fix: handle preview manifest"),
            ("Fixed", "Handle preview manifest"),
        )
        self.assertEqual(
            preview.humanize_subject("not conventional"),
            ("Other", "Not conventional"),
        )

    def test_build_manifest_archives_current_assets(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "preview.json"
            notes = "Preview notes\n"
            content = preview.build_manifest(
                output=output,
                repo=PRODUCT_GITHUB_REPO,
                channel="preview",
                tag="preview-2026-06-02-abcdef123456",
                build_id="2026-06-02-abcdef123456",
                commit="abcdef1234567890",
                built_at="2026-06-02T03:00:00Z",
                base_version="0.6.6",
                protocol=12,
                notes=notes,
                shas={"linux-x86_64": "d" * 64},
                retain=30,
            )
            data = json.loads(content)
            self.assertEqual(data["channel"], "preview")
            self.assertEqual(data["build_id"], "2026-06-02-abcdef123456")
            self.assertEqual(
                data["assets"]["linux-x86_64"]["sha256"],
                "d" * 64,
            )
            self.assertEqual(
                data["assets"]["linux-x86_64"]["url"],
                f"https://github.com/{PRODUCT_GITHUB_REPO}/releases/download/preview-2026-06-02-abcdef123456/herdr-linux-x86_64",
            )
            self.assertEqual(set(data["assets"]), {"linux-x86_64"})
            self.assertIn("2026-06-02-abcdef123456", data["builds"])

    def test_build_manifest_rejects_missing_or_invalid_checksums(self):
        urls = preview.default_asset_urls(
            PRODUCT_GITHUB_REPO,
            "preview-2026-06-02-abcdef123456",
        )
        with self.assertRaisesRegex(SystemExit, "missing linux-x86_64"):
            preview.asset_objects(urls, {})
        with self.assertRaisesRegex(SystemExit, "64 hexadecimal"):
            preview.asset_objects(urls, {"linux-x86_64": "deadbeef"})

    def test_preview_defaults_to_main(self):
        with mock.patch.object(preview, "commit_subjects", return_value=[]):
            notes = preview.build_notes(
                "previous",
                "abcdef1234567890",
                "2026-06-02-abcdef123456",
                "0.7.6",
                PRODUCT_GITHUB_REPO,
            )
        self.assertIn("on `main`", notes)

    def test_build_manifest_accepts_dev_channel(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "dev.json"
            content = preview.build_manifest(
                output=output,
                repo=PRODUCT_GITHUB_REPO,
                channel="dev",
                tag="dev-2026-06-02-abcdef123456",
                build_id="2026-06-02-abcdef123456",
                commit="abcdef1234567890",
                built_at="2026-06-02T03:00:00Z",
                base_version="0.6.6",
                protocol=12,
                notes="Dev notes\n",
                shas={"linux-x86_64": "d" * 64},
                retain=20,
            )
            data = json.loads(content)
            self.assertEqual(data["channel"], "dev")
            self.assertEqual(
                data["assets"]["linux-x86_64"]["url"],
                f"https://github.com/{PRODUCT_GITHUB_REPO}/releases/download/dev-2026-06-02-abcdef123456/herdr-linux-x86_64",
            )

    def test_hidden_subjects_include_preview_manifest_commits(self):
        self.assertTrue(preview.hidden_subject("docs: update preview manifest"))
        self.assertTrue(preview.hidden_subject("docs: update dev manifest"))
        self.assertTrue(preview.hidden_subject("docs: update website manifest"))
        self.assertFalse(preview.hidden_subject("release: v0.7.0"))
        self.assertFalse(preview.hidden_subject("fix: repair preview manifest"))

    def test_latest_publishable_commit_keeps_release_commits(self):
        output = "\n".join(
            [
                "manifest\x00docs: update website manifest for v0.7.0",
                "release\x00release: v0.7.0",
                "feature\x00feat: add plugin v1 system",
            ]
        )
        with mock.patch.object(preview, "run_git", return_value=output):
            self.assertEqual(preview.latest_publishable_commit("origin/master"), "release")

    def test_preview_range_base_advances_to_stable_tag(self):
        with (
            mock.patch.object(preview, "latest_stable_tag", return_value="v0.7.0"),
            mock.patch.object(preview, "git_is_ancestor", return_value=True),
        ):
            self.assertEqual(
                preview.preview_range_base("previous-preview", "release"),
                "v0.7.0",
            )

    def test_preview_range_base_keeps_previous_preview_for_unreleased_work(self):
        def is_ancestor(ancestor: str, descendant: str) -> bool:
            # previous-preview is a real ancestor of new-feature, but v0.7.0 was
            # cut from a different line, so it cannot serve as the range base.
            return (ancestor, descendant) in {
                ("previous-preview", "new-feature"),
                ("v0.7.0", "new-feature"),
            }

        with (
            mock.patch.object(preview, "latest_stable_tag", return_value="v0.7.0"),
            mock.patch.object(preview, "git_is_ancestor", side_effect=is_ancestor),
        ):
            self.assertEqual(
                preview.preview_range_base("previous-preview", "new-feature"),
                "previous-preview",
            )

    def test_preview_range_base_falls_back_to_stable_when_previous_unreachable(self):
        with (
            mock.patch.object(preview, "latest_stable_tag", return_value="v0.8.0"),
            mock.patch.object(preview, "git_is_ancestor", return_value=False),
        ):
            self.assertEqual(
                preview.preview_range_base("rewritten-commit", "new-feature"),
                "v0.8.0",
            )

    def test_post_stable_history_selects_release_and_bases_range_on_stable_tag(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)

            def git(*args: str) -> str:
                return subprocess.check_output(
                    ["git", *args],
                    cwd=repo,
                    text=True,
                    stderr=subprocess.DEVNULL,
                ).strip()

            git("init")
            git("config", "user.email", "test@example.com")
            git("config", "user.name", "Test User")

            marker = repo / "marker.txt"
            marker.write_text("preview\n", encoding="utf-8")
            git("add", "marker.txt")
            git("commit", "-m", "feat: previous preview")
            previous_preview = git("rev-parse", "HEAD")

            marker.write_text("release\n", encoding="utf-8")
            git("commit", "-am", "release: v0.7.0")
            release = git("rev-parse", "HEAD")
            git("tag", "v0.7.0")

            marker.write_text("manifest\n", encoding="utf-8")
            git("commit", "-am", "docs: update website manifest for v0.7.0")

            original_cwd = os.getcwd()
            try:
                os.chdir(repo)
                self.assertEqual(preview.latest_publishable_commit("HEAD"), release)
                self.assertEqual(
                    preview.preview_range_base(previous_preview, release),
                    "v0.7.0",
                )
            finally:
                os.chdir(original_cwd)

    def test_preview_docs_rewrite_links_to_preview_namespace(self):
        source = """---
title: Install Herdr
---

[Install](/docs/install/)
file: ../../../public/assets/logo.svg
"""
        output = subprocess.check_output(
            ["node", "website/scripts/prepare-docs.mjs", "--rewrite-preview-doc-fixture"],
            input=source,
            text=True,
        )
        self.assertIn("[Install](/docs/preview/install/)", output)
        self.assertIn("file: ../../../../public/assets/logo.svg", output)
        self.assertIn("Preview docs describe unreleased preview builds", output)

    def test_preview_docs_rewrite_localized_hero_paths(self):
        source = """---
title: Herdr docs
---

file: ../../../../public/assets/logo.svg
"""
        output = subprocess.check_output(
            ["node", "website/scripts/prepare-docs.mjs", "--rewrite-preview-doc-fixture"],
            input=source,
            text=True,
        )
        self.assertIn("file: ../../../../../public/assets/logo.svg", output)

    def test_build_manifest_marks_channel_enabled(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "preview.json"
            content = preview.build_manifest(
                output=output,
                repo=PRODUCT_GITHUB_REPO,
                channel="preview",
                tag="preview-2026-06-02-abcdef123456",
                build_id="2026-06-02-abcdef123456",
                commit="abcdef1234567890",
                built_at="2026-06-02T03:00:00Z",
                base_version="0.6.6",
                protocol=12,
                notes="Preview notes\n",
                shas={"linux-x86_64": "d" * 64},
                retain=30,
            )
            data = json.loads(content)
            self.assertTrue(data["enabled"])

    def test_select_commit_blocks_disabled_channel(self):
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "preview.json"
            manifest.write_text(json.dumps({"enabled": False, "commit": ""}), encoding="utf-8")
            ns = mock.Mock(manifest=str(manifest), ref="origin/main")
            with self.assertRaisesRegex(SystemExit, "disabled"):
                preview.cmd_select_commit(ns)

    def test_previous_preview_commit_is_none_when_channel_disabled(self):
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "preview.json"
            manifest.write_text(json.dumps({"enabled": False, "commit": ""}), encoding="utf-8")
            self.assertIsNone(preview.previous_preview_commit(manifest))
            self.assertFalse(preview.manifest_enabled(manifest))


class ConventionalCommitTests(unittest.TestCase):
    def test_valid_subjects_allow_scopes_and_bang(self):
        self.assertTrue(conventional_commits.valid_subject("fix(update): handle preview"))
        self.assertTrue(conventional_commits.valid_subject("feat!: change config"))
        self.assertFalse(conventional_commits.valid_subject("update preview channel"))

    def test_commit_message_subject_skips_comments(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "COMMIT_EDITMSG"
            path.write_text(
                "\n# Please enter the commit message\n\nfix(update): switch channel\n",
                encoding="utf-8",
            )
            self.assertEqual(
                conventional_commits.commit_message_subject(path),
                "fix(update): switch channel",
            )


if __name__ == "__main__":
    unittest.main()
