from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import ci_changed_paths


class CiChangedPathsTests(unittest.TestCase):
    def _run_main(self, files: list[str]) -> str:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "out"
            with (
                patch.object(ci_changed_paths, "changed_files", return_value=files),
                patch.dict(os.environ, {"GITHUB_OUTPUT": str(out)}, clear=False),
            ):
                self.assertEqual(ci_changed_paths.main(), 0)
            return out.read_text(encoding="utf-8")

    def test_docs_only_classification(self) -> None:
        text = self._run_main(["docs/next/CHANGELOG.md", "README.md"])
        self.assertIn("docs_only=true", text)
        self.assertIn("rust=false", text)
        self.assertIn("platform_heavy=false", text)

    def test_website_only_skips_rust_lanes(self) -> None:
        text = self._run_main(["website/css/style.css", "website/astro.config.mjs"])
        self.assertIn("docs_only=true", text)
        self.assertIn("rust=false", text)
        self.assertIn("platform_heavy=false", text)
        self.assertIn("release_meta=false", text)

    def test_design_system_json_is_docs_only(self) -> None:
        text = self._run_main([".github/design-system.json"])
        self.assertIn("docs_only=true", text)
        self.assertIn("rust=false", text)

    def test_changelog_with_docs_is_docs_only_without_rust(self) -> None:
        text = self._run_main(["CHANGELOG.md", "docs/guide.md"])
        self.assertIn("docs_only=true", text)
        self.assertIn("rust=false", text)
        self.assertIn("release_meta=true", text)

    def test_installer_runs_maintenance_and_release_metadata(self) -> None:
        text = self._run_main(["website/install.sh"])
        self.assertIn("maintenance=true", text)
        self.assertIn("release_meta=true", text)
        self.assertIn("docs_only=false", text)
        self.assertIn("rust=false", text)

    def test_platform_heavy_classification(self) -> None:
        text = self._run_main(["src/platform/windows.rs", "Cargo.lock"])
        self.assertIn("rust=true", text)
        self.assertIn("platform_heavy=true", text)
        self.assertIn("docs_only=false", text)

    def test_workflow_changes_are_not_docs_only(self) -> None:
        text = self._run_main([".github/workflows/ci.yml"])
        self.assertIn("docs_only=false", text)
        self.assertIn("platform_heavy=true", text)
        self.assertIn("rust=false", text)

    def test_empty_diff_fail_open_runs_rust(self) -> None:
        outputs = ci_changed_paths.classify([])
        self.assertTrue(outputs["rust"])
        self.assertFalse(outputs["docs_only"])


if __name__ == "__main__":
    unittest.main()
