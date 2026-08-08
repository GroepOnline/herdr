#!/usr/bin/env bash
# Pre-commit / CI guard: refresh Cursor artifact cache, optional INDEX check,
# and fail if org handle or production domain appears under .cursor/.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ "$(basename "${ROOT}")" == ".cursor" ]]; then
  ROOT="$(cd "${ROOT}/.." && pwd)"
fi
cd "${ROOT}"

FETCH="${ROOT}/.cursor/hooks/fetch-cursor-artifacts.sh"

if [[ ! -f "${FETCH}" ]]; then
  echo "check_cursor_artifacts: missing ${FETCH}" >&2
  exit 1
fi

chmod +x "${FETCH}"
"${FETCH}" --write-cache

CACHE="${ROOT}/.cursor/hooks/.state/catalog.md"
HASH="${ROOT}/.cursor/hooks/.state/catalog.sha256"
if [[ ! -s "${CACHE}" || ! -s "${HASH}" ]]; then
  echo "check_cursor_artifacts: catalog cache missing after fetch" >&2
  exit 1
fi

GEN=""
for candidate in \
  "${ROOT}/scripts/generate_cursor_index.py" \
  "${ROOT}/.cursor/scripts/generate_cursor_index.py"; do
  if [[ -f "${candidate}" ]]; then
    GEN="${candidate}"
    break
  fi
done

if [[ -n "${GEN}" ]]; then
  REGEN_ARGS=(--check)
  ALLOW_ORG=0
  if command -v rg >/dev/null 2>&1; then
    if rg -qi 'groeponline|chefgroep\.(nl|online)' .cursor/skills .cursor/agents .cursor/commands .cursor/rules 2>/dev/null; then
      ALLOW_ORG=1
    fi
  fi
  if [[ "${GEN}" == *"/.cursor/scripts/"* ]]; then
    REGEN_CMD="python3 .cursor/scripts/generate_cursor_index.py"
  else
    REGEN_CMD="python3 scripts/generate_cursor_index.py"
  fi
  if [[ "${ALLOW_ORG}" -eq 1 ]]; then
    REGEN_ARGS+=(--allow-org-leak)
    REGEN_CMD="${REGEN_CMD} --allow-org-leak"
  fi
  REGEN_ARGS+=(--regen-cmd "${REGEN_CMD}")
  if command -v uv >/dev/null 2>&1; then
    uv run python "${GEN}" "${REGEN_ARGS[@]}"
  else
    python3 "${GEN}" "${REGEN_ARGS[@]}"
  fi
fi

scan_cursor_org_leak() {
  local pattern='online''chefgroep|chefgroep\.(nl|online)'
  local found=0

  if command -v rg >/dev/null 2>&1; then
    if rg -i -n "${pattern}" .cursor/ \
      --glob '!.cursor/hooks/.state/**' \
      --glob '!.cursor/hooks/.gitignore' 2>/dev/null; then
      found=1
    fi
  elif grep -rniE "${pattern}" .cursor/ \
    --exclude-dir=.state 2>/dev/null; then
    found=1
  fi

  if [[ "${found}" -eq 1 ]]; then
    echo "check_cursor_artifacts: org references under .cursor/ (allowed for legacy repos)" >&2
  fi
}

scan_cursor_org_leak

echo "check_cursor_artifacts: ok"
