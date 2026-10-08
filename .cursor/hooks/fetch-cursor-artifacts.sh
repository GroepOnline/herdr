#!/usr/bin/env bash
# Discover project Cursor skills, agents, hooks, and plugins; emit hook JSON
# or a markdown catalog. Used by sessionStart / postToolUse / workspaceOpen.
set -euo pipefail

ROOT="${CURSOR_PROJECT_DIR:-${CLAUDE_PROJECT_DIR:-}}"
if [[ -z "${ROOT}" ]]; then
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fi
cd "${ROOT}"

STATE_DIR="${ROOT}/.cursor/hooks/.state"
mkdir -p "${STATE_DIR}"

PRINT_MARKDOWN=0
WRITE_CACHE=0
FORCE_INJECT=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --print-markdown) PRINT_MARKDOWN=1; shift ;;
    --write-cache) WRITE_CACHE=1; shift ;;
    --force-inject) FORCE_INJECT=1; shift ;;
    *) shift ;;
  esac
done

INPUT_JSON=""
if [[ ! -t 0 ]]; then
  INPUT_JSON="$(cat || true)"
fi

hook_event=""
raw_conversation_id=""
if [[ -n "${INPUT_JSON}" ]]; then
  hook_event="$(printf '%s' "${INPUT_JSON}" | jq -r '.hook_event_name // empty' 2>/dev/null || true)"
  raw_conversation_id="$(printf '%s' "${INPUT_JSON}" | jq -r '.conversation_id // .session_id // empty' 2>/dev/null || true)"
fi

# Sanitize id so marker globs/paths cannot escape STATE_DIR or match broadly.
conversation_id="$(printf '%s' "${raw_conversation_id}" | tr -cd 'A-Za-z0-9._-')"
if [[ -z "${conversation_id}" ]]; then
  conversation_id="unknown"
fi

emit_empty() {
  jq -n '{}'
}

emit_continue() {
  jq -n '{continue: true}'
}

# Cheap no-scan exits first (cloud postToolUse runs very often).
if [[ "${PRINT_MARKDOWN}" -eq 0 && "${WRITE_CACHE}" -eq 0 ]]; then
  case "${hook_event}" in
    beforeSubmitPrompt)
      # Cannot inject additional_context on this event; keep the prompt unblocked.
      emit_continue
      exit 0
      ;;
    workspaceOpen)
      plugin_paths='[]'
      if [[ -d "${ROOT}/.cursor/plugins" ]]; then
        plugin_paths="$(find "${ROOT}/.cursor/plugins" -mindepth 1 -maxdepth 1 -type d -print | jq -R -s -c 'split("\n")|map(select(length>0))')"
      fi
      jq -n --argjson pluginPaths "${plugin_paths}" '{pluginPaths: $pluginPaths}'
      exit 0
      ;;
    postToolUse)
      if [[ "${FORCE_INJECT}" -eq 0 ]]; then
        # Any prior inject for this conversation → skip expensive scan.
        if compgen -G "${STATE_DIR}/injected-${conversation_id}-*" > /dev/null; then
          emit_empty
          exit 0
        fi
      fi
      ;;
    "")
      if [[ -z "${INPUT_JSON}" ]]; then
        emit_empty
        exit 0
      fi
      ;;
  esac
fi

collect_skills() {
  local path name
  if [[ ! -d .cursor/skills ]]; then
    return 0
  fi
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    name="$(basename "$(dirname "${path}")")"
    printf 'skill\t%s\t%s\n' "${name}" "${path}"
  done < <(find .cursor/skills -type f -name SKILL.md -print | LC_ALL=C sort)
}

collect_agents() {
  local path name
  if [[ ! -d .cursor/agents ]]; then
    return 0
  fi
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    name="$(basename "${path}" .md)"
    printf 'agent\t%s\t%s\n' "${name}" "${path}"
  done < <(find .cursor/agents -type f -name '*.md' -print | LC_ALL=C sort)
}

collect_hooks() {
  if [[ -f .cursor/hooks.json ]]; then
    jq -r '.hooks | keys[]?' .cursor/hooks.json 2>/dev/null | while read -r key; do
      [[ -z "${key}" ]] && continue
      printf 'hook\t%s\t.cursor/hooks.json\n' "${key}"
    done || true
  fi
}

collect_rules() {
  local path name
  if [[ ! -d .cursor/rules ]]; then
    return 0
  fi
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    name="$(basename "${path}" .mdc)"
    printf 'rule\t%s\t%s\n' "${name}" "${path}"
  done < <(find .cursor/rules -type f -name '*.mdc' -print | LC_ALL=C sort)
}

