#!/usr/bin/env bash
# beforeSubmitPrompt: report-only / thermos /audit → herdr-thermos-audit policy.
# Outranks review-means-fix. Fails open.
set -euo pipefail

HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/prompt-mode.sh
source "${HOOK_DIR}/lib/prompt-mode.sh"

input="$(cat || true)"
prompt="$(printf '%s' "$input" | extract_prompt_text)"

if is_report_only_prompt "$prompt"; then
  set_report_only_state
  cat <<'JSON'
{
  "permission": "allow",
  "agent_message": "AUDIT-REPORT-ONLY policy active (outranks review-means-fix). Load skill herdr-thermos-audit. Launch parallel Thermos lanes (thermo-nuclear-review-subagent + thermo-nuclear-code-quality-review-subagent) and optional audit-scope-mapper in one turn, all run_in_background true. Do NOT spawn review-fixer; do NOT edit/commit/push the audited repo. When HERDR_ENV=1: paint map/findings/connections/where-live via herdr tabs; prefer `herdr pane run <id> /usr/bin/less -R <file>` (never send-keys q then bash -lc). Write /tmp/<repo>-audit-<date>/canvas/03-FINAL.md and joep-brain ingest_report at fan-in."
}
JSON
  exit 0
fi

clear_report_only_state
echo '{ "permission": "allow" }'
exit 0
