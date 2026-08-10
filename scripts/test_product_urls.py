import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Files that must contain the literal live site URL (mirrors of the constant
# in `src/product_urls.rs`). When the site moves to a custom domain, update
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
RUST_IMPORTERS = ["src/update.rs", "src/remote/attach.rs"]

# These must never reference the (currently dead) custom domain; they are the
# update-chain surface that would silently break installs/updates.
UPDATE_CHAIN_FILES = [
    "src/product_urls.rs",
    "src/update.rs",
    "src/remote/attach.rs",
    "website/src/config/product.ts",
    "scripts/product_config.py",
    "website/install.sh",
    "website/install.ps1",
]


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
        for rel in RUST_IMPORTERS:
            with self.subTest(file=rel):
                text = (ROOT / rel).read_text(encoding="utf-8")
                self.assertIn("crate::product_urls", text, f"{rel} does not import crate::product_urls")
                self.assertIn("STABLE_UPDATE_MANIFEST_URL", text, f"{rel} missing STABLE_UPDATE_MANIFEST_URL")
                self.assertIn("PREVIEW_UPDATE_MANIFEST_URL", text, f"{rel} missing PREVIEW_UPDATE_MANIFEST_URL")
                self.assertIn("DEV_UPDATE_MANIFEST_URL", text, f"{rel} missing DEV_UPDATE_MANIFEST_URL")

    def test_update_chain_never_references_dead_custom_domain(self):
        for rel in UPDATE_CHAIN_FILES:
            with self.subTest(file=rel):
                text = (ROOT / rel).read_text(encoding="utf-8")
                self.assertNotIn(
                    "herdr.chefgroep.nl",
                    text,
                    f"{rel} still references the dead herdr.chefgroep.nl domain",
                )

    def test_channel_manifests_derive_from_site_url(self):
        expected = site_url()
        source = (ROOT / "src" / "product_urls.rs").read_text(encoding="utf-8")
        for suffix in ("latest.json", "preview.json", "dev.json"):
            self.assertIn(
                f"{expected}/{suffix}",
                source,
                f"src/product_urls.rs missing {expected}/{suffix}",
            )


if __name__ == "__main__":
    unittest.main()
