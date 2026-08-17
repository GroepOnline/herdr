//! Product-level URLs shared across update and attach paths.
//!
//! Single source of truth for the branded product site and its channel
//! manifests. `scripts/test_product_urls.py` (maintenance CI) guards that the
//! mirrors in `website/src/config/product.ts`, `scripts/product_config.py`,
//! `website/install.sh`, `website/install.ps1`, `Cargo.toml`,
//! `npm/package.json`, `justfile`, and `website/scripts/check-built-docs.mjs`
//! stay in sync with this constant. Production consumers: `src/update.rs`
//! derives its fallback manifest URLs from the site constant,
//! `src/remote/attach.rs` uses the channel manifest constants, and
//! `src/cli.rs` derives the agent-help footer from the site constants.
//!
//! NOTE: the branded domain must be the GroepOnline deployment. The upstream
//! (herdrdev) Pages deployment must never appear in the update chain; the sync
//! guard fails if it does. The branded domain is served from a Cloudflare Pages
//! project with a custom domain; if its DNS record is missing, restore the
//! custom domain before the next release (the release chain verifies the live
//! manifest after promotion).

/// Branded product site URL (GroepOnline downstream deployment).
pub const PRODUCT_SITE_URL: &str = "https://herdr.chefgroep.nl";

/// Stable update channel manifest.
pub const STABLE_UPDATE_MANIFEST_URL: &str = "https://herdr.chefgroep.nl/latest.json";
/// Preview update channel manifest.
pub const PREVIEW_UPDATE_MANIFEST_URL: &str = "https://herdr.chefgroep.nl/preview.json";
/// Dev update channel manifest.
pub const DEV_UPDATE_MANIFEST_URL: &str = "https://herdr.chefgroep.nl/dev.json";

/// Plain-text first-run guide published at the product site root.
pub const AGENT_GUIDE_URL: &str = "https://herdr.chefgroep.nl/agent-guide.md";
/// Plain-text documentation index published at the product site root.
pub const LLMS_TXT_URL: &str = "https://herdr.chefgroep.nl/llms.txt";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_urls_are_hosted_on_the_product_site() {
        for url in [
            STABLE_UPDATE_MANIFEST_URL,
            PREVIEW_UPDATE_MANIFEST_URL,
            DEV_UPDATE_MANIFEST_URL,
            AGENT_GUIDE_URL,
            LLMS_TXT_URL,
        ] {
            assert!(
                url.starts_with(PRODUCT_SITE_URL),
                "{url} does not start with {PRODUCT_SITE_URL}"
            );
        }
    }
}
