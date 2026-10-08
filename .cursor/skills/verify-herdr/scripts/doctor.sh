#!/usr/bin/env bash
# Read-only check: is this isolated Herdr instance worth driving?
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/env.sh"

fail() { echo "doctor FAIL: $*" >&2; exit 1; }

[[ -x "$HERDR_BIN" ]] || fail "HERDR_BIN not executable: $HERDR_BIN"
version="$("$HERDR_BIN" --version 2>&1 || true)"
[[ "$version" == herdr* ]] || fail "unexpected --version: $version"

status_json="$(env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH HOME="$HERDR_HOME" \
  "$HERDR_BIN" status --json 2>&1)" || fail "status --json failed: $status_json"

printf '%s\n' "$status_json" >"$HERDR_VERIFY_EVIDENCE/doctor.json"

echo "$status_json" | grep -q '"running":true' || fail "server.running != true"
echo "$status_json" | grep -q "$HERDR_HOME/.config/herdr/herdr.sock" \
  || fail "status socket not under HERDR_HOME"

if [[ -f "$HERDR_VERIFY_META/server.pid" ]]; then
  pid="$(cat "$HERDR_VERIFY_META/server.pid")"
  kill -0 "$pid" 2>/dev/null || fail "recorded server pid $pid is not alive"
fi

# compatible may be null on some builds; only fail on explicit false
if echo "$status_json" | grep -q '"compatible":false'; then
  fail "server.compatible == false"
fi

echo "doctor PASS"
echo "  version=$version"
echo "  home=$HERDR_HOME"
echo "  socket=$HERDR_SOCKET"
exit 0
