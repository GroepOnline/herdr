#!/usr/bin/env bash
# Tear down the isolated instance started by launch-server.sh.
# Keeps HERDR_VERIFY_EVIDENCE intact.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Prefer existing env; otherwise recover meta from HERDR_HOME if set.
if [[ -z "${HERDR_BIN:-}" || -z "${HERDR_HOME:-}" ]]; then
  if [[ -n "${HERDR_HOME:-}" && -f "${HERDR_HOME}/.verify-meta/server.pid" ]]; then
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/env.sh"
  elif [[ -n "${HERDR_VERIFY_META:-}" && -f "${HERDR_VERIFY_META}/../.config/herdr/config.toml" ]]; then
    export HERDR_HOME="$(cd "${HERDR_VERIFY_META}/.." && pwd)"
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/env.sh"
  else
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/env.sh"
  fi
fi

EVIDENCE="${HERDR_VERIFY_EVIDENCE:-}"
META="${HERDR_VERIFY_META:-$HERDR_HOME/.verify-meta}"
PID_FILE="$META/server.pid"

stopped=0
if [[ -S "${HERDR_SOCKET:-$HERDR_HOME/.config/herdr/herdr.sock}" ]]; then
  if env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH HOME="$HERDR_HOME" \
    "$HERDR_BIN" server stop >/dev/null 2>&1; then
    stopped=1
  fi
fi

if [[ -f "$PID_FILE" ]]; then
  pid="$(cat "$PID_FILE")"
  expected_bin="$(cat "$META/server.bin" 2>/dev/null || true)"
  expected_home="$(cat "$META/server.home" 2>/dev/null || echo "$HERDR_HOME")"
  if kill -0 "$pid" 2>/dev/null; then
    # Only signal the recorded PID when cmdline still looks like our isolated server.
    cmdline="$(tr '\0' ' ' </proc/"$pid"/cmdline 2>/dev/null || true)"
    environ="$(tr '\0' '\n' </proc/"$pid"/environ 2>/dev/null || true)"
    bin_ok=0
    home_ok=0
    if [[ -n "$expected_bin" && "$cmdline" == *"$expected_bin"* ]]; then
      bin_ok=1
    elif [[ "$cmdline" == *herdr* && "$cmdline" == *server* ]]; then
      bin_ok=1
    fi
    if [[ "$environ" == *"HOME=$expected_home"* ]]; then
      home_ok=1
    fi
    if [[ "$bin_ok" -eq 1 && "$home_ok" -eq 1 ]]; then
      kill "$pid" 2>/dev/null || true
      for _ in 1 2 3 4 5 6 7 8 9 10; do
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.2
      done
      if kill -0 "$pid" 2>/dev/null; then
        kill -9 "$pid" 2>/dev/null || true
      fi
    else
      echo "cleanup: refusing to kill pid=$pid (cmdline/home mismatch); left running" >&2
      echo "  cmdline=$cmdline" >&2
    fi
  fi
  rm -f "$PID_FILE"
fi

# Copy server logs into evidence before wiping home.
if [[ -n "$EVIDENCE" && -d "$EVIDENCE" ]]; then
  if [[ -f "$HERDR_HOME/.config/herdr/herdr-server.log" ]]; then
    cp -f "$HERDR_HOME/.config/herdr/herdr-server.log" "$EVIDENCE/herdr-server.log" || true
  fi
  if [[ -f "$HERDR_HOME/.config/herdr/herdr-server.launch.log" ]]; then
    cp -f "$HERDR_HOME/.config/herdr/herdr-server.launch.log" "$EVIDENCE/herdr-server.launch.log" || true
  fi
  echo "cleanup stopped_via_api=$stopped home=$HERDR_HOME" >"$EVIDENCE/cleanup.txt"
fi

# Remove scratch home only (never evidence).
if [[ -n "${HERDR_HOME:-}" && "$HERDR_HOME" == /tmp/herdr-verify-home.* ]]; then
  rm -rf "$HERDR_HOME"
fi

echo "cleanup done (evidence kept at ${EVIDENCE:-none})"
