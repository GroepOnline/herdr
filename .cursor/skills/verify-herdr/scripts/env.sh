#!/usr/bin/env bash
# Resolve Herdr binary + allocate isolated HOME and evidence dirs.
# Usage: source .cursor/skills/verify-herdr/scripts/env.sh
set -euo pipefail

_verify_herdr_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export HERDR_VERIFY_SKILL_ROOT="$_verify_herdr_root"

_resolve_bin() {
  if [[ -n "${HERDR_BIN:-}" && -x "$HERDR_BIN" ]]; then
    return 0
  fi
  for candidate in \
    /opt/herdr/herdr-linux-x86_64 \
    /tmp/herdr-bin-fresh/herdr-linux-x86_64 \
    /tmp/herdr-bin/herdr-linux-x86_64 \
    "$(command -v herdr 2>/dev/null || true)"; do
    if [[ -n "$candidate" && -x "$candidate" ]]; then
      HERDR_BIN="$candidate"
      return 0
    fi
  done
  # Last resort: download helper (release → CI smoke).
  if [[ -x "$_verify_herdr_root/scripts/download-binary.sh" ]]; then
    HERDR_BIN="$("$_verify_herdr_root/scripts/download-binary.sh" | tail -n1)"
    if [[ -n "$HERDR_BIN" && -x "$HERDR_BIN" ]]; then
      return 0
    fi
  fi
  echo "error: no herdr binary found. Set HERDR_BIN or run scripts/download-binary.sh." >&2
  return 1
}

_resolve_bin
export HERDR_BIN

if [[ -z "${HERDR_HOME:-}" ]]; then
  HERDR_HOME="$(mktemp -d /tmp/herdr-verify-home.XXXXXX)"
fi
export HERDR_HOME

if [[ -z "${HERDR_VERIFY_EVIDENCE:-}" ]]; then
  _run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
  HERDR_VERIFY_EVIDENCE="/opt/cursor/artifacts/herdr-verify/${_run_id}"
fi
mkdir -p "$HERDR_VERIFY_EVIDENCE"
export HERDR_VERIFY_EVIDENCE

export HERDR_VERIFY_META="${HERDR_HOME}/.verify-meta"
mkdir -p "$HERDR_VERIFY_META" "$HERDR_HOME/.config/herdr"

# Disable onboarding noise for headless drives when config is missing.
if [[ ! -f "$HERDR_HOME/.config/herdr/config.toml" ]]; then
  printf 'onboarding = false\n' >"$HERDR_HOME/.config/herdr/config.toml"
fi

export HERDR_SOCKET="$HERDR_HOME/.config/herdr/herdr.sock"

cat >"$HERDR_VERIFY_EVIDENCE/env.txt" <<EOF
HERDR_BIN=$HERDR_BIN
HERDR_HOME=$HERDR_HOME
HERDR_SOCKET=$HERDR_SOCKET
HERDR_VERIFY_EVIDENCE=$HERDR_VERIFY_EVIDENCE
version=$("$HERDR_BIN" --version 2>/dev/null || echo unknown)
EOF

# Child helpers (launch/doctor/cli) re-source this file; keep the drive log quiet.
if [[ "${HERDR_VERIFY_QUIET:-0}" != "1" && "${HERDR_VERIFY_ENV_ANNOUNCED:-0}" != "1" ]]; then
  echo "verify-herdr env ready"
  echo "  HERDR_BIN=$HERDR_BIN"
  echo "  HERDR_HOME=$HERDR_HOME"
  echo "  HERDR_VERIFY_EVIDENCE=$HERDR_VERIFY_EVIDENCE"
fi
export HERDR_VERIFY_ENV_ANNOUNCED=1
export HERDR_VERIFY_QUIET=1
