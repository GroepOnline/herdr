#!/usr/bin/env bash
# beforeShellExecution: soft guidance for Herdr pane/tab canvas commands.
# Never blocks. Fails open. Session lesson: send-keys q + bash -lc → qbash paste.
set -euo pipefail

input="$(cat || true)"
command=""
if command -v jq >/dev/null 2>&1; then
  command="$(printf '%s' "$input" | jq -r '.command // empty' 2>/dev/null || true)"
else
  command="$(printf '%s' "$input" | python3 -c 'import json,sys
try: print(json.load(sys.stdin).get("command") or "")
except Exception: print("")' 2>/dev/null || true)"
fi

# Only annotate herdr pane/tab canvas traffic
if ! printf '%s' "$command" | grep -qiE 'herdr[[:space:]]+(pane|tab)[[:space:]]'; then
  echo '{ "permission": "allow" }'
  exit 0
fi

risky=0
msg=""

if printf '%s' "$command" | grep -qiE 'send-keys.*(^|[[:space:]])q([[:space:]]|$)|pane[[:space:]]+send-keys[[:space:]]+[^;]*[[:space:]]q'; then
  risky=1
  msg="${msg}Avoid send-keys q immediately before pane run (causes qbash paste). "
fi

if printf '%s' "$command" | grep -qiE 'pane[[:space:]]+run.*bash[[:space:]]+-lc.*(less|glow)'; then
  risky=1
  msg="${msg}Prefer: herdr pane run <pane> /usr/bin/less -R <absfile> (no bash -lc). "
fi

if printf '%s' "$command" | grep -qiE 'glow|[[:space:]]less[[:space:]]+-R'; then
  if printf '%s' "$command" | grep -qiE 'bash[[:space:]]+-lc'; then
    risky=1
    msg="${msg}glow/less via bash -lc freezes or races; use /usr/bin/less -R directly. "
  fi
fi

if [[ "$risky" -eq 1 ]]; then
  # JSON-escape message lightly
  safe="$(printf '%s' "$msg" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))' 2>/dev/null || printf '"herdr canvas tip"')"
  printf '{\n  "permission": "allow",\n  "agent_message": %s\n}\n' "$safe"
  exit 0
fi

echo '{ "permission": "allow" }'
exit 0
