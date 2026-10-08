#!/usr/bin/env bash
# Idempotent Cloud Agent update script for herdr.
# Keeps the CI-only validation policy: no local cargo/zig/just builds.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

HERDR_BIN_DIR="${HERDR_CLOUD_BIN_DIR:-/opt/herdr}"
HERDR_BIN="$HERDR_BIN_DIR/herdr-linux-x86_64"
FRESH_DIR="${HERDR_CLOUD_FRESH_DIR:-/tmp/herdr-bin-fresh}"
ARTIFACT_NAME="ci-smoke-herdr-linux-x86_64"
RELEASE_ASSET="herdr-linux-x86_64"
REPO="${HERDR_CLOUD_REPO:-GroepOnline/herdr}"

log() {
  printf '[herdr-cloud-install] %s\n' "$*"
}

install_binary_from() {
  local src="$1"
  local label="$2"
  if [[ ! -f "$src" ]]; then
    return 1
  fi
  mkdir -p "$HERDR_BIN_DIR" "$FRESH_DIR"
  if [[ -w "$HERDR_BIN_DIR" ]]; then
    install -m 0755 "$src" "$HERDR_BIN"
  else
    sudo install -m 0755 "$src" "$HERDR_BIN"
  fi
  install -m 0755 "$src" "$FRESH_DIR/herdr-linux-x86_64"
  if ln -sfn "$HERDR_BIN" /usr/local/bin/herdr 2>/dev/null; then
    :
  elif command -v sudo >/dev/null 2>&1; then
    sudo ln -sfn "$HERDR_BIN" /usr/local/bin/herdr
  fi
  log "herdr binary ready ($label): $("$HERDR_BIN" --version 2>/dev/null || echo unknown)"
  return 0
}

download_latest_release() {
  if ! command -v gh >/dev/null 2>&1; then
    return 1
  fi
  local staging tag
  staging="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$staging'" RETURN

  tag="$(
    gh release list --repo "$REPO" --limit 20 \
      --json tagName,isDraft,isPrerelease \
      --jq '.[] | select(.isDraft == false and .isPrerelease == false) | .tagName' \
      | head -n1
  )"
  if [[ -z "$tag" ]]; then
    return 1
  fi

  log "downloading $RELEASE_ASSET from release $tag"
  if ! gh release download "$tag" --repo "$REPO" -p "$RELEASE_ASSET" -D "$staging" 2>/dev/null; then
    return 1
  fi
  install_binary_from "$staging/$RELEASE_ASSET" "release $tag"
}

download_ci_smoke() {
  if ! command -v gh >/dev/null 2>&1; then
    return 1
  fi

  local run_id staging
  run_id="$(
    gh run list \
      --repo "$REPO" \
      --workflow=ci.yml \
      --branch main \
      --status success \
      --limit 20 \
      --json databaseId \
      --jq '.[].databaseId' \
      | while read -r id; do
          if gh api "repos/${REPO}/actions/runs/${id}/artifacts" \
            --jq ".artifacts[] | select(.name == \"${ARTIFACT_NAME}\" and .expired == false) | .id" \
            | grep -q .; then
            printf '%s\n' "$id"
            break
          fi
        done
  )"
  if [[ -z "$run_id" ]]; then
    return 1
  fi

  staging="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$staging'" RETURN
  log "downloading $ARTIFACT_NAME from CI run $run_id"
  gh run download "$run_id" -n "$ARTIFACT_NAME" -D "$staging" --repo "$REPO"
  install_binary_from "$staging/herdr-linux-x86_64" "ci run $run_id"
}

ensure_herdr_binary() {
  mkdir -p "$HERDR_BIN_DIR"

  if download_latest_release; then
    return 0
  fi
  log "release asset unavailable; trying CI smoke artifact"
  if download_ci_smoke; then
    return 0
  fi

  if [[ -x "$HERDR_BIN" ]]; then
    log "keeping existing $HERDR_BIN ($("$HERDR_BIN" --version 2>/dev/null || echo unknown))"
    return 0
  fi
  log "warning: no herdr binary installed"
}

ensure_website_deps() {
  if [[ ! -f website/package-lock.json ]]; then
    return 0
  fi
  if ! command -v npm >/dev/null 2>&1; then
    log "npm not available; skipping website deps"
    return 0
  fi
  log "installing website npm dependencies"
  (cd website && npm ci --ignore-scripts)
}

ensure_cursor_standards() {
  # Download / refresh Cursor artifact catalog ("standaarden") + committed indexes.
  chmod +x .cursor/hooks/fetch-cursor-artifacts.sh \
    scripts/check_cursor_artifacts.sh \
    .cursor/scripts/cloud-install.sh 2>/dev/null || true

  if [[ -x .cursor/hooks/fetch-cursor-artifacts.sh ]]; then
    log "refreshing Cursor artifact catalog"
    .cursor/hooks/fetch-cursor-artifacts.sh --write-cache \
      || log "warning: fetch-cursor-artifacts failed (non-fatal)"
  fi
  if [[ -f scripts/generate_cursor_index.py ]]; then
    log "regenerating Cursor artifact indexes"
    python3 scripts/generate_cursor_index.py --allow-org-leak \
      || log "warning: generate_cursor_index failed (non-fatal)"
  fi
}

ensure_herdr_binary
ensure_website_deps
ensure_cursor_standards

log "done (local Rust/Zig builds remain forbidden; validate via GitHub Actions)"
