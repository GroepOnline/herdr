//! Product-level URLs shared across update and attach paths.
//!
//! Single source of truth for the live product site and its channel
//! manifests. `scripts/test_product_urls.py` (maintenance CI) guards that the
//! mirrors in `website/src/config/product.ts`, `scripts/product_config.py`,
//! `website/install.sh`, `website/install.ps1`, `Cargo.toml`,
//! `npm/package.json`, `justfile`, and `website/scripts/check-built-docs.mjs`
//! stay in sync.
//!
//! The site is served from Cloudflare Pages (`herdr.pages.dev`). A branded
//! custom domain may replace it once its DNS record exists; update every
//! mirror at the same time (the sync test will force it).

/// Live product site URL.
pub const PRODUCT_SITE_URL: &str = "https://herdr.pages.dev";

/// Stable update channel manifest.
pub const STABLE_UPDATE_MANIFEST_URL: &str = "https://herdr.pages.dev/latest.json";
/// Preview update channel manifest.
pub const PREVIEW_UPDATE_MANIFEST_URL: &str = "https://herdr.pages.dev/preview.json";
/// Dev update channel manifest.
pub const DEV_UPDATE_MANIFEST_URL: &str = "https://herdr.pages.dev/dev.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_urls_are_hosted_on_the_product_site() {
        for url in [
            STABLE_UPDATE_MANIFEST_URL,
            PREVIEW_UPDATE_MANIFEST_URL,
            DEV_UPDATE_MANIFEST_URL,
        ] {
            assert!(
                url.starts_with(PRODUCT_SITE_URL),
                "{url} does not start with {PRODUCT_SITE_URL}"
            );
        }
    }
}
