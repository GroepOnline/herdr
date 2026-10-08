#!/usr/bin/env bash
# Resolve or download a runnable herdr binary for Cloud verification.
# Prefer: HERDR_BIN → /opt/herdr → release asset → CI smoke → PATH.
set -euo pipefail

REPO="${HERDR_CLOUD_REPO:-GroepOnline/herdr}"
OPT_BIN="${HERDR_CLOUD_BIN_DIR:-/opt/herdr}/herdr-linux-x86_64"
FRESH_DIR="${HERDR_CLOUD_FRESH_DIR:-/tmp/herdr-bin-fresh}"
FRESH_BIN="$FRESH_DIR/herdr-linux-x86_64"
ARTIFACT_NAME="ci-smoke-herdr-linux-x86_64"
RELEASE_ASSET="herdr-linux-x86_64"

log() { printf '[download-binary] %s\n' "$*" >&2; }

pick_existing() {
  if [[ -n "${HERDR_BIN:-}" && -x "$HERDR_BIN" ]]; then
    printf '%s\n' "$HERDR_BIN"
    return 0
  fi
  for candidate in "$OPT_BIN" "$FRESH_BIN" /tmp/herdr-bin/herdr-linux-x86_64 \
    "$(command -v herdr 2>/dev/null || true)"; do
    if [[ -n "$candidate" && -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

download_release() {
  command -v gh >/dev/null 2>&1 || return 1
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
  [[ -n "$tag" ]] || return 1
  log "downloading release $tag"
  gh release download "$tag" --repo "$REPO" -p "$RELEASE_ASSET" -D "$staging"
  mkdir -p "$FRESH_DIR"
  install -m 0755 "$staging/$RELEASE_ASSET" "$FRESH_BIN"
  if [[ -w "$(dirname "$OPT_BIN")" ]] || command -v sudo >/dev/null 2>&1; then
    mkdir -p "$(dirname "$OPT_BIN")" 2>/dev/null || sudo mkdir -p "$(dirname "$OPT_BIN")"
    if install -m 0755 "$staging/$RELEASE_ASSET" "$OPT_BIN" 2>/dev/null; then
      :
    else
      sudo install -m 0755 "$staging/$RELEASE_ASSET" "$OPT_BIN"
    fi
    ln -sfn "$OPT_BIN" /usr/local/bin/herdr 2>/dev/null \
      || sudo ln -sfn "$OPT_BIN" /usr/local/bin/herdr
  fi
  printf '%s\n' "$FRESH_BIN"
}

download_ci() {
  command -v gh >/dev/null 2>&1 || return 1
  local run_id staging
  run_id="$(
    gh run list --repo "$REPO" --workflow=ci.yml --branch main --status success \
      --limit 20 --json databaseId --jq '.[].databaseId' \
      | while read -r id; do
          if gh api "repos/${REPO}/actions/runs/${id}/artifacts" \
            --jq ".artifacts[] | select(.name == \"${ARTIFACT_NAME}\" and .expired == false) | .id" \
            | grep -q .; then
            printf '%s\n' "$id"
            break
          fi
        done
  )"
  [[ -n "$run_id" ]] || return 1
  staging="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$staging'" RETURN
  log "downloading CI smoke from run $run_id"
  rm -rf "$FRESH_DIR"
  mkdir -p "$FRESH_DIR"
  gh run download "$run_id" -n "$ARTIFACT_NAME" -D "$FRESH_DIR" --repo "$REPO"
  chmod +x "$FRESH_BIN"
  printf '%s\n' "$FRESH_BIN"
}

if path="$(pick_existing)"; then
  export HERDR_BIN="$path"
else
  if path="$(download_release 2>/dev/null)"; then
    export HERDR_BIN="$path"
  elif path="$(download_ci 2>/dev/null)"; then
    export HERDR_BIN="$path"
  else
    echo "error: could not resolve or download herdr binary" >&2
    exit 1
  fi
fi

log "HERDR_BIN=$HERDR_BIN ($("$HERDR_BIN" --version 2>/dev/null || echo unknown))"
printf '%s\n' "$HERDR_BIN"
