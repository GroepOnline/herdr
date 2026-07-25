from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import ci_changed_paths


class CiChangedPathsTests(unittest.TestCase):
    def test_docs_only_classification(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "out"
            with (
                patch.object(
                    ci_changed_paths,
                    "changed_files",
                    return_value=["docs/next/CHANGELOG.md", "README.md"],
                ),
                patch.dict(os.environ, {"GITHUB_OUTPUT": str(out)}, clear=False),
            ):
                self.assertEqual(ci_changed_paths.main(), 0)
            text = out.read_text(encoding="utf-8")
            self.assertIn("docs_only=true", text)
            self.assertIn("rust=false", text)
            self.assertIn("platform_heavy=false", text)

    def test_platform_heavy_classification(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "out"
            with (
                patch.object(
                    ci_changed_paths,
                    "changed_files",
                    return_value=["src/platform/windows.rs", "Cargo.lock"],
                ),
                patch.dict(os.environ, {"GITHUB_OUTPUT": str(out)}, clear=False),
            ):
                self.assertEqual(ci_changed_paths.main(), 0)
            text = out.read_text(encoding="utf-8")
            self.assertIn("rust=true", text)
            self.assertIn("platform_heavy=true", text)
            self.assertIn("docs_only=false", text)


if __name__ == "__main__":
    unittest.main()
