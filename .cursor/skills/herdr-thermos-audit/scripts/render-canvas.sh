#!/usr/bin/env bash
# Render audit canvas markdown into a Herdr pane (fail open).
set -euo pipefail

PANE=""
DIR=""
FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pane) PANE="${2:-}"; shift 2 ;;
    --dir) DIR="${2:-}"; shift 2 ;;
    --file) FILE="${2:-}"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ -z "${PANE}" ]]; then
  echo "usage: render-canvas.sh --pane <id> (--dir <canvas-dir> | --file <md>)" >&2
  exit 2
fi

if [[ "${HERDR_ENV:-}" != "1" ]]; then
  echo "HERDR_ENV!=1 — refusing to drive Herdr panes" >&2
  exit 1
fi

TARGET=""
if [[ -n "${FILE}" ]]; then
  TARGET="${FILE}"
elif [[ -n "${DIR}" ]]; then
  # Prefer numbered overview, else concatenate
  if [[ -f "${DIR}/00-map.md" ]]; then
    TARGET="${DIR}/00-map.md"
  else
    TARGET="${DIR}/.combined.md"
    cat "${DIR}"/*.md > "${TARGET}" 2>/dev/null || true
  fi
else
  echo "need --dir or --file" >&2
  exit 2
fi

if [[ ! -f "${TARGET}" ]]; then
  echo "missing canvas file: ${TARGET}" >&2
  exit 1
fi

# Sticky and scrollable: the less binary directly, never `bash -lc` + cat (which
# exits immediately and cannot reload with `R`). TARGET stays a single argument,
# so paths containing spaces survive.
herdr pane rename "${PANE}" "thermos-canvas" >/dev/null 2>&1 || true
herdr pane run "${PANE}" /usr/bin/less -R "${TARGET}"
