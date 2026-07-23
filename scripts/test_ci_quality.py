from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from scripts.ci_quality import (
    QualityError,
    check_release_metadata,
    detect_autofix,
    needs_rustfmt,
    sync_release_metadata,
)


class CiQualityTests(unittest.TestCase):
    def write_fixture(self, root: Path, cargo_version: str, npm_version: str) -> None:
        (root / "npm").mkdir()
        (root / "src/bin").mkdir(parents=True)
        (root / "src").mkdir(parents=True, exist_ok=True)
        (root / "scripts").mkdir()

        (root / "src/lib.rs").write_text("\n", encoding="utf-8")

        (root / "Cargo.toml").write_text(
            f'[package]\nname = "herdr"\nversion = "{cargo_version}"\n',
            encoding="utf-8",
        )
        (root / "npm/package.json").write_text(
            json.dumps(
                {
                    "name": "onlinechefgroep-herdr",
                    "version": npm_version,
                    "os": ["linux"],
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        (root / "npm/install.js").write_text(
            f'const VERSION = "{npm_version}";\n',
            encoding="utf-8",
        )
        (root / "CHANGELOG.md").write_text(
            f"""# Changelog

## Unreleased

## [{cargo_version}] - 2026-07-23

### Added
- Added the release note for this version.

## [0.1.0] - 2026-07-22

### Fixed

- Fixed an older release.
""",
            encoding="utf-8",
        )
        (root / "src/bin/herdr-gateway.rs").write_text(
            'fn version() -> &' + 'static str { env!("CARGO_PKG_VERSION") }\n',
            encoding="utf-8",
        )
        (root / "scripts/changelog.py").write_text(
            'DEFAULT_RELEASE_REPO = "OnlineChefGroep/herdr"\n',
            encoding="utf-8",
        )
        (root / "npm/README.md").write_text("# npm package\n", encoding="utf-8")

    def test_check_release_metadata_accepts_matching_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_fixture(root, "1.2.3", "1.2.3")

            check_release_metadata(root)

    def test_check_release_metadata_uses_matching_changelog_section(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_fixture(root, "1.2.3", "1.2.3")
            (root / "CHANGELOG.md").write_text(
                """# Changelog

## Unreleased

## [1.2.3] - 2026-07-23

Release notes without a categorized bullet.

## [1.2.2] - 2026-07-22

### Added

- Added an older release note.
""",
                encoding="utf-8",
            )

            with self.assertRaises(QualityError):
                check_release_metadata(root)

    @patch("scripts.ci_quality.needs_rustfmt", return_value=False)
    def test_sync_release_metadata_updates_npm_files(self, _mock_needs_rustfmt) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_fixture(root, "1.2.3", "1.2.2")

            self.assertEqual(
                detect_autofix(root),
                {"needs_fmt": False, "needs_metadata_sync": True},
            )
            self.assertTrue(sync_release_metadata(root))

            package = json.loads((root / "npm/package.json").read_text(encoding="utf-8"))
            installer = (root / "npm/install.js").read_text(encoding="utf-8")
            self.assertEqual(package["version"], "1.2.3")
            self.assertIn('const VERSION = "1.2.3";', installer)
            self.assertEqual(
                detect_autofix(root),
                {"needs_fmt": False, "needs_metadata_sync": False},
            )
            check_release_metadata(root)

    @patch("scripts.ci_quality.shutil.which", return_value=None)
    def test_needs_rustfmt_false_when_cargo_missing(self, _mock_which) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            self.assertFalse(needs_rustfmt(Path(tmp)))

    @patch("scripts.ci_quality.subprocess.run")
    @patch("scripts.ci_quality.shutil.which", return_value="/usr/bin/cargo")
    def test_needs_rustfmt_false_on_clean_check(self, _mock_which, mock_run) -> None:
        mock_run.return_value = Mock(returncode=0, stdout="", stderr="")
        with tempfile.TemporaryDirectory() as tmp:
            self.assertFalse(needs_rustfmt(Path(tmp)))
        args = mock_run.call_args.args[0]
        self.assertEqual(Path(args[0]).name, "cargo")
        self.assertTrue(Path(args[0]).is_absolute())
        self.assertEqual(args[1:], ["fmt", "--all", "--", "--check"])

    @patch("scripts.ci_quality.subprocess.run")
    @patch("scripts.ci_quality.shutil.which", return_value="/usr/bin/cargo")
    def test_needs_rustfmt_true_on_diff_output(self, _mock_which, mock_run) -> None:
        mock_run.return_value = Mock(
            returncode=1,
            stdout="Diff in src/main.rs:\n",
            stderr="",
        )
        with tempfile.TemporaryDirectory() as tmp:
            self.assertTrue(needs_rustfmt(Path(tmp)))

    @patch("scripts.ci_quality.needs_rustfmt", return_value=True)
    def test_detect_autofix_reports_fmt_need(self, _mock_needs_rustfmt) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_fixture(root, "1.2.3", "1.2.3")
            self.assertEqual(
                detect_autofix(root),
                {"needs_fmt": True, "needs_metadata_sync": False},
            )


if __name__ == "__main__":
    unittest.main()
