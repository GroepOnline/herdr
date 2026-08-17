import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Files that must contain the literal site URL (mirrors of the constant in
# `src/product_urls.rs`). When the site moves to a custom domain, update
# `src/product_urls.rs` AND all of these together.
SITE_URL_MIRRORS = [
    "website/src/config/product.ts",
    "scripts/product_config.py",
    "website/install.sh",
    "website/install.ps1",
    "Cargo.toml",
    "npm/package.json",
    "justfile",
    "website/scripts/check-built-docs.mjs",
]

# Rust modules that import the shared constants instead of mirroring them.
RUST_IMPORTERS = ["src/update.rs", "src/remote/attach.rs", "src/cli.rs"]

# The update-chain surface. None of these may reference an upstream (herdr.dev
# / herdr.pages.dev / herdrdev) deployment, which would silently break
# installs/updates and pull upstream binaries.
UPDATE_CHAIN_FILES = [
    "src/product_urls.rs",
    "src/update.rs",
    "src/remote/attach.rs",
    "src/cli.rs",
    "website/src/config/product.ts",
    "scripts/product_config.py",
    "website/install.sh",
    "website/install.ps1",
]

# Of those, only these carry the literal branded URL; src/update.rs and
# src/remote/attach.rs import the constants from src/product_urls.rs instead.
BRANDED_URL_MIRRORS = [
    "src/product_urls.rs",
    "website/src/config/product.ts",
    "scripts/product_config.py",
    "website/install.sh",
    "website/install.ps1",
]

UPSTREAM_DOMAINS = ("herdr.pages.dev", "herdr.dev")


def site_url() -> str:
    src = (ROOT / "src" / "product_urls.rs").read_text(encoding="utf-8")
    match = re.search(r'PRODUCT_SITE_URL: &str = "([^"]+)"', src)
    assert match, "PRODUCT_SITE_URL not found in src/product_urls.rs"
    return match.group(1)


class ProductUrlSyncTests(unittest.TestCase):
    def test_site_url_mirrors_reference_the_same_url(self):
        expected = site_url()
        for rel in SITE_URL_MIRRORS:
            with self.subTest(file=rel):
                text = (ROOT / rel).read_text(encoding="utf-8")
                self.assertIn(expected, text, f"{rel} does not reference {expected}")

    def test_rust_modules_import_the_shared_constants(self):
        # src/update.rs derives its fallback URLs from the site constant.
        update_text = (ROOT / "src/update.rs").read_text(encoding="utf-8")
        self.assertIn("crate::product_urls::PRODUCT_SITE_URL", update_text)
        # src/remote/attach.rs consumes the channel manifest constants.
        attach_text = (ROOT / "src/remote/attach.rs").read_text(encoding="utf-8")
        self.assertIn("crate::product_urls", attach_text)
        for name in ("STABLE_UPDATE_MANIFEST_URL", "PREVIEW_UPDATE_MANIFEST_URL", "DEV_UPDATE_MANIFEST_URL"):
            self.assertIn(name, attach_text, f"attach.rs missing {name}")
        cli_text = (ROOT / "src/cli.rs").read_text(encoding="utf-8")
        self.assertIn("crate::product_urls::AGENT_GUIDE_URL", cli_text)
        self.assertIn("crate::product_urls::LLMS_TXT_URL", cli_text)
        self.assertNotIn("https://herdr.chefgroep.nl/agent-guide.md", cli_text)
        self.assertNotIn("https://herdr.chefgroep.nl/llms.txt", cli_text)

    def test_update_chain_never_references_upstream_domains(self):
        for rel in UPDATE_CHAIN_FILES:
            with self.subTest(file=rel):
                text = (ROOT / rel).read_text(encoding="utf-8")
                for domain in UPSTREAM_DOMAINS:
                    self.assertNotIn(
                        domain,
                        text,
                        f"{rel} references upstream domain {domain}; "
                        "the update chain must stay on the branded deployment",
                    )

    def test_update_chain_carries_the_branded_site_url(self):
        expected = site_url()
        for rel in BRANDED_URL_MIRRORS:
            with self.subTest(file=rel):
                text = (ROOT / rel).read_text(encoding="utf-8")
                self.assertIn(expected, text, f"{rel} does not reference the branded {expected}")

    def test_public_catalog_and_cli_surfaces_use_the_branded_site(self):
        manifest_update = (ROOT / "src/detect/manifest_update.rs").read_text(encoding="utf-8")
        self.assertIn(
            "https://herdr.chefgroep.nl/agent-detection/index.toml",
            manifest_update,
        )
        main = (ROOT / "src/main.rs").read_text(encoding="utf-8")
        self.assertIn('Home:   https://herdr.chefgroep.nl', main)
        self.assertNotIn('Home:   https://herdr.dev', main)
        self.assertNotIn('Check herdr.dev', main)

    def test_channel_manifests_derive_from_site_url(self):
        expected = site_url()
        source = (ROOT / "src" / "product_urls.rs").read_text(encoding="utf-8")
        for suffix in ("latest.json", "preview.json", "dev.json", "agent-guide.md", "llms.txt"):
            self.assertIn(
                f"{expected}/{suffix}",
                source,
                f"src/product_urls.rs missing {expected}/{suffix}",
            )


if __name__ == "__main__":
    unittest.main()
