#!/usr/bin/env python3
"""Tests for the upstream sync ledger tooling."""

from __future__ import annotations

import unittest

from scripts.upstream_sync_ledger import (
    OverlayRow,
    classify_paths,
    command_check,
    fingerprint_backlog,
    glob_matches,
    normalize_subject,
    parse_overlay,
    uncovered_overlay_paths,
    validate_ledger,
    validate_overlay,
    validate_pin,
    validate_scope,
)

HEADER = "path_glob\tconcern\towner\trationale\ttests\tremoval_condition\n"


def overlay_row(glob: str = "website/**", concern: str = "site") -> OverlayRow:
    return OverlayRow(glob, concern, "maintainer", "why", "just website-build", "never")


class NormalizeSubjectTests(unittest.TestCase):
    def test_strips_pr_suffix_and_lowercases(self) -> None:
        self.assertEqual(normalize_subject("Fix: Broken Pane (#1234)"), "fix: broken pane")

    def test_keeps_subject_without_suffix(self) -> None:
        self.assertEqual(normalize_subject("docs: update dev manifest"), "docs: update dev manifest")


class GlobMatchTests(unittest.TestCase):
    def test_directory_subtree(self) -> None:
        self.assertTrue(glob_matches("website/**", "website/src/content/docs/index.mdx"))
        self.assertTrue(glob_matches("website/**", "website/index.html"))
        self.assertFalse(glob_matches("website/**", "website-old/index.html"))

    def test_exact_path(self) -> None:
        self.assertTrue(glob_matches("setup.sh", "setup.sh"))
        self.assertFalse(glob_matches("setup.sh", "nested/setup.sh"))

    def test_single_segment_wildcard(self) -> None:
        self.assertTrue(glob_matches("src/ui/*.rs", "src/ui/tabs.rs"))
        self.assertFalse(glob_matches("src/ui/*.rs", "src/ui/settings/rows.rs"))

    def test_prefix_wildcard(self) -> None:
        self.assertTrue(glob_matches("docs/CHEF-*", "docs/CHEF-ADR-001.md"))


class ParseOverlayTests(unittest.TestCase):
    def test_parses_valid_rows(self) -> None:
        rows, errors = parse_overlay(HEADER + "website/**\tsite\tmaintainer\twhy\tbuild\tnever\n")
        self.assertEqual(errors, [])
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0].path_glob, "website/**")

    def test_rejects_wrong_header(self) -> None:
        rows, errors = parse_overlay("a\tb\nx\ty\n")
        self.assertTrue(errors)
        self.assertEqual(rows, [])

    def test_rejects_duplicate_glob(self) -> None:
        text = HEADER + "website/**\tsite\tm\twhy\tbuild\tnever\nwebsite/**\tother\tm\twhy\tbuild\tnever\n"
        _, errors = parse_overlay(text)
        self.assertTrue(any("duplicate path_glob" in error for error in errors))

    def test_rejects_missing_column(self) -> None:
        text = HEADER + "website/**\tsite\tm\twhy\tbuild\n"
        _, errors = parse_overlay(text)
        self.assertTrue(any("expected 6 columns" in error for error in errors))

    def test_rejects_empty_required_cell(self) -> None:
        text = HEADER + "website/**\tsite\t\twhy\tbuild\tnever\n"
        _, errors = parse_overlay(text)
        self.assertTrue(any("missing owner" in error for error in errors))


class ValidatePinTests(unittest.TestCase):
    def pin(self, **overrides):
        base = {
            "repo": "herdrdev/herdr",
            "base_tag": "v0.9.3",
            "base_commit": "7b116c05bfda646af39d2524c54e70c751f57ee8",
            "reference": "master",
            "window_since": "2026-04-01",
            "generated_at": "2026-10-08T00:00:00Z",
        }
        base.update(overrides)
        return base

    def test_accepts_valid_pin(self) -> None:
        self.assertEqual(validate_pin(self.pin()), [])

    def test_requires_every_field(self) -> None:
        self.assertTrue(validate_pin(self.pin(reference="")))

    def test_rejects_non_sha_commit(self) -> None:
        self.assertTrue(validate_pin(self.pin(base_commit="not-a-sha")))

    def test_rejects_foreign_repo(self) -> None:
        self.assertTrue(validate_pin(self.pin(repo="someone/else")))


class ValidateOverlayTests(unittest.TestCase):
    def test_flags_glob_without_match(self) -> None:
        errors = validate_overlay([overlay_row("does/not/exist/**")], ["website/index.html"])
        self.assertTrue(any("matches no tracked file" in error for error in errors))

    def test_accepts_covering_glob(self) -> None:
        self.assertEqual(validate_overlay([overlay_row()], ["website/index.html"]), [])