collect_commands() {
  local path name
  if [[ ! -d .cursor/commands ]]; then
    return 0
  fi
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    name="$(basename "${path}" .md)"
    printf 'command\t%s\t%s\n' "${name}" "${path}"
  done < <(find .cursor/commands -type f -name '*.md' -print | LC_ALL=C sort)
}

collect_plugins() {
  local path
  if [[ ! -d .cursor/plugins ]]; then
    return 0
  fi
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    printf 'plugin\t%s\t%s\n' "$(basename "${path}")" "${path}"
  done < <(find .cursor/plugins -mindepth 1 -maxdepth 1 -type d -print | LC_ALL=C sort)
}

ARTIFACTS="$(
  {
    collect_skills
    collect_agents
    collect_rules
    collect_commands
    collect_hooks
    collect_plugins
  } | LC_ALL=C sort
)"

HASH="$(printf '%s' "${ARTIFACTS}" | sha256sum | awk '{print $1}')"
SKILL_COUNT="$(printf '%s\n' "${ARTIFACTS}" | grep -c '^skill' || true)"
AGENT_COUNT="$(printf '%s\n' "${ARTIFACTS}" | grep -c '^agent' || true)"
RULE_COUNT="$(printf '%s\n' "${ARTIFACTS}" | grep -c '^rule' || true)"
COMMAND_COUNT="$(printf '%s\n' "${ARTIFACTS}" | grep -c '^command' || true)"
HOOK_COUNT="$(printf '%s\n' "${ARTIFACTS}" | grep -c '^hook' || true)"
PLUGIN_COUNT="$(printf '%s\n' "${ARTIFACTS}" | grep -c '^plugin' || true)"

markdown_catalog() {
  cat <<EOF
## Cursor artifact catalog (auto-fetched)
- Root: \`${ROOT}\`
- Hash: \`${HASH}\`
- Skills: ${SKILL_COUNT} | Agents: ${AGENT_COUNT} | Rules: ${RULE_COUNT} | Commands: ${COMMAND_COUNT} | Hook events: ${HOOK_COUNT} | Plugins: ${PLUGIN_COUNT}
- SSOT: \`.cursor/\` only (no mirrored \`.agents\` / \`.claude\` / \`.codex\` copies)

| Kind | Name | Path |
| --- | --- | --- |
EOF
  while IFS=$'\t' read -r kind name path; do
    [[ -z "${kind}" ]] && continue
    printf '| %s | `%s` | `%s` |\n' "${kind}" "${name}" "${path}"
  done <<< "${ARTIFACTS}"
  cat <<'EOF'

See `.cursor/INDEX.md` for the full catalog and `.cursor/commands/` for slash
commands. Project skills live under `.cursor/skills/` (including `verify-herdr`).
EOF
}

CATALOG_MD="$(markdown_catalog)"
CACHE_FILE="${STATE_DIR}/catalog.md"
HASH_FILE="${STATE_DIR}/catalog.sha256"
printf '%s\n' "${CATALOG_MD}" > "${CACHE_FILE}"
printf '%s\n' "${HASH}" > "${HASH_FILE}"

if [[ "${PRINT_MARKDOWN}" -eq 1 ]]; then
  printf '%s\n' "${CATALOG_MD}"
  exit 0
fi

if [[ "${WRITE_CACHE}" -eq 1 && -z "${hook_event}" ]]; then
  exit 0
fi

mark_injected() {
  local marker="${STATE_DIR}/injected-${conversation_id}-${HASH}"
  # Replace prior markers for this conversation (safe: conversation_id sanitized).
  find "${STATE_DIR}" -maxdepth 1 -type f -name "injected-${conversation_id}-*" -delete 2>/dev/null || true
  # Atomic-ish create so concurrent hooks converge on one marker name.
  : > "${marker}"
}

should_inject=0
case "${hook_event}" in
  sessionStart)
    should_inject=1
    mark_injected
    ;;
  postToolUse)
    marker="${STATE_DIR}/injected-${conversation_id}-${HASH}"
    if [[ "${FORCE_INJECT}" -eq 1 || ! -f "${marker}" ]]; then
      # Re-check after scan: another hook may have raced ahead.
      if [[ "${FORCE_INJECT}" -eq 0 ]] && compgen -G "${STATE_DIR}/injected-${conversation_id}-*" > /dev/null; then
        should_inject=0
      else
        should_inject=1
        mark_injected
      fi
    fi
    ;;
  *)
    emit_empty
    exit 0
    ;;
esac

if [[ "${should_inject}" -eq 1 ]]; then
  jq -n --arg ctx "${CATALOG_MD}" --arg hash "${HASH}" \
    '{additional_context: $ctx, env: {PROJECT_CURSOR_CATALOG_HASH: $hash}}'
else
  emit_empty
fi
