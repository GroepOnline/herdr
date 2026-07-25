"""Canonical OnlineChefGroep Herdr product metadata (single source of truth)."""

from __future__ import annotations

PRODUCT_GITHUB_ORG = "OnlineChefGroep"
PRODUCT_GITHUB_REPO = "OnlineChefGroep/herdr"
UPSTREAM_GITHUB_REPO = "ogulcancelik/herdr"
PRODUCT_SITE_URL = "https://herdr.chefgroep.nl"
PRODUCT_CONTACT_EMAIL = "hey@chefgroep.online"

DEFAULT_LIVE_MANIFEST_URL = f"{PRODUCT_SITE_URL}/latest.json"
DEFAULT_PREVIEW_MANIFEST_URL = f"{PRODUCT_SITE_URL}/preview.json"
DEFAULT_DEV_MANIFEST_URL = f"{PRODUCT_SITE_URL}/dev.json"

# Homebrew tap is a separate repo; formula generation lives in scripts/homebrew_formula.py
HOMEBREW_TAP_REPO = "OnlineChefGroep/homebrew-tap"
HOMEBREW_FORMULA_NAME = "onlinechefgroep-herdr"
HOMEBREW_INSTALL_HINT = (
    "brew tap OnlineChefGroep/tap && "
    "brew install OnlineChefGroep/tap/onlinechefgroep-herdr"
)
