import tempfile
import unittest
from pathlib import Path

from scripts import agent_detection_manifest_check as check


def manifest(agent_id: str, version: str, contains: str = "ready") -> str:
    return f'''id = "{agent_id}"
version = "{version}"
min_engine_version = 1
updated_at = "2026-06-10T00:00:00Z"

[[rules]]
id = "idle"
state = "idle"
contains = ["{contains}"]
'''


def catalog(agent_id: str = "codex", path: str = "codex.toml") -> str:
    return f'''schema_version = 1

[[agents]]
id = "{agent_id}"
path = "{path}"
'''


def staged_grok_dirs(root: Path) -> tuple[Path, Path]:
    bundled = root / "bundled"
    website = root / "website"
    bundled.mkdir()
    website.mkdir()
    (bundled / "grok.toml").write_bytes(
        (check.DEFAULT_BUNDLED_DIR / "grok.toml").read_bytes()
    )
    (website / "grok.toml").write_bytes(
        (check.DEFAULT_WEBSITE_DIR / "grok.toml").read_bytes()
    )
    (website / "index.toml").write_text(catalog("grok", "grok.toml"))
    return bundled, website


class AgentDetectionManifestCheckTests(unittest.TestCase):
    def test_validates_bundled_and_matching_website_catalog(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bundled = root / "bundled"
            website = root / "website"
            bundled.mkdir()
            website.mkdir()
            content = manifest("codex", "2026.06.10.1")
            (bundled / "codex.toml").write_text(content)
            (website / "codex.toml").write_text(content)
            (website / "index.toml").write_text(catalog())

            bundled_manifests = check.load_manifest_dir(bundled, engine_version=1)
            check.validate_catalog(website, bundled_manifests, engine_version=1)

    def test_rejects_website_version_lower_than_bundled(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bundled = root / "bundled"
            website = root / "website"
            bundled.mkdir()
            website.mkdir()
            (bundled / "codex.toml").write_text(manifest("codex", "2026.06.10.2"))
            (website / "codex.toml").write_text(manifest("codex", "2026.06.10.1"))
            (website / "index.toml").write_text(catalog())

            bundled_manifests = check.load_manifest_dir(bundled, engine_version=1)
            with self.assertRaisesRegex(check.CheckError, "lower than bundled"):
                check.validate_catalog(website, bundled_manifests, engine_version=1)

    def test_allows_explicitly_staged_website_manifest(self):
        with tempfile.TemporaryDirectory() as tmp:
            bundled, website = staged_grok_dirs(Path(tmp))

            bundled_manifests = check.load_manifest_dir(bundled, engine_version=3)
            check.validate_catalog(website, bundled_manifests, engine_version=3)

    def test_rejects_mutated_staged_website_manifest(self):
        with tempfile.TemporaryDirectory() as tmp:
            bundled, website = staged_grok_dirs(Path(tmp))
            with (website / "grok.toml").open("a") as manifest_file:
                manifest_file.write("\n# unexpected mutation\n")

            bundled_manifests = check.load_manifest_dir(bundled, engine_version=3)
            with self.assertRaisesRegex(check.CheckError, "lower than bundled"):
                check.validate_catalog(website, bundled_manifests, engine_version=3)

    def test_rejects_unlisted_website_manifest_lag_for_new_engine(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bundled = root / "bundled"
            website = root / "website"
            bundled.mkdir()
            website.mkdir()
            bundled_content = manifest("codex", "2026.06.10.2").replace(
                "min_engine_version = 1", "min_engine_version = 2"
            )
            (bundled / "codex.toml").write_text(bundled_content)
            (website / "codex.toml").write_text(manifest("codex", "2026.06.10.1"))
            (website / "index.toml").write_text(catalog())

            bundled_manifests = check.load_manifest_dir(bundled, engine_version=2)
            with self.assertRaisesRegex(check.CheckError, "lower than bundled"):
                check.validate_catalog(website, bundled_manifests, engine_version=2)

    def test_rejects_same_version_content_drift(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bundled = root / "bundled"
            website = root / "website"
            bundled.mkdir()
            website.mkdir()
            (bundled / "codex.toml").write_text(manifest("codex", "2026.06.10.1", "ready"))
            (website / "codex.toml").write_text(manifest("codex", "2026.06.10.1", "changed"))
            (website / "index.toml").write_text(catalog())

            bundled_manifests = check.load_manifest_dir(bundled, engine_version=1)
            with self.assertRaisesRegex(check.CheckError, "same version"):
                check.validate_catalog(website, bundled_manifests, engine_version=1)

    def test_rejects_unknown_catalog_agent(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bundled = root / "bundled"
            website = root / "website"
            bundled.mkdir()
            website.mkdir()
            (bundled / "codex.toml").write_text(manifest("codex", "2026.06.10.1"))
            (website / "newagent.toml").write_text(manifest("newagent", "2026.06.10.1"))
            (website / "index.toml").write_text(catalog("newagent", "newagent.toml"))

            bundled_manifests = check.load_manifest_dir(bundled, engine_version=1)
            with self.assertRaisesRegex(check.CheckError, "unknown agent"):
                check.validate_catalog(website, bundled_manifests, engine_version=1)

    def test_rejects_manifest_requiring_newer_engine(self):
        with tempfile.TemporaryDirectory() as tmp:
            bundled = Path(tmp) / "bundled"
            bundled.mkdir()
            (bundled / "codex.toml").write_text(
                manifest("codex", "2026.06.10.1").replace(
                    "min_engine_version = 1", "min_engine_version = 2"
                )
            )

            with self.assertRaisesRegex(check.CheckError, "exceeds engine"):
                check.load_manifest_dir(bundled, engine_version=1)

    def test_rejects_top_non_empty_lines_below_engine_three(self):
        with tempfile.TemporaryDirectory() as tmp:
            bundled = Path(tmp) / "bundled"
            bundled.mkdir()
            content = manifest("codex", "2026.06.10.1").replace(
                'contains = ["ready"]',
                'region = "top_non_empty_lines(1)"\ncontains = ["ready"]',
            )
            (bundled / "codex.toml").write_text(content)

            with self.assertRaisesRegex(check.CheckError, "requires min_engine_version 3"):
                check.load_manifest_dir(bundled, engine_version=3)

    def test_top_non_empty_lines_requires_canonical_positive_bounded_count(self):
        base_rule = {
            "id": "test",
            "state": "working",
            "contains": ["ready"],
        }
        name = "top_non_empty_lines"
        for count in ("1", str(check.MAX_TOP_REGION_LINE_COUNT)):
            rule = {**base_rule, "region": f"{name}({count})"}
            check.validate_rule(Path("test.toml"), 0, rule, {"gates": 0, "matchers": 0})
        for count in (
            "0",
            "01",
            "+1",
            str(check.MAX_TOP_REGION_LINE_COUNT + 1),
            "9" * 40,
        ):
            rule = {**base_rule, "region": f"{name}({count})"}
            with self.subTest(region=rule["region"]):
                with self.assertRaisesRegex(check.CheckError, "invalid region"):
                    check.validate_rule(
                        Path("test.toml"), 0, rule, {"gates": 0, "matchers": 0}
                    )


GENERIC_ERROR_MATCHERS = {"error", "error:", "err", "err:", "failed", "failure"}


def iter_matchers(node: object):
    if not isinstance(node, dict):
        return
    for key in ("contains", "regex", "line_regex"):
        for value in node.get(key, []):
            yield key, value
    for nested_key in ("any", "all", "not"):
        for nested in node.get(nested_key, []):
            yield from iter_matchers(nested)


def is_generic_error_matcher(kind: str, value: str) -> bool:
    compact = value.strip().lower()
    if compact in GENERIC_ERROR_MATCHERS:
        return True
    if kind in {"regex", "line_regex"}:
        stripped = (
            compact.replace("(?i)", "").replace("^", "").replace("$", "").replace(".*", "")
        )
        if stripped in GENERIC_ERROR_MATCHERS:
            return True
    return False


class AiderDetectionContractTests(unittest.TestCase):
    def test_aider_manifest_forbids_generic_error_and_keeps_error_bottom_scoped(self):
        path = check.DEFAULT_BUNDLED_DIR / "aider.toml"
        manifest = check.load_toml(path)
        states = {rule["state"] for rule in manifest["rules"]}
        self.assertEqual(states, {"idle", "working", "blocked"})

        error_rules = [rule for rule in manifest["rules"] if rule["id"] == "error_detected"]
        self.assertEqual(len(error_rules), 1)
        error_rule = error_rules[0]
        self.assertEqual(error_rule["state"], "blocked")
        self.assertTrue(str(error_rule.get("region", "")).startswith("bottom_"))

        generic = [
            (rule["id"], kind, value)
            for rule in manifest["rules"]
            for kind, value in iter_matchers(rule)
            if is_generic_error_matcher(kind, value)
        ]
        self.assertEqual(generic, [])

    def test_aider_has_no_hooks_integration(self):
        project = check.PROJECT_ROOT
        integrations = (project / "src/api/schema/integrations.rs").read_text(encoding="utf-8")
        self.assertNotRegex(integrations, r"\bAider\b")
        self.assertFalse((project / "src/integration/assets/aider").exists())
        cli = (project / "src/cli/integration.rs").read_text(encoding="utf-8")
        self.assertNotIn('"aider"', cli)

    def test_capture_script_maps_error_chrome_to_blocked(self):
        from scripts import capture_agent_screen

        self.assertEqual(capture_agent_screen.STATE_CHOICES["error"], "blocked")
        self.assertEqual(capture_agent_screen.STATE_CHOICES["e"], "blocked")
        self.assertEqual(capture_agent_screen.STATE_CHOICES["blocked"], "blocked")


if __name__ == "__main__":
    unittest.main()
