#!/usr/bin/env bash
# Shared helpers for beforeSubmitPrompt review/audit mode hooks.
# Sourced by audit-report-only.sh and review-means-fix.sh.

prompt_mode_state_dir() {
  printf '%s\n' "${HOME}/.cursor/hooks/state"
}

# Sanitize the caller-supplied scope so it cannot escape the state dir or match
# broadly. Empty scope falls back to a shared bucket.
prompt_mode_scope() {
  local scope
  scope="$(printf '%s' "${1:-}" | tr -cd 'A-Za-z0-9._-')"
  printf '%s\n' "${scope:-global}"
}

# State is per conversation/workspace: an audit prompt in one conversation must
# not suppress fix behavior in another, and unrelated prompts must not clear it.
prompt_mode_state_file() {
  printf '%s\n' "$(prompt_mode_state_dir)/last-prompt-review-mode-$(prompt_mode_scope "${1:-}")"
}

# True if the user prompt asks for report-only / thermos audit (no auto-fix).
is_report_only_prompt() {
  local prompt="${1:-}"
  [[ -n "$prompt" ]] || return 1
  printf '%s' "$prompt" | grep -qiE \
    '/audit|[[:space:]]audit[[:space:]]|^audit[[:space:]]|\baudit\b|thermos|niet[[:space:]]+handelen|report[-[:space:]]?only|geen[[:space:]]+fix|puur.*(review|audit)|review.*zonder.*(fix|handelen)|alleen[[:space:]]+(review|audit)|no[[:space:]]+fix(es|ing)?'
}

set_report_only_state() {
  mkdir -p "$(prompt_mode_state_dir)"
  printf 'report-only\n' >"$(prompt_mode_state_file "${1:-}")"
}

clear_report_only_state() {
  rm -f "$(prompt_mode_state_file "${1:-}")" 2>/dev/null || true
}

has_report_only_state() {
  local f
  f="$(prompt_mode_state_file "${1:-}")"
  [[ -f "$f" ]] && grep -qx 'report-only' "$f" 2>/dev/null
}

extract_prompt_scope() {
  # stdin: hook JSON → stdout: conversation/workspace id (may be empty)
  if command -v jq >/dev/null 2>&1; then
    jq -r '.conversation_id // .session_id // .workspace_id // empty' 2>/dev/null || true
  else
    python3 -c 'import json,sys
try:
  d=json.load(sys.stdin)
except Exception:
  d={}
print(d.get("conversation_id") or d.get("session_id") or d.get("workspace_id") or "")' 2>/dev/null || true
  fi
}

extract_prompt_text() {
  # stdin: hook JSON → stdout: prompt text (may be empty)
  if command -v jq >/dev/null 2>&1; then
    jq -r '.prompt // .text // empty' 2>/dev/null || true
  else
    python3 -c 'import json,sys
try:
  d=json.load(sys.stdin)
except Exception:
  d={}
print(d.get("prompt") or d.get("text") or "")' 2>/dev/null || true
  fi
}