class FingerprintBacklogTests(unittest.TestCase):
    def test_counts_unique_upstream_subjects_only(self) -> None:
        upstream = [
            ("a", "2026-09-01", "fix: alpha (#1)"),
            ("b", "2026-09-02", "docs: update preview manifest"),
            ("c", "2026-09-03", "docs: update preview manifest"),
        ]
        downstream = [("d", "2026-09-02", "docs: update preview manifest")]
        backlog = fingerprint_backlog(upstream, downstream)
        self.assertEqual([entry[0] for entry in backlog], ["a"])

    def test_excludes_subjects_present_downstream(self) -> None:
        upstream = [("a", "2026-09-01", "fix: alpha")]
        downstream = [("b", "2026-09-01", "Fix: Alpha")]
        self.assertEqual(fingerprint_backlog(upstream, downstream), [])


class ClassifyPathsTests(unittest.TestCase):
    def test_classifies_and_detects_relocations(self) -> None:
        upstream = {"src/a.rs": "1", "src/b.rs": "2", "crates/b.rs": "3"}
        downstream = {"src/a.rs": "9", "src/b.rs": "2", "src/legacy.rs": "3"}
        result = classify_paths(upstream, downstream)
        self.assertEqual(result["paths_common"], 2)
        self.assertEqual(result["paths_identical"], 1)
        self.assertEqual(result["paths_differing"], ["src/a.rs"])
        self.assertEqual(result["paths_only_upstream"], ["crates/b.rs"])
        self.assertEqual(result["paths_only_downstream"], ["src/legacy.rs"])
        self.assertEqual(result["relocated_only_downstream"], {"src/legacy.rs": "crates/b.rs"})

    def test_flags_critical_prefixes(self) -> None:
        upstream = {"src/integration/mod.rs": "1"}
        downstream = {"src/integration/mod.rs": "2"}
        result = classify_paths(upstream, downstream)
        self.assertEqual(result["critical_paths_changed"], ["src/integration/mod.rs"])


class CoverageTests(unittest.TestCase):
    def test_reports_uncovered_paths(self) -> None:
        rows = [overlay_row()]
        uncovered = uncovered_overlay_paths(["website/a.md", "src/other.rs"], rows)
        self.assertEqual(uncovered, ["src/other.rs"])


class OverlayScopeTests(unittest.TestCase):
    def test_flags_undeclared_overlay_path(self) -> None:
        errors = validate_scope(["website/index.html", "src/random.rs"], [overlay_row()])
        self.assertTrue(any("not declared" in error for error in errors))

    def test_accepts_declared_overlay_paths(self) -> None:
        self.assertEqual(validate_scope(["website/index.html"], [overlay_row()]), [])

    def test_accepts_empty_scope(self) -> None:
        self.assertEqual(validate_scope([], []), [])


class ValidateLedgerTests(unittest.TestCase):
    def ledger(self, **overrides):
        base = {
            "schema_version": 1,
            "upstream": {"base_commit": "abc1234"},
            "counts": {"unported": 1, "paths_only_downstream": 1},
            "paths_only_downstream_list": ["website/a.md"],
            "unported_commits": [{"sha": "a", "date": "2026-09-01", "subject": "fix: x"}],
        }
        base.update(overrides)
        return base

    def test_accepts_consistent_ledger(self) -> None:
        pin = {"base_commit": "abc1234"}
        self.assertEqual(validate_ledger(self.ledger(), pin, [overlay_row()]), [])

    def test_flags_base_mismatch(self) -> None:
        errors = validate_ledger(self.ledger(), {"base_commit": "zzz"}, [overlay_row()])
        self.assertTrue(any("base_commit does not match" in error for error in errors))

    def test_flags_count_mismatch(self) -> None:
        ledger = self.ledger(counts={"unported": 5, "paths_only_downstream": 1})
        errors = validate_ledger(ledger, {"base_commit": "abc1234"}, [overlay_row()])
        self.assertTrue(any("counts.unported" in error for error in errors))

    def test_flags_uncovered_paths(self) -> None:
        errors = validate_ledger(self.ledger(), {"base_commit": "abc1234"}, [])
        self.assertTrue(any("not covered" in error for error in errors))


class CommittedLedgerTests(unittest.TestCase):
    """The committed pin, overlay inventory and snapshot must stay consistent."""

    def test_repository_ledger_is_consistent(self) -> None:
        self.assertEqual(command_check(quiet=True), 0)


if __name__ == "__main__":
    unittest.main()
