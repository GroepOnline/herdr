#!/usr/bin/env bash
# Shared helpers for beforeSubmitPrompt review/audit mode hooks.
# Sourced by audit-report-only.sh and review-means-fix.sh.

prompt_mode_state_dir() {
  printf '%s\n' "${HOME}/.cursor/hooks/state"
}

prompt_mode_state_file() {
  printf '%s\n' "$(prompt_mode_state_dir)/last-prompt-review-mode"
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
  printf 'report-only\n' >"$(prompt_mode_state_file)"
}

clear_report_only_state() {
  rm -f "$(prompt_mode_state_file)" 2>/dev/null || true
}

has_report_only_state() {
  local f
  f="$(prompt_mode_state_file)"
  [[ -f "$f" ]] && grep -qx 'report-only' "$f" 2>/dev/null
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
