#!/usr/bin/env bash
# End-to-end proof: workspace create → pane run → wait-output → pane read.
# Leaves evidence under HERDR_VERIFY_EVIDENCE; cleans up the instance afterward.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/env.sh"

CLI="$SCRIPT_DIR/cli.sh"
MARKER="verify-herdr-ok-$$"
TRANSCRIPT="$HERDR_VERIFY_EVIDENCE/cli-transcript.log"

cleanup_instance() {
  "$SCRIPT_DIR/cleanup.sh" || true
}
trap cleanup_instance EXIT

{
  echo "== launch =="
  "$SCRIPT_DIR/launch-server.sh"
  echo "== doctor =="
  "$SCRIPT_DIR/doctor.sh"
  echo "== workspace create =="
  create_json="$("$CLI" workspace create --cwd "$HERDR_HOME" --focus)"
  printf '%s\n' "$create_json" | tee "$HERDR_VERIFY_EVIDENCE/workspace-create.json"
  echo "== pane list =="
  "$CLI" pane list | tee "$HERDR_VERIFY_EVIDENCE/pane-list.txt"
  # Prefer explicit override, else pane_id from create response, else focused pane in list.
  if [[ -n "${HERDR_VERIFY_PANE:-}" ]]; then
    PANE="$HERDR_VERIFY_PANE"
  else
    PANE="$(printf '%s' "$create_json" | python3 -c '
import json,sys
data=json.load(sys.stdin)
print(data["result"]["root_pane"]["pane_id"])
')"
  fi
  if [[ -z "$PANE" ]]; then
    echo "PROOF FAIL: could not resolve pane id from workspace create" >&2
    exit 1
  fi
  echo "using pane=$PANE marker=$MARKER"
  echo "== pane run =="
  # Use echo so the shell prints a clean line (printf quoting varies by shell).
  "$CLI" pane run "$PANE" echo "$MARKER"
  echo "== wait-output =="
  "$CLI" pane wait-output "$PANE" --match "$MARKER" --timeout 5000
  echo "== pane read =="
  "$CLI" pane read "$PANE" --source recent --format text \
    | tee "$HERDR_VERIFY_EVIDENCE/pane-read.txt"
} 2>&1 | tee "$TRANSCRIPT"

if ! grep -q "$MARKER" "$HERDR_VERIFY_EVIDENCE/pane-read.txt"; then
  echo "PROOF FAIL: marker '$MARKER' not found in pane-read.txt" >&2
  exit 1
fi

echo "PROOF PASS: marker found in pane-read.txt"
echo "evidence: $HERDR_VERIFY_EVIDENCE"
# trap runs cleanup; evidence survives
exit 0
