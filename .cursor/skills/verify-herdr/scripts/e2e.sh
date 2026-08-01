#!/usr/bin/env bash
# Full end-to-end proof across the verify-herdr feature map.
# One isolated server, smooth step progress, evidence kept on exit.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Prefer durable cloud install; allow explicit override.
if [[ -z "${HERDR_BIN:-}" || ! -x "${HERDR_BIN:-}" ]]; then
  if [[ -x /opt/herdr/herdr-linux-x86_64 ]]; then
    export HERDR_BIN=/opt/herdr/herdr-linux-x86_64
  else
    HERDR_BIN="$("$SCRIPT_DIR/download-binary.sh" | tail -n1)"
    export HERDR_BIN
  fi
fi

# shellcheck disable=SC1091
source "$SCRIPT_DIR/env.sh"

CLI="$SCRIPT_DIR/cli.sh"
TRANSCRIPT="$HERDR_VERIFY_EVIDENCE/cli-transcript.log"
SUMMARY="$HERDR_VERIFY_EVIDENCE/e2e-summary.txt"
MARKER_RUN="verify-e2e-run-$$"
MARKER_SEND="verify-e2e-send-$$"
STEP=0
FAILED=0

step() {
  STEP=$((STEP + 1))
  printf '\n▸ [%02d] %s\n' "$STEP" "$*"
}

ok() { printf '  ✓ %s\n' "$*"; }
fail() {
  FAILED=1
  printf '  ✗ %s\n' "$*" >&2
}

cleanup_instance() {
  "$SCRIPT_DIR/cleanup.sh" || true
}
trap cleanup_instance EXIT

{
  echo "verify-herdr e2e start $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "binary=$HERDR_BIN version=$("$HERDR_BIN" --version 2>/dev/null || echo unknown)"
  echo "evidence=$HERDR_VERIFY_EVIDENCE"

  step "server lifecycle — launch"
  "$SCRIPT_DIR/launch-server.sh"
  ok "server listening"

  step "status JSON doctor"
  "$SCRIPT_DIR/doctor.sh"
  ok "doctor PASS"

  step "workspace create + focus"
  create_json="$("$CLI" workspace create --cwd "$HERDR_HOME" --focus)"
  printf '%s\n' "$create_json" | tee "$HERDR_VERIFY_EVIDENCE/workspace-create.json" >/dev/null
  PANE="$(printf '%s' "$create_json" | python3 -c '
import json,sys
print(json.load(sys.stdin)["result"]["root_pane"]["pane_id"])
')"
  WS="$(printf '%s' "$create_json" | python3 -c '
import json,sys
print(json.load(sys.stdin)["result"]["workspace"]["workspace_id"])
' 2>/dev/null || true)"
  if [[ -z "$PANE" ]]; then
    fail "could not resolve pane id"
    exit 1
  fi
  ok "workspace=${WS:-unknown} pane=$PANE"
  "$CLI" workspace list | tee "$HERDR_VERIFY_EVIDENCE/workspace-list.txt" >/dev/null
  "$CLI" pane list | tee "$HERDR_VERIFY_EVIDENCE/pane-list.txt" >/dev/null

  step "pane run + wait-output + read"
  "$CLI" pane run "$PANE" echo "$MARKER_RUN" >/dev/null 2>&1
  "$CLI" pane wait-output "$PANE" --match "$MARKER_RUN" --timeout 5000 \
    >"$HERDR_VERIFY_EVIDENCE/wait-run.json" 2>&1
  "$CLI" pane read "$PANE" --source recent --format text \
    >"$HERDR_VERIFY_EVIDENCE/pane-read-run.txt" 2>&1
  if grep -q "$MARKER_RUN" "$HERDR_VERIFY_EVIDENCE/pane-read-run.txt"; then
    ok "marker $MARKER_RUN present"
  else
    fail "marker $MARKER_RUN missing from pane read"
    exit 1
  fi

  step "pane send-text + send-keys + wait-output"
  "$CLI" pane send-text "$PANE" "echo $MARKER_SEND" >/dev/null 2>&1
  "$CLI" pane send-keys "$PANE" Enter >/dev/null 2>&1
  "$CLI" pane wait-output "$PANE" --match "$MARKER_SEND" --timeout 5000 \
    >"$HERDR_VERIFY_EVIDENCE/wait-send.json" 2>&1
  "$CLI" pane read "$PANE" --source recent --format text \
    >"$HERDR_VERIFY_EVIDENCE/pane-read-send.txt" 2>&1
  if grep -q "$MARKER_SEND" "$HERDR_VERIFY_EVIDENCE/pane-read-send.txt"; then
    ok "marker $MARKER_SEND present"
  else
    fail "marker $MARKER_SEND missing from pane read"
    exit 1
  fi

  step "pane split (layout)"
  if "$CLI" pane split "$PANE" --direction right --focus \
    >"$HERDR_VERIFY_EVIDENCE/pane-split.txt" 2>&1; then
    ok "split ok"
  else
    fail "pane split failed"
    cat "$HERDR_VERIFY_EVIDENCE/pane-split.txt" >&2 || true
    exit 1
  fi
  "$CLI" pane list >"$HERDR_VERIFY_EVIDENCE/pane-list-after-split.txt"

  step "tab create"
  if [[ -n "${WS:-}" ]]; then
    tab_rc=0
    "$CLI" tab create --workspace "$WS" --cwd "$HERDR_HOME" --focus \
      >"$HERDR_VERIFY_EVIDENCE/tab-create.txt" 2>&1 || tab_rc=$?
  else
    tab_rc=0
    "$CLI" tab create --cwd "$HERDR_HOME" --focus \
      >"$HERDR_VERIFY_EVIDENCE/tab-create.txt" 2>&1 || tab_rc=$?
  fi
  if [[ "$tab_rc" -eq 0 ]]; then
    "$CLI" tab list >"$HERDR_VERIFY_EVIDENCE/tab-list.txt" 2>&1 || true
    ok "tab create ok"
  else
    fail "tab create failed"
    cat "$HERDR_VERIFY_EVIDENCE/tab-create.txt" >&2 || true
    exit 1
  fi

  step "config check + api snapshot"
  if "$CLI" config check >"$HERDR_VERIFY_EVIDENCE/config-check.txt" 2>&1; then
    ok "config check ok"
  else
    fail "config check failed"
    cat "$HERDR_VERIFY_EVIDENCE/config-check.txt" >&2 || true
    exit 1
  fi
  "$CLI" api snapshot >"$HERDR_VERIFY_EVIDENCE/api-snapshot.json" 2>&1 || \
    "$CLI" status --json >"$HERDR_VERIFY_EVIDENCE/api-snapshot.json"

  step "final status snapshot"
  "$CLI" status --json >"$HERDR_VERIFY_EVIDENCE/status-final.json"
  ok "status captured"

  {
    echo "PROOF PASS: verify-herdr e2e"
    echo "steps=$STEP"
    echo "pane=$PANE"
    echo "markers=$MARKER_RUN,$MARKER_SEND"
    echo "evidence=$HERDR_VERIFY_EVIDENCE"
    echo "version=$("$HERDR_BIN" --version 2>/dev/null || echo unknown)"
  } | tee "$SUMMARY"

  echo
  echo "════════════════════════════════════════"
  echo " VERIFY-HERDR E2E PASS"
  echo " evidence: $HERDR_VERIFY_EVIDENCE"
  echo "════════════════════════════════════════"
} 2>&1 | tee "$TRANSCRIPT"

exit "$FAILED"
