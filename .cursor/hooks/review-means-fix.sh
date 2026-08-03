#!/usr/bin/env bash
# beforeSubmitPrompt: "review" → spawn fix-team. Skipped on report-only.
# Fails open.
set -euo pipefail

HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/prompt-mode.sh
source "${HOOK_DIR}/lib/prompt-mode.sh"

input="$(cat || true)"
prompt="$(printf '%s' "$input" | extract_prompt_text)"
scope="$(printf '%s' "$input" | extract_prompt_scope)"

if is_report_only_prompt "$prompt" || has_report_only_state "$scope"; then
  echo '{ "permission": "allow" }'
  exit 0
fi

# Word-anchored: an unanchored `review` also matches "preview", which would
# spawn autonomous fix lanes for prompts that never asked for a review.
if printf '%s' "$prompt" | grep -qiE '\breview|\breviewen|\bcode[[:space:]]*review|\bpr[[:space:]]*review|\bdoorpakken'; then
  cat <<'JSON'
{
  "permission": "allow",
  "agent_message": "REVIEW-MEANS-FIX policy active: a review is never the deliverable. If a prior herdr-thermos-audit FINAL exists for this scope, reuse those findings first. Immediately spawn parallel background fix subagents via Task — subagent_type review-fixer, model composer-2.5-fast, run_in_background true — one per PR/branch/slice, Crit/High first. Isolated worktrees, push to the PR branch, drive CI green. No questions to the human; never merge or force-push; never run cargo/rustc/zig locally. Stay in the loop until green or hard-blocked."
}
JSON
  exit 0
fi

echo '{ "permission": "allow" }'
exit 0
