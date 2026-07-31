#!/usr/bin/env bash
# Start an isolated headless Herdr server and wait for the API socket.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/env.sh"

if [[ -S "$HERDR_SOCKET" ]]; then
  if env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH HOME="$HERDR_HOME" \
    "$HERDR_BIN" status --json 2>/dev/null | grep -q '"running":true'; then
    echo "server already running for HERDR_HOME=$HERDR_HOME"
    exit 0
  fi
fi

mkdir -p "$HERDR_HOME/.config/herdr"
LOG="$HERDR_HOME/.config/herdr/herdr-server.launch.log"

# Headless server (background). Record PID for cleanup.
env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH HOME="$HERDR_HOME" \
  "$HERDR_BIN" server >"$LOG" 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" >"$HERDR_VERIFY_META/server.pid"
echo "$HERDR_SOCKET" >"$HERDR_VERIFY_META/server.socket"
# Record identity so cleanup only kills this exact process.
printf '%s\n' "$HERDR_BIN" >"$HERDR_VERIFY_META/server.bin"
printf '%s\n' "$HERDR_HOME" >"$HERDR_VERIFY_META/server.home"

deadline=$((SECONDS + 20))
while (( SECONDS < deadline )); do
  if [[ -S "$HERDR_SOCKET" ]] && env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH HOME="$HERDR_HOME" \
    "$HERDR_BIN" status --json 2>/dev/null | grep -q '"running":true'; then
    echo "server ready pid=$SERVER_PID socket=$HERDR_SOCKET"
    env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH HOME="$HERDR_HOME" \
      "$HERDR_BIN" status --json >"$HERDR_VERIFY_EVIDENCE/status-after-launch.json" || true
    exit 0
  fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "error: server exited early; log:" >&2
    cat "$LOG" >&2 || true
    exit 1
  fi
  sleep 0.2
done

echo "error: timed out waiting for socket $HERDR_SOCKET" >&2
cat "$LOG" >&2 || true
exit 1
