#!/usr/bin/env bash
# Isolated herdr CLI wrapper. Requires env.sh sourced or will source it.
set -euo pipefail

if [[ -z "${HERDR_BIN:-}" || -z "${HERDR_HOME:-}" ]]; then
  # shellcheck disable=SC1091
  source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/env.sh"
fi

exec env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH \
  HOME="$HERDR_HOME" \
  "$HERDR_BIN" "$@"
