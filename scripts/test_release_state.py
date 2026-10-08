"""Regression tests for the actual release-state files touched by a version bump.

These tests exercise the real repository content (not synthetic fixtures) the
same way ``just release-docs-check`` and ``just release-prepare`` do: the
``Cargo.toml`` package version must match the locked ``herdr`` entry in
``Cargo.lock``, the changelog must contain a well-formed, dated section with
categorized bullets for the current version, and README.md's in-page nav
anchors must resolve to real headings.

They also guard the bulk find/replace made across versioned documentation
snapshots (renaming the GitHub owner referenced by ``owner/herdr`` style
links): each touched snapshot file must reference exactly one owner for the
``herdr`` repository, never a mix of an old and a new owner left behind by a
partial rename.
"""

from __future__ import annotations

import re
import unittest
from datetime import date
from pathlib import Path

from scripts.changelog import (
    ChangelogError,
    extract_section_body,
    normalize_version,
)
from scripts.ci_quality import check_release_note_bullets
from scripts.product_config import cargo_version

ROOT = Path(__file__).resolve().parents[1]
CHANGELOG_PATH = ROOT / "CHANGELOG.md"
CARGO_LOCK_PATH = ROOT / "Cargo.lock"
README_PATH = ROOT / "README.md"

OWNER_RENAME_TOUCHED_DOCS = [
    "docs/versions/0.7.5/website/src/content/docs/agent-skill.mdx",
    "docs/versions/0.7.5/website/src/content/docs/cli-reference.mdx",
    "docs/versions/0.7.5/website/src/content/docs/install.mdx",
    "docs/versions/0.7.5/website/src/content/docs/plugins.mdx",
    "docs/versions/0.7.5/website/src/content/docs/socket-api.mdx",
    "docs/versions/0.7.6/website/src/content/docs/agent-skill.mdx",
    "docs/versions/0.7.6/website/src/content/docs/cli-reference.mdx",
    "docs/versions/0.7.6/website/src/content/docs/install.mdx",
    "docs/versions/0.7.6/website/src/content/docs/plugins.mdx",
    "docs/versions/0.7.6/website/src/content/docs/socket-api.mdx",
    "docs/versions/0.7.7/website/src/content/docs/agent-skill.mdx",
    "docs/versions/0.7.7/website/src/content/docs/cli-reference.mdx",
    "docs/versions/0.7.7/website/src/content/docs/fleet-ops.mdx",
    "docs/versions/0.7.7/website/src/content/docs/install.mdx",
    "docs/versions/0.7.7/website/src/content/docs/moshi.mdx",
    "docs/versions/0.7.7/website/src/content/docs/plugins.mdx",
    "docs/versions/0.7.7/website/src/content/docs/socket-api.mdx",
    "docs/versions/0.7.7/website/src/content/docs/troubleshooting.mdx",
]

URL_OWNER_RE = re.compile(r"github\.com/([A-Za-z][\w.-]*)/herdr(?:-[\w-]+)*\b")
BARE_OWNER_RE = re.compile(r"(?<![\w./~])([A-Za-z][\w.-]*)/herdr(?:-[\w-]+)*\b")
UPSTREAM_CITATION_OWNERS = frozenset({"ogulcancelik"})

CHANGELOG_HEADING_RE = re.compile(
    r"^##\s+\[(?P<version>[^\]]+)\]\s+-\s+(?P<date>\d{4}-\d{2}-\d{2})\s*$",
    re.MULTILINE,
)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def owners_referenced(text: str) -> set[str]:
    owners = {match.group(1) for match in URL_OWNER_RE.finditer(text)}
    owners |= {match.group(1) for match in BARE_OWNER_RE.finditer(text)}
    return owners - UPSTREAM_CITATION_OWNERS


def slugify_heading(title: str) -> str:
    slug = title.strip().lower()
    slug = re.sub(r"[^a-z0-9\s-]", "", slug)
    slug = re.sub(r"\s+", "-", slug).strip("-")
    return slug


class ChangelogVersionConsistencyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.version = normalize_version(cargo_version(ROOT))
        self.changelog_text = read(CHANGELOG_PATH)

    def test_cargo_version_looks_like_a_release(self) -> None:
        self.assertRegex(self.version, r"^\d+\.\d+\.\d+$")

    def test_changelog_has_a_dated_section_for_the_current_version(self) -> None:
        headings = {
            match.group("version"): match.group("date")
            for match in CHANGELOG_HEADING_RE.finditer(self.changelog_text)
        }
        self.assertIn(
            self.version,
            headings,
            f"{CHANGELOG_PATH} is missing a dated '## [{self.version}] - YYYY-MM-DD' heading",
        )
        date.fromisoformat(headings[self.version])

    def test_changelog_current_version_heading_is_not_duplicated(self) -> None:
        occurrences = [
            match
            for match in CHANGELOG_HEADING_RE.finditer(self.changelog_text)
            if match.group("version") == self.version
        ]
        self.assertEqual(len(occurrences), 1)

    def test_changelog_section_body_is_extractable_and_non_empty(self) -> None:
        body = extract_section_body(self.changelog_text, self.version)
        self.assertTrue(body.strip())

    def test_changelog_current_version_has_categorized_bullets(self) -> None:
        check_release_note_bullets(self.changelog_text, self.version)

    def test_extracting_an_unreleased_version_number_fails(self) -> None:
        with self.assertRaises(ChangelogError):
            extract_section_body(self.changelog_text, "99.99.99")


class CargoVersionConsistencyTests(unittest.TestCase):
    def test_cargo_lock_herdr_entry_matches_cargo_toml_version(self) -> None:
        toml_version = cargo_version(ROOT)
        lock_text = read(CARGO_LOCK_PATH)
        match = re.search(r'name = "herdr"\nversion = "([^"]+)"', lock_text)
        self.assertIsNotNone(match, "Cargo.lock is missing a herdr package entry")
        self.assertEqual(
            match.group(1),
            toml_version,
            "Cargo.lock's herdr package version has drifted from Cargo.toml",
        )

    def test_cargo_lock_has_exactly_one_herdr_entry(self) -> None:
        lock_text = read(CARGO_LOCK_PATH)
        occurrences = len(re.findall(r'name = "herdr"\n', lock_text))
        self.assertEqual(occurrences, 1)


class ReadmeAnchorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.text = read(README_PATH)
        self.headings = [
            match.group(1).strip()
            for match in re.finditer(r"^##\s+(.+?)\s*$", self.text, re.MULTILINE)
        ]
        self.slugs = {slugify_heading(title) for title in self.headings}

    def test_local_anchor_links_resolve_to_headings(self) -> None:
        anchors = set(re.findall(r'href="#([a-z0-9-]+)"', self.text))
        anchors.update(re.findall(r'\]\(#([a-z0-9-]+)\)', self.text))
        for anchor in sorted(anchors):
            with self.subTest(anchor=anchor):
                self.assertIn(anchor, self.slugs)

    def test_expected_top_level_sections_are_present(self) -> None:
        for expected in ("install", "quick-start", "supported-agents", "sponsor"):
            self.assertIn(expected, self.slugs)


class VersionedDocsOwnerRenameConsistencyTests(unittest.TestCase):
    def test_rename_touched_docs_reference_a_single_owner(self) -> None:
        for relative_path in OWNER_RENAME_TOUCHED_DOCS:
            path = ROOT / relative_path
            with self.subTest(file=relative_path):
                text = read(path)
                owners = owners_referenced(text)
                self.assertLessEqual(
                    len(owners),
                    1,
                    f"{relative_path} mixes multiple owners for herdr links: {sorted(owners)}",
                )

    def test_owner_rename_did_not_leave_a_stale_reference(self) -> None:
        owners_by_file = {}
        for relative_path in OWNER_RENAME_TOUCHED_DOCS:
            text = read(ROOT / relative_path)
            owners = owners_referenced(text)
            if owners:
                owners_by_file[relative_path] = owners

        self.assertTrue(owners_by_file)
        distinct_owners = {owner for owners in owners_by_file.values() for owner in owners}
        self.assertEqual(
            len(distinct_owners),
            1,
            f"rewritten docs disagree on the herdr repo owner: {owners_by_file}",
        )


if __name__ == "__main__":
    unittest.main()
